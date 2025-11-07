use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use rush::cli::HoldRange;
use rush::config::{HoldDurations, InitiatorConfig, ReceiverConfig};
use rush::metrics::{InitiatorMetrics, ReceiverMetrics};
use rush::modes::{initiator, receiver};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

fn available_loopback_addr() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);
    addr
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn initiator_and_receiver_exchange_connections() {
    let bind_addr = available_loopback_addr();

    let receiver_cfg = ReceiverConfig {
        bind: bind_addr,
        metrics_interval: None,
    };
    let receiver_metrics = Arc::new(ReceiverMetrics::default());
    let receiver_shutdown = CancellationToken::new();

    let receiver_task = tokio::spawn({
        let cfg = receiver_cfg.clone();
        let shutdown = receiver_shutdown.clone();
        let metrics = receiver_metrics.clone();
        async move {
            let _ = receiver::run(cfg, shutdown, metrics).await;
        }
    });

    // Allow the receiver to bind before initiator starts connecting.
    sleep(Duration::from_millis(50)).await;

    let initiator_cfg = InitiatorConfig {
        target: bind_addr,
        rate_per_sec: 40.0,
        hold: HoldDurations::from(HoldRange {
            min_ms: 10,
            max_ms: 50,
        }),
        max_open: 16,
    };
    let initiator_metrics = Arc::new(InitiatorMetrics::default());
    let initiator_shutdown = CancellationToken::new();

    let initiator_task = tokio::spawn({
        let cfg = initiator_cfg.clone();
        let shutdown = initiator_shutdown.clone();
        let metrics = initiator_metrics.clone();
        async move {
            let _ = initiator::run(cfg, shutdown, metrics).await;
        }
    });

    // Let both modes run and churn connections for a short window.
    sleep(Duration::from_millis(400)).await;

    initiator_shutdown.cancel();
    receiver_shutdown.cancel();

    let _ = initiator_task.await;
    let _ = receiver_task.await;

    let initiator_snapshot = initiator_metrics.snapshot();
    let receiver_snapshot = receiver_metrics.snapshot();

    assert!(
        initiator_snapshot.succeeded > 0,
        "expected successful initiator connections"
    );
    assert!(
        receiver_snapshot.accepted > 0,
        "receiver should accept connections"
    );
    assert!(
        receiver_snapshot.accepted > 0,
        "receiver should accept connections"
    );
}
