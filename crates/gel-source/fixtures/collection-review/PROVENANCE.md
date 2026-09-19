# Post-freeze assessment fixtures

Selected after freezing collection runtime commit
`676e751718881705a1538fb84a6ba2dd7f103677`. Runtime/search source was not tuned
for these questions. This is a developer-authored contract assessment, not a
blind independent evaluation, human review, semantic benchmark or new web crawl.

- ownership.txt: existing Rust Book excerpt, SHA256
  `5284e31747fcb796ef577c64343c36d1627ac44a16b683a4dc4e81fa520dec71`.
  Copyright (c) 2010 The Rust Project Developers, MIT, full notice in
  [LICENSE-ownership.txt](LICENSE-ownership.txt). Original revision and range:
  [provenance](../../../../docs/REAL-SOURCE-DEMO.md).
- source-builder.txt: frozen project documentation, SHA256
  `201db5a76fefbce5ed893410837ff784c4bcd5c21da966705f692a5c087b2282`.
- ocean-scale.txt: frozen project documentation, SHA256
  `5ded58dffff2adf67ae394d4e184785eb9dc0f84c02de85005a35b088afb840d`.
- unicode.txt: authored synthetic fixture, NOT a real-world document benchmark,
  SHA256 `3aec47fe721b6eae1e67411db1a0cc2f09f457613767b24ddf813166f30eebd2`.

Project-owned documents remain under the root license. The existing third-party
excerpt remains MIT, not relicensed. These files preserve source line breaks.
The 24 questions include expected failures to handle semantic paraphrases and
cross-line phrases. UNKNOWN in those cases is contract compliance, not successful
semantic understanding. Four Unicode cases are reported separately from real text.
