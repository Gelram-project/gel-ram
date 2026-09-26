# Build a new collection without hashing every intermediate bank

`CollectionBuilder` stages a **new** source collection and computes its final
collection root at `build()`. The staging value exposes neither lookup nor a
root or snapshot, so an intermediate stale root cannot be used as evidence.
Each accepted document still receives its own hash, ID and revision increment.

```rust
use gel_source::collection::CollectionBuilder;
let mut builder = CollectionBuilder::new();
builder.add("Example", "Synthetic source bytes.").unwrap();
let collection = builder.build();
assert_eq!(collection.revision(), 1);
```

This is not a transaction on an existing collection. A failed `add` keeps
previous staged documents and does not consume an ID or revision. Discard the
builder if the entire import must be cancelled. The builder owns and copies its
input strings; it does not eliminate all copies or allocations.

For the same ordered accepted inputs, the resulting GELSET01 serialization,
IDs, revision and root are identical to sequential `Collection::add` calls.
No migration or change to historical pins is required. Ordinary live mutations
still refresh their root immediately; their cost is not made incremental.
Admission limits remain in force (including the 1024-document collection cap).
This public document collection is not the 10M numerical Ocean bank.

For N equal-sized documents, the previous new-bank construction hashes growing
prefixes after every insertion. Staging avoids those intermediate full-bank
hashes: it hashes each document and one final canonical bank (plus the tiny
initial empty root). This is a source-level work reduction, not a measured
claim of a particular speedup, physical RAM processing, or reduced RSS.

## Reproduction

```sh
cargo test --locked --offline -p gel-source --test collection_builder
cargo run --locked --offline --release -p gel-source --example collection_build_compare > new-build-raw.csv 2> new-build-log.txt
```

Use fresh output paths: shell redirection can overwrite an existing file.
The example emits 180 raw observations: sizes 8/64/256, 30 recorded pairs,
one unrecorded warm-up pair per size, alternating variant order. Input creation,
serialization, reopen and equality checks are outside the timer. The timer
includes constructing the bank, allocating owned strings and hashing. Each pair
must agree on exact bytes and root; the result is also decoded again.
The log pins the collection and harness source hashes. Record toolchain,
executable hash, hardware, load and execution command alongside outputs.

Both variant results coexist briefly; allocator/cache state is not isolated.
These are warm paired construction measurements, not peak memory measurements,
copy-volume accounting, search latency or evidence of tail guarantees.
N=30 makes empirical p99 the observed maximum. Keep all samples, not only wins.
The harness returns failure on any mismatch; `BUILD_COMPARE=PASS` means matching
results, not a promised performance improvement on every machine.
