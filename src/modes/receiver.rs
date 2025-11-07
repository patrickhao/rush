use std::sync::Arc;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use crate::config::ReceiverConfig;
use crate::metrics::ReceiverMetrics;

pub async fn run(
    config: ReceiverConfig,
    shutdown: CancellationToken,
    metrics: Arc<ReceiverMetrics>,
) -> Result<()> {
    let listener = TcpListener::bind(config.bind).await?;
    tracing::info!(bind = %config.bind, "receiver listening");

    if let Some(period) = config.metrics_interval {
        spawn_metrics_task(metrics.clone(), period, shutdown.clone());
    }

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("receiver shutting down");
                break;
            }
            conn = listener.accept() => {
                match conn {
                    Ok((stream, peer)) => {
                        metrics.record_accept();
                        let metrics = metrics.clone();
                        tokio::spawn(async move {
                            handle_connection(stream, peer, metrics).await;
                        });
                    }
                    Err(err) => {
                        metrics.record_error();
                        tracing::warn!(error = %err, "accept error");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    mut stream: TcpStream,
    peer: std::net::SocketAddr,
    metrics: Arc<ReceiverMetrics>,
) {
    let mut buf = [0_u8; 1024];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                metrics.record_disconnect();
                break;
            }
            Ok(_) => continue,
            Err(err) => {
                metrics.record_error();
                metrics.record_disconnect();
                tracing::debug!(peer = %peer, error = %err, "receiver read error");
                break;
            }
        }
    }
}

fn spawn_metrics_task(
    metrics: Arc<ReceiverMetrics>,
    period: Duration,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        let mut ticker = time::interval(period);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = ticker.tick() => {
                    let snapshot = metrics.snapshot();
                    tracing::info!(target: "rush::receiver", metrics = %snapshot, "receiver metrics");
                }
            }
        }
    });
}
