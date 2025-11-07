use std::time::Duration;

use anyhow::{Result, bail};
use rand::Rng;

use crate::cli::{Cli, Commands, HoldRange, InitiatorArgs, ReceiverArgs};

#[derive(Clone, Debug)]
pub struct Config {
    mode: Mode,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Receiver(ReceiverConfig),
    Initiator(InitiatorConfig),
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self> {
        let mode = match cli.command {
            Commands::Receiver(args) => Mode::Receiver(ReceiverConfig::try_from(args)?),
            Commands::Initiator(args) => Mode::Initiator(InitiatorConfig::try_from(args)?),
        };

        Ok(Self { mode })
    }

    pub fn into_mode(self) -> Mode {
        self.mode
    }
}

#[derive(Clone, Debug)]
pub struct ReceiverConfig {
    pub bind: std::net::SocketAddr,
    pub metrics_interval: Option<Duration>,
}

impl TryFrom<ReceiverArgs> for ReceiverConfig {
    type Error = anyhow::Error;

    fn try_from(args: ReceiverArgs) -> Result<Self> {
        let metrics_interval = args
            .print_metrics_ms
            .and_then(|ms| (ms > 0).then(|| Duration::from_millis(ms)));

        Ok(Self {
            bind: args.bind,
            metrics_interval,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InitiatorConfig {
    pub target: std::net::SocketAddr,
    pub rate_per_sec: f64,
    pub hold: HoldDurations,
    pub max_open: usize,
}

impl InitiatorConfig {
    pub fn tick_interval(&self) -> Duration {
        let seconds = (1.0 / self.rate_per_sec).max(1e-6);
        Duration::from_secs_f64(seconds)
    }
}

impl TryFrom<InitiatorArgs> for InitiatorConfig {
    type Error = anyhow::Error;

    fn try_from(args: InitiatorArgs) -> Result<Self> {
        if args.freq <= 0.0 || !args.freq.is_finite() {
            bail!("--freq must be a positive finite number");
        }

        if args.max_open == 0 {
            bail!("--max-open must be greater than zero");
        }

        Ok(Self {
            target: args.target,
            rate_per_sec: args.freq,
            hold: HoldDurations::from(args.hold_ms),
            max_open: args.max_open,
        })
    }
}

#[derive(Clone, Debug)]
pub struct HoldDurations {
    min_ms: u64,
    max_ms: u64,
}

impl HoldDurations {
    pub fn sample_duration<R: Rng + ?Sized>(&self, rng: &mut R) -> Duration {
        if self.min_ms == self.max_ms {
            return Duration::from_millis(self.min_ms);
        }

        let ms = rng.gen_range(self.min_ms..=self.max_ms);
        Duration::from_millis(ms)
    }

    pub fn min_duration(&self) -> Duration {
        Duration::from_millis(self.min_ms)
    }

    pub fn max_duration(&self) -> Duration {
        Duration::from_millis(self.max_ms)
    }
}

impl From<HoldRange> for HoldDurations {
    fn from(range: HoldRange) -> Self {
        Self {
            min_ms: range.min_ms,
            max_ms: range.max_ms,
        }
    }
}
