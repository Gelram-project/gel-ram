# Try GEL RAM on your hardware

This guide is the shortest reproducible path through the current public core.
It uses public source, deterministic fixtures and caller-selected text. It does
not require the private application, a private knowledge bank or an external LLM.

## 0. Prepare a fresh checkout

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
git rev-parse HEAD
```

Keep the exact commit SHA with any result you share.

## 1. Run the complete public verification gate

```text
cargo run --locked --offline -p xtask -- verify
```

Expected final marker:

```text
GEL_VERIFY_ALL=PASS
```

A failed gate must be investigated before interpreting benchmark output.

## 2. Run GEL Live Lab on your own UTF-8 text

```text
cargo run --locked --offline --release -p gel-live-lab
```

Useful commands:

```text
open PATH
find PHRASE
match 1
save NEW_PATH
load SHA256 PATH
exit
```

The bundle is plaintext. Keep the displayed SHA-256 pin independently if you
want to verify a fresh-process reopen.

See [Live Lab](LIVE-LAB.md).

### 2a. Evidence Lab: several documents, citations and a restart

```text
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence -- --demo
cargo run --locked --offline --release -p gel-live-lab --bin gel-evidence
```

The first command runs a scripted demonstration and must end with
`GEL_EVIDENCE_DEMO=PASS`. The second is interactive: `add PATH` for each UTF-8
file, then `find PHRASE`, `proof 1`, `save NEW_PATH`, exit, and reopen with
`load SHA256 PATH`. `replace ID PATH` and `drop ID` invalidate earlier results.
This is phrase retrieval from plaintext sources, not a chatbot.
See [Evidence Lab](EVIDENCE-LAB.md) and the [public walkthrough film](../media/GEL-EVIDENCE-LAB-EN.mp4).

## 3. Reproduce exact document readout

```text
cargo run --locked --offline -p gel-source --example source_find -- "garbage collection"
```

Try `ownership` and a phrase that is absent. The example reports source pins,
exact quotations/ranges and modification rejection. This is bounded source
extraction, not semantic question answering.

See [Document Readout](DOCUMENT-READOUT.md).

## 4. Run the independent byte/numeric/ranking audit

Choose a new output directory:

```text
cargo run --locked --offline --release -p gel-cli --example data_integrity -- target/integrity-demo-01
```

Expected final marker:

```text
GEL_DATA_INTEGRITY_ALL=PASS
```

Encoded-byte recovery and numerical conversion error are reported separately.

## 5. Explore the public Q8 record

```text
cargo run --locked --offline --release -p gel-phase-quad --example quad_playground -- --interactive
```

Try:

```text
phase 128
mask 3
noise 100
view 3
show
quit
```

Four reversible coordinate views describe one stored carrier. They do not create
four independent memories.

## 6. Verify the Ocean Scale R3 research package

Linux only for the bundled Ocean research verifier:

```sh
printf '%s\n' 'b712a6c6c4afb241e02d560d673eafb6bfce78049b498a8478f785508b8dcf48  research/ocean-scale-r3.tar.gz' | sha256sum --check -
ocean_run="$(mktemp -d)"
tar -xzf research/ocean-scale-r3.tar.gz -C "$ocean_run"
package="$ocean_run/p2-m1-c1-review-r3"
rustc +1.85.0 --edition=2021 "$package/tools/verify_review.rs" -o "$ocean_run/verify-ocean"
"$ocean_run/verify-ocean" "$package" "$ocean_run/evidence"
```

This verifies the pinned source/evidence package and its small offline suite.
It does **not** allocate and rerun the full 10M campaign by default.

For the opt-in large campaigns and their RAM requirements, follow
[Ocean Scale](OCEAN-SCALE.md).

## 7. Share a useful independent result

Open:
https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml

Include:

- exact commit or release tag;
- exact command;
- CPU, RAM, OS and Rust version;
- requested/effective worker counts where relevant;
- every timing sample, not only the fastest;
- PASS/failure markers;
- regressions and slower results.

Remove usernames, private paths, credentials, private datasets and confidential
material before posting logs.

A benchmark/correctness report does **not** require a code contribution or CLA.
Code intended for merge follows [CONTRIBUTING](../CONTRIBUTING.md).
