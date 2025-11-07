use std::sync::Arc;

use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::config::{Config, Mode};
use crate::metrics::{InitiatorMetrics, ReceiverMetrics};
use crate::modes::{initiator, receiver};

pub async fn run(config: Config) -> Result<()> {
    let shutdown = CancellationToken::new();
    let signal_token = shutdown.clone();
    let ctrl_c_task = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal_token.cancel();
        }
    });

    let result = match config.into_mode() {
        Mode::Receiver(cfg) => {
            let metrics = Arc::new(ReceiverMetrics::default());
            receiver::run(cfg, shutdown, metrics).await
        }
        Mode::Initiator(cfg) => {
            let metrics = Arc::new(InitiatorMetrics::default());
            initiator::run(cfg, shutdown, metrics).await
        }
    };

    ctrl_c_task.abort();
    let _ = ctrl_c_task.await;

    result
}
