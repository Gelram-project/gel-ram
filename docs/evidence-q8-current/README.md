# Q8 current candidate evidence

Final local campaign, 2026-09-11. Logs contain numerical metadata and timings,
not private phase codes or plaintext knowledge. The external input is not shipped.
Failed development builds and the earlier exploratory campaign were kept privately;
these are the completed final-source measurement runs, not selected fastest trials.

- `v2-*` files: all 48 runs of the existing V2 benchmark.
- `evidence-*` files: all 24 runs with a packed canonical-single baseline.
- `generate-*` files: deterministic synthetic fixture generation.
- [Hardware before](hardware-before.txt) / [load after](hardware-after.txt).
- [Full summary](summary.txt), recomputable with [summarize.rs](summarize.rs).
- [Source fingerprints](MEASURED-SOURCE-SHA256SUMS.txt).

Build and run the synthetic campaign from the repository root on Linux;
the output directory must not already exist:

```text
cargo build --locked --offline --release -p gel-phase-quad --examples
rustc --edition=2021 docs/evidence-q8-current/reproduce.rs -o /tmp/gel-q8-reproduce
/tmp/gel-q8-reproduce target/release/examples /tmp/gel-q8-new-campaign
rustc --edition=2021 docs/evidence-q8-current/summarize.rs -o /tmp/gel-q8-summarize
/tmp/gel-q8-summarize /tmp/gel-q8-new-campaign
```

Use the project's pinned Rust 1.85.0 toolchain. An optional third argument to
the runner is a caller-owned Q8DEMO01 fixture. Without it, external rows are
absent by design: the shipped sample cannot stand in for the private real bank.
Output fixture files belong outside the checkout and public archive.

Every comparison failure or benchmark failure stops the runner. Reference
worker fallback prevents reporting that run as an ordinary full-worker result.
The runner does not isolate the host, authenticate its environment, or measure
power, thermal throttling, hardware refresh, peak RSS or semantic accuracy.
Sample timing ranges are not confidence intervals.
