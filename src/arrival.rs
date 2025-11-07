use std::time::Duration;

pub trait ArrivalGenerator: Send {
    fn next_delay(&mut self) -> Duration;
}

#[derive(Clone)]
pub struct FixedInterval {
    period: Duration,
}

impl FixedInterval {
    pub fn new(period: Duration) -> Self {
        Self { period }
    }
}

impl ArrivalGenerator for FixedInterval {
    fn next_delay(&mut self) -> Duration {
        self.period
    }
}
