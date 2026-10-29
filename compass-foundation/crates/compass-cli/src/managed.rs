//! CLI adapters for explicit managed collections. All behavior lives in core.
use clap::{Args, Subcommand};
use compass_core::{
    Error, Result, acquisition,
    collection::{Collection, PrepareRequest},
    retrieval::{self, RetrieveRequest},
    semantic, spectral_acquisition,
};
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::PathBuf, sync::atomic::AtomicBool};

#[derive(Subcommand)]
pub enum Managed {
    /// Prepare, update or inspect an explicit persistent collection.
    Collection {
        #[command(subcommand)]
        command: CollectionCommand,
    },
    /// Install the pinned local semantic model (explicit network operation).
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
    /// Find captured paths within one immutable generation.
    Files {
        #[command(flatten)]
        source: Source,
        #[arg(long, default_value = "")]
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Rank verified evidence with literal, lexical, semantic or hybrid retrieval.
    Query {
        #[command(flatten)]
        source: Source,
        #[arg(long, allow_hyphen_values = true)]
        query: String,
        #[arg(long,default_value="lexical",value_parser=["literal","lexical","semantic","hybrid"])]
        mode: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Search symbols or expand structural neighbors with evidence.
    Neighbors {
        #[command(flatten)]
        source: Source,
        #[arg(long, conflicts_with = "query")]
        node: Option<String>,
        #[arg(long, conflicts_with = "node")]
        query: Option<String>,
        #[arg(long, default_value_t = 1)]
        depth: usize,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Pack verified context under a byte budget with a pinned continuation cursor.
    Retrieve {
        #[command(flatten)]
        source: Source,
        #[arg(long, allow_hyphen_values = true)]
        query: String,
        #[arg(long,default_value="hybrid",value_parser=["literal","lexical","semantic","hybrid"])]
        mode: String,
        #[arg(long,default_value="none",value_parser=["none","bfs","heat"])]
        expansion: String,
        #[arg(long)]
        theta: Option<f64>,
        #[arg(long)]
        acquisition_policy: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        /// Compact query intent JSON (requires spectral_window_v1).
        #[arg(long)]
        query_intent: Option<PathBuf>,
        #[arg(long, default_value_t = 16384)]
        budget_bytes: usize,
        #[arg(long)]
        cursor: Option<String>,
        /// Acquire explicit evidence obligations from a bounded JSON contract.
        #[arg(long)]
        contract: Option<PathBuf>,
    },
}
#[derive(Args)]
pub struct Source {
    #[arg(long)]
    store: PathBuf,
    #[arg(long)]
    generation: Option<String>,
}
#[derive(Subcommand)]
pub enum CollectionCommand {
    /// Capture safe bytes and build persistent local indexes.
    Prepare {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        store: PathBuf,
        #[arg(long,default_value_t=4*1024*1024*1024u64)]
        max_disk_bytes: u64,
        #[arg(long)]
        semantic_model: Option<PathBuf>,
    },
    /// Reconcile source changes while preserving earlier generations.
    Update {
        #[arg(long)]
        store: PathBuf,
    },
    /// Report current generation, coverage and resource usage.
    Status {
        #[arg(long)]
        store: PathBuf,
    },
}
#[derive(Subcommand)]
pub enum ModelCommand {
    Install {
        #[arg(long)]
        model_dir: PathBuf,
    },
}
fn open(source: &Source, offset: usize) -> Result<Collection> {
    if offset > 0 && source.generation.is_none() {
        return Err(Error::invalid(
            "--offset greater than zero requires --generation to pin pagination",
        ));
    }
    Collection::open(&source.store, source.generation.as_deref())
}
pub fn run(command: &Managed, cancel: &AtomicBool) -> Result<(Value, i32)> {
    let value = match command {
        Managed::Collection { command } => match command {
            CollectionCommand::Prepare {
                root,
                store,
                max_disk_bytes,
                semantic_model,
            } => retrieval::prepare(
                &PrepareRequest {
                    root: root.clone(),
                    store: store.clone(),
                    max_disk_bytes: *max_disk_bytes,
                },
                semantic_model.as_deref(),
                cancel,
            )?,
            CollectionCommand::Update { store } => retrieval::update(store, cancel)?,
            CollectionCommand::Status { store } => retrieval::status(store)?,
        },
        Managed::Model {
            command: ModelCommand::Install { model_dir },
        } => json!({"model":semantic::install(model_dir,cancel)?,"model_dir":model_dir}),
        Managed::Files {
            source,
            query,
            limit,
            offset,
        } => open(source, *offset)?.files(query, *limit, *offset)?,
        Managed::Query {
            source,
            query,
            mode,
            limit,
            offset,
        } => retrieval::query(
            &open(source, *offset)?,
            query,
            mode,
            *limit,
            *offset,
            cancel,
        )?,
        Managed::Neighbors {
            source,
            node,
            query,
            depth,
            limit,
        } => retrieval::neighbors(
            &open(source, 0)?,
            node.as_deref(),
            query.as_deref(),
            *depth,
            *limit,
            cancel,
        )?,
        Managed::Retrieve {
            source,
            query,
            mode,
            expansion,
            theta,
            budget_bytes,
            cursor,
            contract,
            acquisition_policy,
            scope,
            query_intent,
        } => {
            spectral_acquisition::validate_options(
                acquisition_policy.as_deref(),
                scope.as_deref(),
                contract.is_some(),
                cursor.is_some(),
                *theta,
                expansion,
            )?;
            if query_intent.is_some() && acquisition_policy.is_none() {
                return Err(Error::invalid("query_intent requires spectral_window_v1"));
            }
            let intent: Option<spectral_acquisition::QueryIntent> = query_intent
                .as_ref()
                .map(|path| -> Result<_> {
                    let mut raw = Vec::new();
                    File::open(path)
                        .and_then(|file| file.take(16385).read_to_end(&mut raw))
                        .map_err(|e| Error::invalid(e.to_string()))?;
                    if raw.len() > 16384 {
                        return Err(Error::invalid("query_intent too large"));
                    }
                    serde_json::from_slice(&raw).map_err(|e| Error::invalid(e.to_string()))
                })
                .transpose()?;
            let contract = contract
                .as_ref()
                .map(|path| -> Result<Value> {
                    let mut raw = Vec::new();
                    File::open(path)
                        .and_then(|file| file.take(65537).read_to_end(&mut raw))
                        .map_err(|error| Error::io("cannot read contract file", error))?;
                    if raw.len() > 65536 {
                        return Err(Error::invalid("contract file exceeds 64 KiB input limit"));
                    }
                    serde_json::from_slice(&raw)
                        .map_err(|_| Error::invalid("invalid contract JSON"))
                })
                .transpose()?;
            let cursor_generation = cursor
                .as_deref()
                .map(|cursor| {
                    if contract.is_some() {
                        acquisition::cursor_generation(cursor)
                    } else {
                        retrieval::cursor_generation(cursor)
                    }
                })
                .transpose()?;
            if let (Some(supplied), Some(pinned)) = (&source.generation, &cursor_generation)
                && supplied != pinned
            {
                return Err(Error::invalid(
                    "--generation differs from cursor generation",
                ));
            }
            let generation = cursor_generation
                .as_deref()
                .or(source.generation.as_deref());
            let explicit_generation = source.generation.is_some();
            if let Some(contract) = &contract {
                acquisition::validate_contract(contract, explicit_generation)?;
            }
            let collection = Collection::open(&source.store, generation)?;
            let request = RetrieveRequest {
                query: query.clone(),
                mode: mode.clone(),
                expansion: expansion.clone(),
                theta: theta.unwrap_or(1.0),
                budget_bytes: *budget_bytes,
                cursor: cursor.clone(),
            };
            if acquisition_policy.is_some() {
                spectral_acquisition::retrieve_with_intent(
                    &collection,
                    &spectral_acquisition::Request {
                        query: query.clone(),
                        mode: mode.clone(),
                        scope: scope.clone().unwrap(),
                        budget_bytes: *budget_bytes,
                    },
                    intent.as_ref(),
                    cancel,
                )?
            } else if let Some(contract) = &contract {
                acquisition::retrieve(&collection, &request, contract, explicit_generation, cancel)?
            } else {
                retrieval::retrieve(&collection, &request, cancel)?
            }
        }
    };
    let code = if value["quality"] == "partial" { 3 } else { 0 };
    Ok((value, code))
}
