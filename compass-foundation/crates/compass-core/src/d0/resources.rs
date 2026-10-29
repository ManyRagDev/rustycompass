use super::types::{Error, Limits, Result};
use serde_json::{Value, json};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

#[cfg(test)]
type Hook<'a> = Box<dyn FnMut(&str) -> bool + 'a>;

pub(crate) struct Context<'a> {
    pub limits: Limits,
    pub cancel: &'a AtomicBool,
    start: Instant,
    pub structural: u128,
    pub peak: u128,
    pub reserved: u128,
    pub read: u128,
    pub cells: u128,
    #[cfg(test)]
    pub hook: Option<Hook<'a>>,
    #[cfg(test)]
    pub forced_expired: bool,
}
impl<'a> Context<'a> {
    pub fn new(limits: Limits, cancel: &'a AtomicBool) -> Self {
        Self {
            limits,
            cancel,
            start: Instant::now(),
            structural: 0,
            peak: 0,
            reserved: 0,
            read: 0,
            cells: 0,
            #[cfg(test)]
            hook: None,
            #[cfg(test)]
            forced_expired: false,
        }
    }
    pub fn check(&mut self, event: &str) -> Result<()> {
        #[cfg(test)]
        if let Some(hook) = self.hook.as_mut() {
            self.forced_expired |= hook(event);
        }
        #[cfg(not(test))]
        let _ = event;
        if self.cancel.load(Ordering::Relaxed) {
            return Err(Error("cancelled"));
        }
        #[cfg(test)]
        if self.forced_expired {
            return Err(Error("deadline_exceeded"));
        }
        if self
            .limits
            .duration
            .is_some_and(|n| self.start.elapsed().as_millis() >= n as u128)
        {
            return Err(Error("deadline_exceeded"));
        }
        Ok(())
    }
    pub fn add_structural(&mut self, n: u128) -> Result<()> {
        let total = self
            .structural
            .checked_add(n)
            .ok_or(Error("arithmetic_overflow"))?;
        if total > self.limits.get("max_logical_memory_bytes") as u128 {
            return Err(Error("input_limit"));
        }
        self.structural = total;
        self.peak = self.peak.max(total);
        Ok(())
    }
    pub fn memory(&mut self, n: u128) -> bool {
        let total = self.structural + n;
        if total > self.limits.get("max_logical_memory_bytes") as u128 {
            return false;
        }
        self.peak = self.peak.max(total);
        true
    }
    pub fn reserve_bytes(&mut self, size: u64) -> bool {
        let debit = size as u128 + 1;
        if self.reserved + debit > self.limits.get("max_total_object_bytes") as u128 {
            return false;
        }
        self.reserved += debit;
        true
    }
    pub fn reserve_cells(&mut self, n: u128) -> bool {
        if n > self.limits.get("max_cells_per_unit") as u128
            || self.cells + n > self.limits.get("max_cells_total") as u128
        {
            return false;
        }
        self.cells += n;
        true
    }
    pub fn value(&self) -> Value {
        json!({"logical_memory_peak":self.peak.to_string(),"object_bytes_read":self.read.to_string(),"object_bytes_reserved":self.reserved.to_string(),"cells_reserved":self.cells.to_string()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_memory_and_nonrefundable_reservations() {
        let flag = AtomicBool::new(false);
        let mut l = Limits::default();
        l.values.insert("max_logical_memory_bytes".into(), 149308);
        l.values.insert("max_cells_total".into(), 4);
        l.values.insert("max_total_object_bytes".into(), 3);
        let mut c = Context::new(l, &flag);
        c.add_structural(83680).unwrap();
        assert!(c.memory(65628));
        assert_eq!(c.peak, 149308);
        assert!(!c.memory(65629));
        assert_eq!(c.peak, 149308);
        assert!(c.reserve_cells(4));
        assert!(!c.reserve_cells(1));
        assert!(c.reserve_cells(0));
        assert_eq!(c.cells, 4);
        assert!(c.reserve_bytes(1));
        assert!(!c.reserve_bytes(1));
        assert!(c.reserve_bytes(0));
        assert_eq!(c.reserved, 3);
    }
    #[test]
    fn interrupt_precedes_deadline_and_pending_error() {
        let flag = AtomicBool::new(true);
        let mut c = Context::new(Limits::default(), &flag);
        c.forced_expired = true;
        assert_eq!(c.check("error").unwrap_err(), Error("cancelled"));
        flag.store(false, Ordering::Relaxed);
        assert_eq!(c.check("error").unwrap_err(), Error("deadline_exceeded"));
    }
}
