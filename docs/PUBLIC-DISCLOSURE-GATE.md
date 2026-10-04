# Public disclosure gate

`cargo run --locked --offline -p xtask -- disclosure` reads every text file of
the tree (not `target` or `.git`; binary files are skipped) and fails on any of:

- **User directory paths**: a name after an absolute `/home/`, `/media/`,
  `/Users/`, `\Users\` or `/run/media/`, other than CI runner names and
  placeholders (`runner`, `runneradmin`, `private`, `user`, `username`,
  `example`). Relative paths such as `../media/gifs` are fine.
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

**What it does not do.** It holds no list of words: a list kept in a public file,
even as hashes, can be recovered by guessing, so wording is reviewed before
publication outside this repository. It does not read pull request or release
descriptions, commit messages, branch names, text inside images or films, or
compiled binaries. It is not a review of meaning: a description that gives away
a private mechanism in ordinary words passes.
