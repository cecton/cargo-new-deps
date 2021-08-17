[![Latest Version](https://img.shields.io/crates/v/cargo-new-deps.svg)](https://crates.io/crates/cargo-new-deps)
![License](https://img.shields.io/crates/l/cargo-new-deps)

cargo-new-deps
==============

List the newly added dependencies and their features.

Example:

```
$ cargo new-deps
ansi_term pulled by: tracing-subscriber
async-stream pulled by: tokio
chrono pulled by: opentelemetry-otlp, tracing-bunyan-formatter, tracing-subscriber
crossbeam-channel pulled by: opentelemetry
fnv pulled by: opentelemetry
gethostname pulled by: tracing-bunyan-formatter
grpcio +openssl pulled by: opentelemetry-otlp
humantime-serde pulled by: configuration
log +std pulled by: tracing-log
matchers pulled by: tracing-subscriber
opentelemetry pulled by: configuration, execution, opentelemetry-otlp, server
opentelemetry-otlp pulled by: execution, server
prost pulled by: opentelemetry-otlp
prost-build pulled by: opentelemetry-otlp
protobuf pulled by: opentelemetry-otlp
serde_json +arbitrary_precision pulled by: tracing-bunyan-formatter
sharded-slab pulled by: tracing-subscriber
smallvec pulled by: tracing-subscriber
thread_local pulled by: tracing-subscriber
tokio-stream pulled by: opentelemetry, opentelemetry-otlp, tokio
tonic pulled by: opentelemetry-otlp
tonic-build pulled by: opentelemetry-otlp
tower pulled by: hyper
tracing-attributes pulled by: tracing
tracing-core pulled by: tracing-bunyan-formatter, tracing-log, tracing-subscriber, tracing-test
tracing-core +std pulled by: tracing
tracing-futures pulled by: tracing-subscriber
tracing-log pulled by: tracing-bunyan-formatter, tracing-subscriber, tracing-subscriber
tracing-serde pulled by: tracing-subscriber
tracing-subscriber pulled by: tracing-bunyan-formatter, tracing-test
tracing-test pulled by: execution, query-planner, server
tracing-test-macro pulled by: tracing-test
```

Installation
------------

```
cargo install cargo-new-deps
```
