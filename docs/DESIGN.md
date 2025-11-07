# Rush Design Notes

## Overview
- Rust-based lightweight network stress tester to help experiment with connection churn and observe server behavior.
- Two operating modes: `receiver` (accepts inbound TCP connections) and `initiator` (opens outbound TCP connections at a fixed cadence, keeps them alive for a random dwell time, then closes them).
- Architecture favors composability so upcoming features—network quality simulation (loss, latency, jitter) and custom arrival processes—can slot in without rewriting the core.

## MVP Goals
1. Provide a single binary with a clean CLI to start either mode.
2. Initiator opens connections at a fixed frequency (per second) and closes each connection after a random dwell time sampled from a uniform range.
3. Receiver accepts connections, keeps them open until peers disconnect, and logs lightweight metrics (counts, error snapshots).
4. Keep implementation small, readable, and idiomatic; prefer async Rust (Tokio) to handle many concurrent sockets.

## Non-Goals (for now)
- Simulating packet loss, jitter, or bandwidth throttling.
- Modeling arbitrary connection-arrival distributions beyond a simple fixed rate.
- TLS/encryption or application-level payloads.
- Persistent storage of metrics.

## CLI & Configuration
```
rush receiver --bind 0.0.0.0:9000
rush initiator --target 10.0.0.5:9000 --freq 50 --hold-ms 100..2000 --max-open 200
```
- `receiver` options: `--bind <addr:port>`, `--print-metrics-ms <period>`.
- `initiator` options: `--target <addr:port>`, `--freq <connections/sec>`, `--hold-ms <min..max>`, `--max-open <cap to avoid unbounded load>`.
- Config parsing handled via `clap`, mapped into a small `Config` struct shared with runtime components.

## Runtime Layout
```
main.rs
 ├─ cli (argument parsing)
 ├─ config (validated settings)
 ├─ runtime
 │   ├─ metrics (atomic counters, histograms later)
 │   └─ net (traits/helpers for connection lifecycle)
 └─ modes
     ├─ initiator
     └─ receiver
```
- `runtime::Executor` selects the appropriate mode and wires shared pieces (metrics reporter, shutdown hooks).
- Tokio runtime drives async tasks, letting us reuse timers, TCP streams, and select! macros cleanly.

### Receiver Mode
- Listens on configured socket via `TcpListener`.
- Each accepted `TcpStream` is handed to a lightweight handler task that waits for EOF or error; no payload exchange yet.
- Metrics increment on accept, disconnect, and errors; periodic reporter prints to stdout.
- Design keeps accept loop narrow so future features (drop injections, latency filters) can wrap the accepted stream before handing it off.

### Initiator Mode
- `ConnectionScheduler` uses `tokio::time::interval` to tick at the requested frequency.
- Each tick spawns a `ConnectionWorker` (bounded by `max_open`) that:
  1. Connects to the target.
  2. Samples a dwell time via `rand` within `[hold_min, hold_max]`.
  3. Sleeps for that duration while keeping the socket open, then closes it.
- Scheduler tracks inflight count to throttle when exceeding `max_open`, ensuring the tool does not DOS the host unintentionally.
- Metrics capture connect attempts, successes, failures, and dwell durations (histogram later).

## Extensibility Hooks
- `net::Behavior` trait: wraps `TcpStream` operations (read, write, shutdown) so future behaviors (loss injection, artificial delays) can decorate the stream without touching mode logic.
- `arrival::Generator` trait: default implementation is `FixedInterval`, but we can later swap in Poisson, bursty traces, or file-driven schedules.
- Metrics module exposes a `Recorder` trait so exporters (stdout now, file/prometheus later) can coexist.
- By isolating these traits, MVP stays minimal yet affords future drop-in components.

## Coding Style Constraints
- Prefer small modules and free functions over deep trait hierarchies until needed.
- Follow Rust 2021 idioms: `?` for error propagation, `anyhow::Result` for binaries, `tracing` for structured logs (with sane defaults).
- Keep comments for intent or non-obvious decisions; avoid restating what the code already conveys.
- Enforce clear ownership boundaries: configs are immutable once constructed, metrics behind `Arc`.

## Open Questions / Next Steps
1. Finalize the exact metric set (counts only vs. histograms).
2. Decide whether to expose JSON metrics for machine parsing in addition to stdout.
3. Evaluate whether initiator should support multiple targets per run (round-robin list).
4. Add integration tests using loopback sockets to validate both modes once basics land.
