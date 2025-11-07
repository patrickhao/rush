# Rush
Lightweight and fast network stress tester.

## Usage
Build and run with Cargo:

```
cargo run -- <command> [options]
```

Start a receiver that listens on a port and periodically prints metrics:

```
cargo run -- receiver --bind 0.0.0.0:9000 --print-metrics-ms 1000
```

Start an initiator that opens outbound connections at a fixed rate, keeps them open for a random dwell time, and then closes them:

```
cargo run -- initiator \
  --target 127.0.0.1:9000 \
  --freq 25 \
  --hold-ms 100..2000 \
  --max-open 512
```

See `docs/DESIGN.md` for the architecture and planned extensions.
