# Public disclosure gate

`cargo run --locked --offline -p xtask -- disclosure` reads every text file of
the tree (not `target` or `.git`; binary files are skipped) and fails on any of:

- **Private paths**: a private user name after `/home/`, `/media/`, `/Users/`,
  `\Users\` or `/run/media/`. Relative paths such as `../media/gifs` are fine.
- **Internal names**: words and two-word phrases of the separate private
  project. They are compared, case-insensitively and across punctuation, with
  a list of SHA-256 hashes in [the gate](../xtask/src/disclosure.rs); the plain
  list is kept by the owner outside the repository, so the gate itself names
  nothing. A finding is reported by an 8-character hash prefix.
- **Network addresses**: IPv4 addresses outside loopback and the documentation
  ranges 192.0.2.0/24, 198.51.100.0/24 and 203.0.113.0/24. One exception: in the
  question files of the answer-or-abstain sets, an address found only inside the
  last column is not a finding. That column quotes a Wikipedia passage byte for
  byte and `answer-bench check` verifies it by SHA-256; a race time written
  as four dot-separated numbers in a quoted results table has the shape of an
  address. Every other
  column and every other check applies to those files as usual.
- **Credentials**: strings shaped like GitHub, Groq, OpenAI-style, Slack or AWS
  keys, and private key blocks.
- **E-mail addresses** other than the published contacts
  (`gelram.licensing@gmail.com`, `noreply@anthropic.com`) and no-reply or
  example domains.

`xtask verify` runs the gate, so every pull request and release check runs it
too. Final marker: `PUBLIC_DISCLOSURE_GATE=PASS`.

**What it does not do.** It finds only what it is told to look for. It is not a
review of meaning: a description that gives away a private mechanism in
ordinary words passes. A change to the hash list changes the gate and goes
through review like any other code.
