//! Sparse free heat. Factual relations and retrieval policy live outside this module.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

#[derive(Clone, Debug)]
pub struct Graph {
    pub ids: Vec<String>,
    adjacency: Vec<Vec<(usize, f64)>>,
    degree: Vec<f64>,
    max_degree: f64,
    edge_count: usize,
    max_neighbors: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Heat {
    pub operator: String,
    pub status: String,
    pub scores: Vec<f64>,
    pub theta: f64,
    pub max_degree: f64,
    pub iterations: usize,
    pub edge_visits: usize,
    pub truncation_tail_bound: f64,
    pub floating_point_error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BreadthFirst {
    pub nodes: Vec<(usize, usize)>,
    pub complete: bool,
    pub edge_visits: usize,
}

/// Controls an incremental trajectory. Unlike `heat`, its time is not limited
/// to ten: each continuation is split into bounded Poisson series instead.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectralConfig {
    pub max_increment: f64,
    pub poisson_tail_tolerance: f64,
}

impl Default for SpectralConfig {
    fn default() -> Self {
        Self {
            max_increment: 10.0,
            poisson_tail_tolerance: 1e-14,
        }
    }
}

/// One request owns one budget, including initialization and observables.
/// Node visits include vector initialization, copies, and arithmetic passes.
/// An operation is charged before it starts; counters can conservatively
/// include its unfinished part if time or cancellation interrupts it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectralLimits {
    #[serde(alias = "max_steps")]
    pub max_increments: usize,
    pub max_operator_applications: usize,
    pub max_node_visits: usize,
    pub max_edge_visits: usize,
    pub max_samples: usize,
    #[serde(alias = "max_time_ms")]
    pub max_elapsed_ms: u64,
}

impl Default for SpectralLimits {
    fn default() -> Self {
        Self {
            max_increments: 32_768,
            max_operator_applications: 2_000_000,
            max_node_visits: 200_000_000,
            max_edge_visits: 200_000_000,
            max_samples: 256,
            max_elapsed_ms: 30_000,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpectralWork {
    pub increments: usize,
    pub operator_applications: usize,
    pub node_visits: usize,
    pub edge_visits: usize,
    pub samples: usize,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpectralError {
    /// Accumulated L2 Poisson-tail bound under the exact-arithmetic operator.
    pub truncation_l2: f64,
    /// Forward-error model for f64 arithmetic, NOT a certified bound. In
    /// particular, the platform exp implementation is not interval arithmetic.
    pub roundoff_l2_estimate: f64,
    pub roundoff_certified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectralInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectralSample {
    pub theta: f64,
    pub physical_time: Option<f64>,
    /// h(theta) = exp(-theta A)v, for selecting evidence.
    pub h: Vec<f64>,
    /// y(theta) = exp(-(theta/2) A)v, for the SAME-scale observable.
    pub y: Vec<f64>,
    pub z_q: Option<f64>,
    /// Seed-conditioned spectral curvature; computed from a squared residual,
    /// not cancellation between two nearly equal spectral moments.
    pub c_q: Option<f64>,
    pub spectral_mean: Option<f64>,
    /// `estimated` never certifies a stopping window. Exact lack of seed or
    /// dynamics and unresolved residuals are explicitly non-informative.
    pub quality: String,
    pub error: SpectralError,
    pub heat_error: SpectralError,
    /// These intervals include an UNCERTIFIED rounding estimate. They are
    /// useful for an experimental policy, never a proof of semantic coverage
    /// or a certified interval enclosure of the floating-point calculation.
    pub z_q_interval_estimate: Option<SpectralInterval>,
    pub c_q_interval_estimate: Option<SpectralInterval>,
    pub work: SpectralWork,
}

struct SpectralBudget {
    limits: SpectralLimits,
    work: SpectralWork,
    start: Instant,
    status: &'static str,
}

impl SpectralBudget {
    fn check(&mut self, cancel: &AtomicBool) -> Result<bool> {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        self.work.elapsed_ms = self.start.elapsed().as_millis().min(u64::MAX as u128) as u64;
        if self.status != "complete" {
            return Ok(false);
        }
        if self.work.elapsed_ms >= self.limits.max_elapsed_ms {
            self.status = "time_limit";
            return Ok(false);
        }
        Ok(true)
    }

    fn charge(
        &mut self,
        nodes: usize,
        edges: usize,
        applications: usize,
        increments: usize,
        cancel: &AtomicBool,
    ) -> Result<bool> {
        if !self.check(cancel)? {
            return Ok(false);
        }
        let additions = [
            (
                self.work.node_visits,
                nodes,
                self.limits.max_node_visits,
                "node_limit",
            ),
            (
                self.work.edge_visits,
                edges,
                self.limits.max_edge_visits,
                "edge_limit",
            ),
            (
                self.work.operator_applications,
                applications,
                self.limits.max_operator_applications,
                "operator_limit",
            ),
            (
                self.work.increments,
                increments,
                self.limits.max_increments,
                "increment_limit",
            ),
        ];
        for (used, added, limit, reason) in additions {
            if used.checked_add(added).is_none_or(|total| total > limit) {
                self.status = reason;
                return Ok(false);
            }
        }
        self.work.node_visits += nodes;
        self.work.edge_visits += edges;
        self.work.operator_applications += applications;
        self.work.increments += increments;
        Ok(true)
    }
}

struct SpectralState {
    theta: f64,
    values: Vec<f64>,
    error: SpectralError,
}

#[derive(Default)]
struct SpectralBuffers {
    term: Vec<f64>,
    next: Vec<f64>,
    sum: Vec<f64>,
}

/// Fixed graph and fixed seed, sampled in strictly increasing theta. Only the
/// two current states and three reusable scratch vectors are retained, O(V+E).
/// Calling `sample` does no work at future scales. `None` means that a request
/// limit stopped an unfinished sample; only earlier returned samples are valid.
pub struct SpectralTrajectory<'a> {
    graph: &'a Graph,
    config: SpectralConfig,
    budget: SpectralBudget,
    h: SpectralState,
    y: SpectralState,
    buffers: SpectralBuffers,
    seed_norm: f64,
    has_dynamics: bool,
    last_sample: Option<f64>,
    edge_count: usize,
    max_neighbors: usize,
}

impl Graph {
    /// Undirected projection. Duplicate pairs never accumulate weight.
    pub fn new(ids: Vec<String>, edges: &[(String, String)]) -> Result<Self> {
        Self::weighted(
            ids,
            &edges
                .iter()
                .map(|(a, b)| (a.clone(), b.clone(), 1.0))
                .collect::<Vec<_>>(),
        )
    }

    pub fn weighted(mut ids: Vec<String>, edges: &[(String, String, f64)]) -> Result<Self> {
        if ids.len() > 100_000 || edges.len() > 1_000_000 {
            return Err(Error::invalid("graph size limit exceeded"));
        }
        ids.sort();
        ids.dedup();
        let index: BTreeMap<_, _> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();
        let mut pairs = BTreeMap::new();
        for (a, b, w) in edges {
            if !w.is_finite() || *w <= 0.0 {
                return Err(Error::invalid("edge weight must be finite and positive"));
            }
            let (&a, &b) = (
                index
                    .get(a.as_str())
                    .ok_or_else(|| Error::invalid("unknown graph node"))?,
                index
                    .get(b.as_str())
                    .ok_or_else(|| Error::invalid("unknown graph node"))?,
            );
            if a != b {
                let pair = (a.min(b), a.max(b));
                if let Some(previous) = pairs.insert(pair, *w)
                    && previous != *w
                {
                    return Err(Error::invalid("conflicting duplicate weights"));
                }
            }
        }
        let mut adjacency = vec![vec![]; ids.len()];
        let mut degree = vec![0.0; ids.len()];
        for ((a, b), w) in pairs {
            adjacency[a].push((b, w));
            adjacency[b].push((a, w));
            degree[a] += w;
            degree[b] += w;
        }
        let max_degree = degree.iter().copied().fold(0.0, f64::max);
        if !max_degree.is_finite() {
            return Err(Error::invalid("degree overflow"));
        }
        Ok(Self {
            ids,
            edge_count: adjacency.iter().map(Vec::len).sum(),
            max_neighbors: adjacency.iter().map(Vec::len).max().unwrap_or(0),
            adjacency,
            degree,
            max_degree,
        })
    }

    pub fn max_degree(&self) -> f64 {
        self.max_degree
    }

    /// Prepare a fixed seed once. The existing `heat` API retains its original
    /// theta range, normalization and output contract.
    pub fn spectral_trajectory(
        &self,
        seeds: &[(String, f64)],
        config: SpectralConfig,
        limits: SpectralLimits,
        cancel: &AtomicBool,
    ) -> Result<SpectralTrajectory<'_>> {
        if !config.max_increment.is_finite()
            || !(0.0..=10.0).contains(&config.max_increment)
            || config.max_increment == 0.0
            || !config.poisson_tail_tolerance.is_finite()
            || !(0.0..1.0).contains(&config.poisson_tail_tolerance)
            || config.poisson_tail_tolerance == 0.0
        {
            return Err(Error::invalid("invalid spectral precision or increment"));
        }
        let mut trajectory = SpectralTrajectory {
            graph: self,
            config,
            budget: SpectralBudget {
                limits,
                work: SpectralWork::default(),
                start: Instant::now(),
                status: "complete",
            },
            h: SpectralState {
                theta: 0.0,
                values: Vec::new(),
                error: SpectralError::default(),
            },
            y: SpectralState {
                theta: 0.0,
                values: Vec::new(),
                error: SpectralError::default(),
            },
            buffers: SpectralBuffers::default(),
            seed_norm: 0.0,
            has_dynamics: false,
            last_sample: None,
            edge_count: self.edge_count,
            max_neighbors: self.max_neighbors,
        };
        let n = self.ids.len();
        let lookup_visits = (usize::BITS - n.leading_zeros()) as usize + 1;
        // Account for all preparation passes and scratch allocation before
        // doing O(V) work, even for a graph consisting entirely of isolates.
        // Binary seed lookups also have a bounded number of node visits.
        if !trajectory.budget.charge(
            n.saturating_mul(8)
                .saturating_add(seeds.len().saturating_mul(lookup_visits)),
            trajectory.edge_count,
            0,
            0,
            cancel,
        )? {
            return Ok(trajectory);
        }
        let mut v = vec![0.0; n];
        for (seed_index, (id, mass)) in seeds.iter().enumerate() {
            if seed_index % 256 == 0 && !trajectory.budget.check(cancel)? {
                return Ok(trajectory);
            }
            if !mass.is_finite() || *mass < 0.0 {
                return Err(Error::invalid("invalid seed mass"));
            }
            let i = self
                .ids
                .binary_search(id)
                .map_err(|_| Error::invalid("unknown seed"))?;
            v[i] += mass;
        }
        let mass: f64 = v.iter().sum();
        if !mass.is_finite() {
            return Err(Error::invalid("seed mass overflow"));
        }
        let mut squared_norm = 0.0;
        for x in &mut v {
            if mass > 0.0 {
                *x /= mass;
            }
            squared_norm += *x * *x;
        }
        trajectory.seed_norm = squared_norm.sqrt();
        for (i, neighbors) in self.adjacency.iter().enumerate() {
            if !trajectory.budget.check(cancel)? {
                return Ok(trajectory);
            }
            for &(j, _) in neighbors {
                // With positive weights, equality along every edge is an
                // exact stationarity test, without subtractive cancellation.
                trajectory.has_dynamics |= v[i] != v[j];
            }
        }
        let error = SpectralError {
            roundoff_l2_estimate: gamma(seeds.len().saturating_mul(2).saturating_add(8))
                * trajectory.seed_norm,
            ..SpectralError::default()
        };
        trajectory.h.values = v.clone();
        trajectory.y.values = v;
        trajectory.h.error = error.clone();
        trajectory.y.error = error;
        trajectory.buffers = SpectralBuffers {
            term: vec![0.0; n],
            next: vec![0.0; n],
            sum: vec![0.0; n],
        };
        Ok(trajectory)
    }

    pub fn heat(
        &self,
        seeds: &[(String, f64)],
        theta: f64,
        max_edge_visits: usize,
        cancel: &AtomicBool,
    ) -> Result<Heat> {
        if !theta.is_finite() || !(0.0..=10.0).contains(&theta) {
            return Err(Error::invalid("theta must be finite in 0..10"));
        }
        let mut v = vec![0.0; self.ids.len()];
        for (id, mass) in seeds {
            if !mass.is_finite() || *mass < 0.0 {
                return Err(Error::invalid("invalid seed mass"));
            }
            let i = self
                .ids
                .binary_search(id)
                .map_err(|_| Error::invalid("unknown seed"))?;
            v[i] += mass;
        }
        let mass: f64 = v.iter().sum();
        if !mass.is_finite() {
            return Err(Error::invalid("seed mass overflow"));
        }
        if mass > 0.0 {
            for x in &mut v {
                *x /= mass;
            }
        }
        let mut result = Heat {
            operator: "free_heat_uniformization_v1".into(),
            status: "complete".into(),
            scores: v.clone(),
            theta,
            max_degree: self.max_degree,
            iterations: 0,
            edge_visits: 0,
            truncation_tail_bound: 0.0,
            floating_point_error:
                "not_certified; f64 operations separate from analytic series-tail bound".into(),
        };
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::cancelled());
        }
        if theta == 0.0 || self.max_degree == 0.0 || mass == 0.0 {
            return Ok(result);
        }
        let mut coefficient = (-theta).exp();
        result.scores = v.iter().map(|x| coefficient * x).collect();
        // P = I-L/d_max is nonnegative and mass preserving. The next Poisson
        // coefficient bounds the remaining series by a geometric tail once
        // its ratios are <1. No clipping or renormalization hides truncation.
        for k in 0..128 {
            let next = coefficient * theta / (k + 1) as f64;
            let ratio = theta / (k + 2) as f64;
            let tail = if ratio < 1.0 {
                (next / (1.0 - ratio)).min(1.0)
            } else {
                1.0
            };
            result.truncation_tail_bound = tail;
            if tail <= 1e-8 {
                return Ok(result);
            }
            let visits: usize = self.adjacency.iter().map(Vec::len).sum();
            if result.edge_visits.saturating_add(visits) > max_edge_visits {
                result.status = "work_limit".into();
                return Ok(result);
            }
            let mut propagated = vec![0.0; v.len()];
            for (i, neighbors) in self.adjacency.iter().enumerate() {
                if cancel.load(Ordering::Relaxed) {
                    return Err(Error::cancelled());
                }
                propagated[i] += (1.0 - self.degree[i] / self.max_degree) * v[i];
                for &(j, w) in neighbors {
                    propagated[j] += w / self.max_degree * v[i];
                }
            }
            v = propagated;
            coefficient = next;
            for (out, x) in result.scores.iter_mut().zip(&v) {
                *out += coefficient * x;
            }
            result.iterations += 1;
            result.edge_visits += visits;
        }
        result.status = "iteration_limit".into();
        Ok(result)
    }

    pub fn bfs(
        &self,
        seeds: &[String],
        depth: usize,
        max_nodes: usize,
        max_edge_visits: usize,
        cancel: &AtomicBool,
    ) -> Result<BreadthFirst> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut result = BreadthFirst {
            nodes: vec![],
            complete: true,
            edge_visits: 0,
        };
        for seed in seeds {
            let i = self
                .ids
                .binary_search(seed)
                .map_err(|_| Error::invalid("unknown seed"))?;
            if seen.insert(i) {
                queue.push_back((i, 0));
            }
        }
        while let Some((i, d)) = queue.pop_front() {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::cancelled());
            }
            if result.nodes.len() >= max_nodes {
                result.complete = false;
                break;
            }
            result.nodes.push((i, d));
            if d == depth {
                continue;
            }
            for &(j, _) in &self.adjacency[i] {
                if result.edge_visits >= max_edge_visits {
                    result.complete = false;
                    return Ok(result);
                }
                result.edge_visits += 1;
                if seen.insert(j) {
                    queue.push_back((j, d + 1));
                }
            }
        }
        Ok(result)
    }
}

// Standard gamma_n error model. It deliberately remains an estimate: the
// implementation does not control the rounding mode or enclose libm::exp.
fn gamma(operations: usize) -> f64 {
    let error = operations as f64 * f64::EPSILON;
    if error < 0.5 {
        error / (1.0 - error)
    } else {
        f64::INFINITY
    }
}

struct SparsePropagation<'a> {
    graph: &'a Graph,
    config: &'a SpectralConfig,
    edge_count: usize,
    max_neighbors: usize,
    seed_norm: f64,
}

impl SparsePropagation<'_> {
    fn advance(
        &self,
        state: &mut SpectralState,
        target: f64,
        budget: &mut SpectralBudget,
        buffers: &mut SpectralBuffers,
        cancel: &AtomicBool,
    ) -> Result<bool> {
        let n = state.values.len();
        while state.theta < target {
            let next_time = if target - state.theta <= self.config.max_increment {
                target
            } else {
                state.theta + self.config.max_increment
            };
            // Derive the actual increment from the stored endpoints. Adding
            // a small decimal step repeatedly and then declaring that its
            // rounded sum is the exact elapsed time would hide time drift.
            let increment = next_time - state.theta;
            if increment <= 0.0 || next_time == state.theta {
                budget.status = "time_resolution_limit";
                return Ok(false);
            }
            if !budget.charge(n.saturating_mul(2), 0, 0, 1, cancel)? {
                return Ok(false);
            }
            buffers.term.copy_from_slice(&state.values);
            let mut coefficient = (-increment).exp();
            for (out, value) in buffers.sum.iter_mut().zip(&buffers.term) {
                *out = coefficient * value;
            }
            let mut term_roundoff = 0.0;
            let mut series_roundoff = (4.0 + 2.0 * increment) * f64::EPSILON * self.seed_norm;
            let operator_roundoff =
                gamma(self.max_neighbors.saturating_mul(2).saturating_add(12)) * self.seed_norm;
            let mut completed = false;
            for k in 0..128 {
                let next_coefficient = coefficient * increment / (k + 1) as f64;
                let ratio = increment / (k + 2) as f64;
                let tail = if ratio < 1.0 {
                    (next_coefficient / (1.0 - ratio)).min(1.0)
                } else {
                    1.0
                };
                if tail <= self.config.poisson_tail_tolerance {
                    if !budget.charge(n.saturating_mul(2), 0, 0, 0, cancel)? {
                        return Ok(false);
                    }
                    if buffers.sum.iter().any(|x| !x.is_finite()) {
                        budget.status = "numerical_limit";
                        return Ok(false);
                    }
                    state.values.copy_from_slice(&buffers.sum);
                    state.theta = next_time;
                    // Every exact-arithmetic partial heat operator is an L2
                    // contraction. Thus local absolute tails add; there is no
                    // intermediate mass normalization to conceal their loss.
                    state.error.truncation_l2 += tail * self.seed_norm;
                    state.error.roundoff_l2_estimate += series_roundoff;
                    if !state.error.truncation_l2.is_finite()
                        || !state.error.roundoff_l2_estimate.is_finite()
                    {
                        budget.status = "numerical_limit";
                        return Ok(false);
                    }
                    completed = true;
                    break;
                }
                // One zeroing pass, one source-node pass, one series pass.
                if !budget.charge(n.saturating_mul(3), self.edge_count, 1, 0, cancel)? {
                    return Ok(false);
                }
                buffers.next.fill(0.0);
                for (i, neighbors) in self.graph.adjacency.iter().enumerate() {
                    if i % 256 == 0 && !budget.check(cancel)? {
                        return Ok(false);
                    }
                    buffers.next[i] +=
                        (1.0 - self.graph.degree[i] / self.graph.max_degree) * buffers.term[i];
                    for (edge_index, &(j, weight)) in neighbors.iter().enumerate() {
                        if edge_index % 1024 == 0 && cancel.load(Ordering::Relaxed) {
                            return Err(Error::cancelled());
                        }
                        buffers.next[j] += weight / self.graph.max_degree * buffers.term[i];
                    }
                }
                std::mem::swap(&mut buffers.term, &mut buffers.next);
                coefficient = next_coefficient;
                for (out, value) in buffers.sum.iter_mut().zip(&buffers.term) {
                    *out += coefficient * value;
                }
                term_roundoff += operator_roundoff;
                series_roundoff += coefficient
                    * (term_roundoff + gamma((k + 1) * 4 + 8) * self.seed_norm)
                    + 2.0 * f64::EPSILON * self.seed_norm;
            }
            if !completed {
                budget.status = "series_limit";
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl SpectralTrajectory<'_> {
    /// `complete` means that every requested sample was completed so far,
    /// not that the caller found a useful scale or sufficient evidence.
    pub fn status(&self) -> &'static str {
        self.budget.status
    }

    pub fn work(&self) -> SpectralWork {
        let mut work = self.budget.work.clone();
        work.elapsed_ms = self
            .budget
            .start
            .elapsed()
            .as_millis()
            .min(u64::MAX as u128) as u64;
        work
    }

    pub fn sample(&mut self, theta: f64, cancel: &AtomicBool) -> Result<Option<SpectralSample>> {
        if !theta.is_finite() || theta < 0.0 {
            return Err(Error::invalid(
                "spectral theta must be finite and nonnegative",
            ));
        }
        if self.last_sample.is_some_and(|previous| theta <= previous) {
            return Err(Error::invalid("spectral scales must strictly increase"));
        }
        if !self.budget.check(cancel)? {
            return Ok(None);
        }
        if self.budget.work.samples >= self.budget.limits.max_samples {
            self.budget.status = "sample_limit";
            return Ok(None);
        }
        if self.has_dynamics {
            let solver = SparsePropagation {
                graph: self.graph,
                config: &self.config,
                edge_count: self.edge_count,
                max_neighbors: self.max_neighbors,
                seed_norm: self.seed_norm,
            };
            // Both states continue from their own previous times. In
            // particular, y never accidentally becomes h at the same theta.
            if !solver.advance(
                &mut self.y,
                theta / 2.0,
                &mut self.budget,
                &mut self.buffers,
                cancel,
            )? || !solver.advance(
                &mut self.h,
                theta,
                &mut self.budget,
                &mut self.buffers,
                cancel,
            )? {
                return Ok(None);
            }
        } else {
            self.h.theta = theta;
            self.y.theta = theta / 2.0;
        }
        let Some(mut sample) = self.observe(theta, cancel)? else {
            return Ok(None);
        };
        if !self.budget.check(cancel)? {
            return Ok(None);
        }
        self.last_sample = Some(theta);
        self.budget.work.samples += 1;
        sample.work = self.work();
        Ok(Some(sample))
    }

    fn observe(&mut self, theta: f64, cancel: &AtomicBool) -> Result<Option<SpectralSample>> {
        let n = self.y.values.len();
        // A*y: clear + node traversal; norm/dot pass; residual pass; two
        // returned copies. This work counts even when the observable is zero.
        if !self
            .budget
            .charge(n.saturating_mul(6), self.edge_count, 1, 0, cancel)?
        {
            return Ok(None);
        }
        self.buffers.next.fill(0.0);
        if self.graph.max_degree > 0.0 {
            for (i, neighbors) in self.graph.adjacency.iter().enumerate() {
                if i % 256 == 0 && !self.budget.check(cancel)? {
                    return Ok(None);
                }
                // Difference form avoids subtracting D*y - W*y after large
                // nearly equal sums. It is still the full graph's A=L/dmax.
                for (edge_index, &(j, weight)) in neighbors.iter().enumerate() {
                    if edge_index % 1024 == 0 && cancel.load(Ordering::Relaxed) {
                        return Err(Error::cancelled());
                    }
                    self.buffers.next[i] +=
                        weight / self.graph.max_degree * (self.y.values[i] - self.y.values[j]);
                }
            }
        }
        let mut squared_norm = 0.0;
        let mut moment = 0.0;
        let mut squared_operator_norm = 0.0;
        for (&y, &ay) in self.y.values.iter().zip(&self.buffers.next) {
            squared_norm += y * y;
            moment += y * ay;
            squared_operator_norm += ay * ay;
        }
        let mut sample = SpectralSample {
            theta,
            physical_time: (self.graph.max_degree > 0.0)
                .then(|| theta / self.graph.max_degree)
                .filter(|time| time.is_finite()),
            h: self.h.values.clone(),
            y: self.y.values.clone(),
            z_q: None,
            c_q: None,
            spectral_mean: None,
            quality: "numerically_indeterminate".into(),
            error: self.y.error.clone(),
            heat_error: self.h.error.clone(),
            z_q_interval_estimate: None,
            c_q_interval_estimate: None,
            work: SpectralWork::default(),
        };
        if self.seed_norm == 0.0 {
            sample.quality = "no_seed".into();
            return Ok(Some(sample));
        }
        if !squared_norm.is_finite()
            || !moment.is_finite()
            || !squared_operator_norm.is_finite()
            || squared_norm <= 0.0
        {
            return Ok(Some(sample));
        }
        let norm = squared_norm.sqrt();
        let mean = moment / squared_norm;
        let mut squared_residual = 0.0;
        for (&y, &ay) in self.y.values.iter().zip(&self.buffers.next) {
            squared_residual += (ay - mean * y).powi(2);
        }
        let residual = squared_residual.sqrt();
        let curvature = (theta * (residual / norm)).powi(2);
        let z_q = (norm / self.seed_norm).powi(2);
        if !mean.is_finite() || !curvature.is_finite() || !z_q.is_finite() {
            return Ok(Some(sample));
        }
        sample.z_q = Some(z_q);
        sample.c_q = Some(curvature);
        sample.spectral_mean = Some(mean);
        let arithmetic = gamma(n.saturating_mul(2).saturating_add(8));
        let norm_error = arithmetic * norm;
        let delta = sample.error.truncation_l2 + sample.error.roundoff_l2_estimate + norm_error;
        let lower_norm = (norm - delta).max(0.0);
        let upper_norm = norm + delta;
        let seed_norm_error = arithmetic * self.seed_norm;
        let seed_lower = self.seed_norm - seed_norm_error;
        if lower_norm <= 0.0 || seed_lower <= 0.0 || !upper_norm.is_finite() {
            return Ok(Some(sample));
        }
        sample.z_q_interval_estimate = Some(SpectralInterval {
            lower: (lower_norm / (self.seed_norm + seed_norm_error)).powi(2),
            upper: (upper_norm / seed_lower).powi(2),
        });
        let ay_roundoff =
            gamma(self.max_neighbors.saturating_mul(3).saturating_add(8)) * 4.0 * norm;
        let moment_error = 4.0 * norm * delta
            + 2.0 * delta * delta
            + norm * ay_roundoff
            + arithmetic * norm * squared_operator_norm.sqrt();
        let mean_lower = ((moment - moment_error) / upper_norm.powi(2)).max(0.0);
        let mean_upper = ((moment + moment_error) / lower_norm.powi(2)).min(2.0);
        if !mean_lower.is_finite() || !mean_upper.is_finite() || mean_lower > mean_upper {
            return Ok(Some(sample));
        }
        let mean_error = (mean - mean_lower).abs().max((mean_upper - mean).abs());
        // ||A||2 <= 2, mu in [0,2]. Perturbing A*y - mu*y gives
        // 4*delta + |delta_mu|*||y_hat|| plus the operator/residual model.
        let residual_error = 4.0 * delta
            + mean_error * norm
            + ay_roundoff
            + 3.0 * f64::EPSILON * (squared_operator_norm.sqrt() + mean.abs() * norm);
        let lower = (theta * ((residual - residual_error).max(0.0) / upper_norm)).powi(2);
        let upper = (theta * ((residual + residual_error) / lower_norm)).powi(2);
        if !lower.is_finite() || !upper.is_finite() {
            return Ok(Some(sample));
        }
        sample.c_q_interval_estimate = Some(SpectralInterval { lower, upper });
        sample.quality = if !self.has_dynamics {
            "no_dynamics"
        } else if residual <= residual_error
            || curvature < f64::MIN_POSITIVE
            || !(0.0..=2.0).contains(&mean)
        {
            // A rounded zero or a plateau below the resolution of the model
            // must never masquerade as an informative spectral window.
            "numerically_indeterminate"
        } else {
            "estimated"
        }
        .into();
        Ok(Some(sample))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn flag() -> AtomicBool {
        AtomicBool::new(false)
    }
    #[test]
    fn two_nodes_closed_form_and_mass() {
        let g = Graph::new(vec!["a".into(), "b".into()], &[("a".into(), "b".into())]).unwrap();
        for theta in [0.0, 0.1, 1.0, 10.0] {
            let h = g.heat(&[("a".into(), 1.0)], theta, 10000, &flag()).unwrap();
            let exact = (1.0 + (-2.0 * theta).exp()) / 2.0;
            assert!((h.scores[0] - exact).abs() <= h.truncation_tail_bound + 1e-14);
            assert!((h.scores.iter().sum::<f64>() - 1.0).abs() <= h.truncation_tail_bound + 1e-14);
            assert!(h.scores.iter().all(|x| *x >= 0.0));
        }
    }
    #[test]
    fn isolated_disconnected_duplicate_and_limits() {
        let g = Graph::new(
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            &[
                ("a".into(), "b".into()),
                ("b".into(), "a".into()),
                ("c".into(), "c".into()),
            ],
        )
        .unwrap();
        let h = g.heat(&[("c".into(), 1.0)], 1.0, 1000, &flag()).unwrap();
        assert_eq!(h.scores[0], 0.0);
        assert_eq!(h.scores[3], 0.0);
        assert!((h.scores[2] - 1.0).abs() < 1e-8);
        assert_eq!(
            g.heat(&[("a".into(), 1.0)], 1.0, 0, &flag())
                .unwrap()
                .status,
            "work_limit"
        );
        assert!(g.heat(&[], f64::NAN, 100, &flag()).is_err());
        assert!(g.heat(&[], 1.0, 100, &AtomicBool::new(true)).is_err());
        assert_eq!(
            g.bfs(&["a".into()], 2, 100, 100, &flag())
                .unwrap()
                .nodes
                .len(),
            2
        );
    }
    #[test]
    fn energy_decreases_on_star() {
        let g = Graph::new(
            vec!["a".into(), "b".into(), "c".into()],
            &[("a".into(), "b".into()), ("a".into(), "c".into())],
        )
        .unwrap();
        let mut previous = f64::INFINITY;
        for theta in [0.0, 0.5, 1.0, 2.0, 10.0] {
            let h = g.heat(&[("a".into(), 1.0)], theta, 10000, &flag()).unwrap();
            let e = (h.scores[0] - h.scores[1]).powi(2) + (h.scores[0] - h.scores[2]).powi(2);
            assert!(e <= previous + 1e-14);
            previous = e;
        }
    }

    fn pair() -> Graph {
        Graph::new(vec!["a".into(), "b".into()], &[("a".into(), "b".into())]).unwrap()
    }

    fn sample_once(graph: &Graph, theta: f64, config: SpectralConfig) -> SpectralSample {
        graph
            .spectral_trajectory(
                &[("a".into(), 1.0)],
                config,
                SpectralLimits::default(),
                &flag(),
            )
            .unwrap()
            .sample(theta, &flag())
            .unwrap()
            .unwrap()
    }

    #[test]
    fn spectral_pair_uses_half_time_and_closed_form_curvature() {
        let graph = pair();
        let mut trajectory = graph
            .spectral_trajectory(
                &[("a".into(), 1.0)],
                SpectralConfig::default(),
                SpectralLimits::default(),
                &flag(),
            )
            .unwrap();
        for theta in [0.0, 1e-200, 0.1, 1.0, 2.0, 10.0, 100.0] {
            let sample = trajectory.sample(theta, &flag()).unwrap().unwrap();
            let expected_h = (1.0 + (-2.0 * theta).exp()) / 2.0;
            let expected_y = (1.0 + (-theta).exp()) / 2.0;
            let expected_z = (1.0 + (-2.0 * theta).exp()) / 2.0;
            let expected_c = (theta / theta.cosh()).powi(2);
            assert!((sample.h[0] - expected_h).abs() < 1e-11);
            assert!((sample.y[0] - expected_y).abs() < 1e-11);
            assert!((sample.z_q.unwrap() - expected_z).abs() < 1e-11);
            assert!((sample.c_q.unwrap() - expected_c).abs() < 1e-11);
            let interval = sample.c_q_interval_estimate.unwrap();
            assert!(expected_c >= interval.lower - 2e-14);
            assert!(expected_c <= interval.upper + 2e-14);
            assert!(!sample.error.roundoff_certified);
            if theta == 100.0 || theta == 1e-200 {
                assert_eq!(sample.quality, "numerically_indeterminate");
            }
        }
    }

    #[test]
    fn spectral_continuation_preserves_tail_loss_and_accumulates_work() {
        let graph = pair();
        let config = SpectralConfig {
            max_increment: 1.0,
            poisson_tail_tolerance: 0.05,
        };
        let first = sample_once(&graph, 1.0, config.clone());
        let later = sample_once(&graph, 4.0, config);
        let first_mass = first.h.iter().sum::<f64>();
        let later_mass = later.h.iter().sum::<f64>();
        assert!(first_mass < 0.999);
        assert!(later_mass < first_mass);
        assert!(later.heat_error.truncation_l2 > first.heat_error.truncation_l2);
        assert!(later.work.increments > first.work.increments);
        assert!(later.work.node_visits > first.work.node_visits);
        assert!((1.0 - later_mass) <= later.heat_error.truncation_l2 + 1e-14);
    }

    #[test]
    fn spectral_direct_and_composed_agree_with_accumulated_error() {
        let graph = Graph::new(
            vec!["a".into(), "b".into(), "c".into()],
            &[("a".into(), "b".into()), ("b".into(), "c".into())],
        )
        .unwrap();
        let direct = sample_once(&graph, 7.0, SpectralConfig::default());
        let mut composed = graph
            .spectral_trajectory(
                &[("a".into(), 1.0)],
                SpectralConfig {
                    max_increment: 0.75,
                    ..SpectralConfig::default()
                },
                SpectralLimits::default(),
                &flag(),
            )
            .unwrap();
        composed.sample(1.0, &flag()).unwrap().unwrap();
        let result = composed.sample(7.0, &flag()).unwrap().unwrap();
        let difference = direct
            .h
            .iter()
            .zip(&result.h)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        let allowance = direct.heat_error.truncation_l2
            + direct.heat_error.roundoff_l2_estimate
            + result.heat_error.truncation_l2
            + result.heat_error.roundoff_l2_estimate;
        assert!(difference <= allowance);
        assert!((direct.c_q.unwrap() - result.c_q.unwrap()).abs() < 1e-11);
    }

    #[test]
    fn spectral_limits_are_shared_by_all_scales_and_isolates() {
        let graph = pair();
        let mut trajectory = graph
            .spectral_trajectory(
                &[("a".into(), 1.0)],
                SpectralConfig::default(),
                SpectralLimits {
                    max_increments: 3,
                    ..SpectralLimits::default()
                },
                &flag(),
            )
            .unwrap();
        assert!(trajectory.sample(1.0, &flag()).unwrap().is_some());
        assert!(trajectory.sample(2.0, &flag()).unwrap().is_none());
        assert_eq!(trajectory.status(), "increment_limit");
        assert_eq!(trajectory.work().increments, 3);
        assert_eq!(trajectory.work().samples, 1);
        let applications = trajectory.work().operator_applications;
        assert!(trajectory.sample(4.0, &flag()).unwrap().is_none());
        assert_eq!(applications, trajectory.work().operator_applications);

        let isolates = Graph::new((0..1000).map(|i| i.to_string()).collect(), &[]).unwrap();
        let mut limited = isolates
            .spectral_trajectory(
                &[],
                SpectralConfig::default(),
                SpectralLimits {
                    max_node_visits: 100,
                    ..SpectralLimits::default()
                },
                &flag(),
            )
            .unwrap();
        assert_eq!(limited.status(), "node_limit");
        assert!(limited.sample(1.0, &flag()).unwrap().is_none());
        assert_eq!(limited.work().edge_visits, 0);
    }

    #[test]
    fn spectral_operator_edge_sample_and_time_limits_are_explicit() {
        let graph = pair();
        let cases = [
            (
                SpectralLimits {
                    max_operator_applications: 0,
                    ..SpectralLimits::default()
                },
                "operator_limit",
            ),
            (
                SpectralLimits {
                    // Initialization has two directed adjacency visits.
                    max_edge_visits: 2,
                    ..SpectralLimits::default()
                },
                "edge_limit",
            ),
            (
                SpectralLimits {
                    max_samples: 0,
                    ..SpectralLimits::default()
                },
                "sample_limit",
            ),
            (
                SpectralLimits {
                    max_elapsed_ms: 0,
                    ..SpectralLimits::default()
                },
                "time_limit",
            ),
        ];
        for (limits, reason) in cases {
            let mut trajectory = graph
                .spectral_trajectory(
                    &[("a".into(), 1.0)],
                    SpectralConfig::default(),
                    limits.clone(),
                    &flag(),
                )
                .unwrap();
            assert!(trajectory.sample(1.0, &flag()).unwrap().is_none());
            assert_eq!(trajectory.status(), reason);
            assert!(trajectory.work().edge_visits <= limits.max_edge_visits);
            assert!(trajectory.work().node_visits <= limits.max_node_visits);
            assert!(trajectory.work().operator_applications <= limits.max_operator_applications);
            assert_eq!(trajectory.work().samples, 0);
        }
    }

    #[test]
    fn spectral_stationary_empty_seed_and_no_edges_are_noninformative() {
        let graph = pair();
        for seeds in [vec![], vec![("a".into(), 1.0), ("b".into(), 1.0)]] {
            let mut trajectory = graph
                .spectral_trajectory(
                    &seeds,
                    SpectralConfig::default(),
                    SpectralLimits::default(),
                    &flag(),
                )
                .unwrap();
            let sample = trajectory.sample(1000.0, &flag()).unwrap().unwrap();
            assert_eq!(sample.work.increments, 0);
            if seeds.is_empty() {
                assert_eq!(sample.quality, "no_seed");
                assert!(sample.c_q.is_none());
            } else {
                assert_eq!(sample.quality, "no_dynamics");
                assert_eq!(sample.c_q, Some(0.0));
                assert_eq!(sample.z_q, Some(1.0));
            }
        }
        let isolated = Graph::new(vec!["a".into()], &[]).unwrap();
        let sample = sample_once(&isolated, 1000.0, SpectralConfig::default());
        assert_eq!(sample.quality, "no_dynamics");
        assert_eq!(sample.h, vec![1.0]);
        assert_eq!(sample.physical_time, None);
    }

    #[test]
    fn spectral_rescaling_and_unseeded_component_use_physical_time() {
        let first = pair();
        let second = Graph::weighted(
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
            &[
                ("a".into(), "b".into(), 1.0),
                ("c".into(), "d".into(), 10.0),
            ],
        )
        .unwrap();
        let base = sample_once(&first, 1.0, SpectralConfig::default());
        let disconnected = sample_once(&second, 10.0, SpectralConfig::default());
        assert_eq!(base.physical_time, disconnected.physical_time);
        assert!((base.h[0] - disconnected.h[0]).abs() < 1e-11);
        assert!((base.y[0] - disconnected.y[0]).abs() < 1e-11);
        assert!((base.c_q.unwrap() - disconnected.c_q.unwrap()).abs() < 1e-11);
        assert_eq!(disconnected.h[2..], [0.0, 0.0]);

        let scaled = Graph::weighted(
            vec!["a".into(), "b".into()],
            &[("a".into(), "b".into(), 7.0)],
        )
        .unwrap();
        let scaled_sample = sample_once(&scaled, 1.0, SpectralConfig::default());
        assert_eq!(base.h, scaled_sample.h);
        assert_eq!(base.c_q, scaled_sample.c_q);
        assert_eq!(scaled_sample.physical_time, Some(1.0 / 7.0));
    }

    #[test]
    fn spectral_validates_finiteness_order_overflow_and_cancellation() {
        let graph = pair();
        for seeds in [
            vec![("a".into(), f64::NAN)],
            vec![("a".into(), -1.0)],
            vec![("a".into(), f64::MAX), ("b".into(), f64::MAX)],
        ] {
            assert!(
                graph
                    .spectral_trajectory(
                        &seeds,
                        SpectralConfig::default(),
                        SpectralLimits::default(),
                        &flag(),
                    )
                    .is_err()
            );
        }
        let cancel = flag();
        let mut trajectory = graph
            .spectral_trajectory(
                &[("a".into(), 1.0)],
                SpectralConfig::default(),
                SpectralLimits::default(),
                &cancel,
            )
            .unwrap();
        assert!(trajectory.sample(f64::INFINITY, &cancel).is_err());
        trajectory.sample(1.0, &cancel).unwrap().unwrap();
        assert!(trajectory.sample(1.0, &cancel).is_err());
        cancel.store(true, Ordering::Relaxed);
        assert!(trajectory.sample(2.0, &cancel).is_err());
        assert!(
            graph
                .spectral_trajectory(
                    &[],
                    SpectralConfig::default(),
                    SpectralLimits::default(),
                    &cancel,
                )
                .is_err()
        );
    }
}
