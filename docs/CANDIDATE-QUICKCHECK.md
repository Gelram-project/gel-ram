# Check the Evidence Lab PR candidate yourself

Run these commands from this candidate checkout, not from the older v0.3.0
tag. This guide does not say that the candidate has been published.
Rust 1.85.0 and cached locked dependencies are prerequisites. Initial toolchain
installation and `cargo fetch --locked` need internet; execution below is offline.
No model, private bank, paid service or remote API is needed.

## One command: correctness gate

```sh
cargo run --locked --offline -p xtask -- verify
```

Success requires exit code zero and `GEL_VERIFY_ALL=PASS`. Read failures rather
than treating an earlier PASS line as the final result. The gate includes
workspace tests, source demonstrations and the multi-document demo. It is not
a speed or semantic-accuracy benchmark. Linux is the locally tested platform;
candidate-specific Windows/macOS execution remains pending.

## Watch a small real run

```sh
cargo run --locked --offline -p gel-live-lab --bin gel-evidence -- --demo
```

Expected: two documents, original quotes, `CITATION=PASS`, `FIND=UNKNOWN` for
`invented answer`, deletion of a document, then `GEL_EVIDENCE_DEMO=PASS`.
This demo uses tiny built-in fixtures, not the Ocean or a language model.

## Reject changed bytes

```sh
cargo run --locked --offline -p gel-source --example source_readout -- "Demo vessel"
```

Expected final marker: `SOURCE_E2E=PASS`. This synthetic fixture includes a
modified-source rejection check. It does not establish the truth of a source.

## Try your own document and a restart

Start `gel-evidence` without `--demo`, then type:

```text
add PATH_TO_YOUR_UTF8_FILE
list
find PHRASE_FROM_YOUR_FILE
proof 1
save NEW_SNAPSHOT_PATH
exit
```

Keep the printed `BUNDLE_SHA256` independently. Start a new process and type:

```text
load SAVED_SHA256 SNAPSHOT_PATH
find PHRASE_FROM_YOUR_FILE
proof 1
exit
```

Replace placeholders with your own paths, hash and literal phrase. Use a new
snapshot path: existing files are not overwritten. Documents and snapshots
are **plaintext**, not encrypted. Do not submit private documents or generated
reports publicly. A process restart does not prove physical power-loss safety.

## Read the timing honestly

`search_ns` measures the indicated phrase-search operation, excluding terminal
rendering. It is neither token generation speed nor full-Ocean ORB/s. Tiny-demo
timings are not throughput measurements. Historical Ocean measurements and
candidate collection measurements remain separately labelled in their guides.

The [70-second film](../media/EVIDENCE-LAB-GUIDE.md) illustrates the candidate;
the two older films illustrate a separate private application. Source snapshots,
test results and media have separate scopes. Technical PASS grants no publication
or legal approval.
