use std::net::SocketAddr;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "Lightweight network stress tester")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Receiver(ReceiverArgs),
    Initiator(InitiatorArgs),
}

#[derive(Args, Debug)]
pub struct ReceiverArgs {
    #[arg(long, value_name = "addr:port", default_value = "0.0.0.0:9000")]
    pub bind: SocketAddr,
    #[arg(long, value_name = "millis")]
    pub print_metrics_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct InitiatorArgs {
    #[arg(long, value_name = "addr:port")]
    pub target: SocketAddr,
    #[arg(long, value_name = "per-sec", default_value = "10")]
    pub freq: f64,
    #[arg(long, value_name = "min..max", default_value = "1000..1000")]
    pub hold_ms: HoldRange,
    #[arg(long, value_name = "count", default_value = "200")]
    pub max_open: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoldRange {
    pub min_ms: u64,
    pub max_ms: u64,
}

impl FromStr for HoldRange {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        let (min, max) = if let Some((start, end)) = trimmed.split_once("..=") {
            (start, end)
        } else if let Some((start, end)) = trimmed.split_once("..") {
            (start, end)
        } else {
            return Err("expected range format like 100..2000".to_string());
        };

        let min_ms: u64 = min
            .parse()
            .map_err(|_| "failed to parse hold range minimum".to_string())?;
        let max_ms: u64 = max
            .parse()
            .map_err(|_| "failed to parse hold range maximum".to_string())?;

        if min_ms > max_ms {
            return Err("hold range minimum must be <= maximum".to_string());
        }

        Ok(HoldRange { min_ms, max_ms })
    }
}
