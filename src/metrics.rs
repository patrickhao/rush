use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct ReceiverMetrics {
    accepts: AtomicU64,
    disconnects: AtomicU64,
    errors: AtomicU64,
    active: AtomicU64,
}

impl ReceiverMetrics {
    pub fn record_accept(&self) {
        self.accepts.fetch_add(1, Ordering::Relaxed);
        self.active.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_disconnect(&self) {
        self.disconnects.fetch_add(1, Ordering::Relaxed);
        decrement(&self.active);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ReceiverSnapshot {
        ReceiverSnapshot {
            accepted: self.accepts.load(Ordering::Relaxed),
            closed: self.disconnects.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            active: self.active.load(Ordering::Relaxed),
        }
    }
}

pub struct ReceiverSnapshot {
    pub accepted: u64,
    pub closed: u64,
    pub errors: u64,
    pub active: u64,
}

impl fmt::Display for ReceiverSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "accepted={} closed={} active={} errors={}",
            self.accepted, self.closed, self.active, self.errors
        )
    }
}

#[derive(Default)]
pub struct InitiatorMetrics {
    attempted: AtomicU64,
    succeeded: AtomicU64,
    failed: AtomicU64,
    throttled: AtomicU64,
    active: AtomicU64,
    completed: AtomicU64,
}

impl InitiatorMetrics {
    pub fn record_attempt(&self) {
        self.attempted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_success(&self) {
        self.succeeded.fetch_add(1, Ordering::Relaxed);
        self.active.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_completion(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
        decrement(&self.active);
    }

    pub fn record_throttled(&self) {
        self.throttled.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> InitiatorSnapshot {
        InitiatorSnapshot {
            attempted: self.attempted.load(Ordering::Relaxed),
            succeeded: self.succeeded.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            throttled: self.throttled.load(Ordering::Relaxed),
            active: self.active.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
        }
    }
}

pub struct InitiatorSnapshot {
    pub attempted: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub throttled: u64,
    pub active: u64,
    pub completed: u64,
}

impl fmt::Display for InitiatorSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "attempted={} succeeded={} active={} failed={} throttled={} completed={}",
            self.attempted,
            self.succeeded,
            self.active,
            self.failed,
            self.throttled,
            self.completed
        )
    }
}

fn decrement(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        value.checked_sub(1)
    });
}
