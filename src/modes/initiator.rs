use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use rand::thread_rng;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tokio::time::{self, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use crate::config::{HoldDurations, InitiatorConfig};
use crate::metrics::InitiatorMetrics;

pub async fn run(
    config: InitiatorConfig,
    shutdown: CancellationToken,
    metrics: Arc<InitiatorMetrics>,
) -> Result<()> {
    let limiter = Arc::new(Semaphore::new(config.max_open));
    let mut ticker = time::interval(config.tick_interval());
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    spawn_metrics_reporter(metrics.clone(), shutdown.clone());

    tracing::info!(
        target = %config.target,
        rate_per_sec = config.rate_per_sec,
        hold_min_ms = config.hold.min_duration().as_millis(),
        hold_max_ms = config.hold.max_duration().as_millis(),
        max_open = config.max_open,
        "initiator running"
    );

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("initiator shutting down");
                break;
            }
            _ = ticker.tick() => {
                if try_spawn_worker(
                    config.target,
                    config.hold.clone(),
                    metrics.clone(),
                    limiter.clone()
                ).is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn try_spawn_worker(
    target: std::net::SocketAddr,
    hold: HoldDurations,
    metrics: Arc<InitiatorMetrics>,
    limiter: Arc<Semaphore>,
) -> Result<(), ()> {
    match limiter.clone().try_acquire_owned() {
        Ok(permit) => {
            tokio::spawn(async move {
                connection_worker(target, hold, metrics, permit).await;
            });
            Ok(())
        }
        Err(TryAcquireError::NoPermits) => {
            metrics.record_throttled();
            Ok(())
        }
        Err(TryAcquireError::Closed) => {
            tracing::warn!("connection limiter closed unexpectedly");
            Err(())
        }
    }
}

async fn connection_worker(
    target: std::net::SocketAddr,
    hold: HoldDurations,
    metrics: Arc<InitiatorMetrics>,
    permit: OwnedSemaphorePermit,
) {
    metrics.record_attempt();
    match TcpStream::connect(target).await {
        Ok(mut stream) => {
            metrics.record_success();
            let dwell = hold.sample_duration(&mut thread_rng());
            tracing::trace!(peer = %target, dwell_ms = dwell.as_millis(), "initiator connection established");
            time::sleep(dwell).await;
            if let Err(err) = stream.shutdown().await {
                tracing::debug!(peer = %target, error = %err, "shutdown error");
            }
            metrics.record_completion();
        }
        Err(err) => {
            metrics.record_failure();
            tracing::debug!(target = %target, error = %err, "connect error");
        }
    }

    drop(permit);
}

fn spawn_metrics_reporter(metrics: Arc<InitiatorMetrics>, shutdown: CancellationToken) {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(5));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = ticker.tick() => {
                    let snapshot = metrics.snapshot();
                    tracing::info!(target: "rush::initiator", metrics = %snapshot, "initiator metrics");
                }
            }
        }
    });
}
