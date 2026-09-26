# Reproduce collection and quantization evidence

```sh
cargo run --locked --offline --release -p gel-source --example collection_campaign -- ../new-collection-run
cargo run --locked --offline --release -p gel-source --example collection_recheck -- ../new-collection-run
cargo run --locked --offline --release -p gel-cli --example quantization_matrix
```

The collection campaign generates authored synthetic documents; it does not read
private files or the Internet. Sizes 8/64/256, three passes with reversed middle
order. 50 EXACT/NEAR/MISS/UNICODE samples per size/pass, plus 5 each of serialization,
deserialization, insert, update, delete, save and load: **2115 observations**.
The oracle checks exact generated identifiers and byte-equal reconstruction, not
semantic recall. Four query classes are not independent natural-language datasets.

Raw CSV, summary, corpus pins and environment remain in the new output directory.
The reducer checks the entire configuration/sample matrix and recomputes min,
p50/p95/p99/max and total. Percentiles are nearest-rank; N=5 or 50 within a group
is too small for strong tail claims (at N=50 p99 is the maximum). No p99.9 claim.
Raw times include slower samples. No best-run selection.

Search includes result allocation, not rendering. Update/insert/delete exclude
the preparatory clone but include root hashing/serialization. Save includes sync;
load is a warm-process/page-cache observation, not cold-boot latency. One thread.
No affinity, frequency lock, CPU isolation or other-host reproduction is claimed.
Linux VmRSS/VmHWM samples are process-wide; HWM is cumulative across cases, not
an independently reset bank peak. Input bytes and snapshot bytes exclude allocator,
BTreeMap and query/result overhead. Index overhead and cold start are not measured.

The independent reducer checks arithmetic/configuration, not the honesty of the
host clock or authenticity of a forged matching pair of raw data and summary.
Source pins and original retained logs supply provenance separately.

## Q reference is a different experiment

Q1/Q2/Q4/Q8 affine min/max uses 32 f32 samples per block, packed LSB first,
and two f32 endpoints (8 metadata bytes). Total block sizes 12/16/24/40 bytes,
not just 4/8/16/32 payload bytes. No private encoder or decoder is included.
Six synthetic sample families, 21 timings after warmup, full raw nanoseconds,
RMSE and maximum absolute error. Tiny signals beside outliers can be lost even
in Q8; loss is not necessarily noise. F16 is not implemented by this historical
Q matrix; see the separate data-integrity reference and the public
[precision matrix](PRECISION-MATRIX.md) (F32/F16/Q1–Q16, untimed).

No results here establish media perception, 4x independent capacity, conversation
quality or superiority to commercial AI. Existing Ocean runs are a different
numeric workload and must not be merged into this text-search timing table.
