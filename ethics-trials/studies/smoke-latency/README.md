# Smoke Latency

`smoke-latency` is a tiny low-token study package for scheduler, transport, and
parallelism testing.

It is not intended to answer substantive behavioral questions. Its purpose is
to let the runner exercise:

- rapid request launch
- paced backpressure
- per-model sequencing
- artifact writing
- SQLite indexing
- JSONL event streaming

with minimal prompt and completion cost.
