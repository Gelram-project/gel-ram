# Check GEL RAM on your hardware

One command runs the short public checks on your machine and prints one block
of plain text. Paste that block into a reproduction issue: a result from a
machine other than the author's is the thing this project is missing most.
Slow machines, unusual systems and failures are wanted as much as fast passes.

## What you need

- Git and [rustup](https://rustup.rs); the checkout pins Rust 1.85.0.
- Linux, macOS or Windows. The two kill series need a Unix host; on Windows
  they are reported as SKIPPED with the reason.
- A network connection for the clone and `cargo fetch` only. The check itself
  runs offline.

## The command

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal
cargo fetch --locked
cargo run --locked --offline -p xtask -- witness ../gel-check
```

`../gel-check` must be a new directory outside the checkout; it receives the
block (in the file WITNESS.txt) and one log per step. The block described here comes from
the main branch after v0.8.0 and from the next release on; v0.8.0 prints a
shorter block, which is also welcome.

## How long it takes

On the author's machine (24 threads) the six steps take a little over a minute. The
first run also compiles the workspace, which takes longer than the checks on
most machines. The block gives the wall time of every step; compiling counts in
the time of the step that needed it.

## What the block holds

- the tool version and the git revision of your checkout, and whether the
  working tree was clean;
- the operating system family and version, the kernel release, the CPU
  architecture, the CPU model string, the number of logical cores and the
  total RAM (`unknown` where the system does not tell without extra tools);
- the `rustc` version;
- for each step: PASS, FAIL or SKIPPED, the exit status, the wall time in
  seconds and the last result line the program printed;
- the number of replacements made by the privacy filter, the block's own
  SHA-256 and a final line `GEL_WITNESS=PASS|FAIL pass=… fail=… skipped=…`.

A failed step stays in the block with its result line, and the command ends
with a non-zero exit status. The steps themselves are listed in
[the reproduction guide](REPRODUCE.md#a-short-witness).

## What it does not hold

No host name, user name, home directory or any other absolute path, no IP or
MAC address and no serial number. The facts are read from system files such
as `/proc/cpuinfo` and `/proc/meminfo`; nothing else is run to collect them,
and nothing is sent anywhere: you decide what to paste. Before the block is
written, a filter replaces any such detail it finds, for example in the result
line of a failed step, by a label such as `<redacted-path>`, and counts the
replacements on the line `redactions=`. A block that still holds one after
filtering is refused and not written. The step logs in the output directory
are not filtered; read a log before you attach it.

## Report it

1. Open a [reproduction issue](https://github.com/Gelram-project/gel-ram/issues/new?template=reproduction.yml).
2. Say who ran the test and give the commit (the `revision=` line).
3. Paste the whole block, from `GEL_WITNESS_2` to the `GEL_WITNESS=` line, into
   the field for the hardware check block.
4. Add anything the block cannot know: a laptop on battery, a virtual machine,
   other work running at the same time, and how the first run went for you.

For the complete suite (the full verification gate, the mutation matrix and the
comparison with grep), see [the reproduction guide](REPRODUCE.md).
