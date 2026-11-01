# License decision guide and `gel-license`

`gel-license` reads a distribution tree, or your own answers about one use of
GEL RAM, and prints which sentences of
[GEL RAM Noncommercial Reciprocal License 1.0](../LICENSE) its fixed rules
cite for those answers. It is offline and read-only.

**This guide and the program do not amend, add to or narrow the license.**
Under LICENSE section 17 the LICENSE is the complete public license grant; no
statement here grants or removes a right. This is not legal advice, and no
output of the program is a legal certification: every report ends with
`LEGAL_CERTIFICATION=NO`. Where this guide and the LICENSE differ, the LICENSE
controls. Questions about a particular use go to the Licensor (RR — GEL RAM
Project) at `gelram.licensing@gmail.com` (LICENSE section 19).

## Commands

```sh
gel-license check DIR             # a distribution tree you received or prepared
gel-license use --template        # the questions, with empty answers
gel-license use ANSWERS_FILE      # your answers, one verdict
gel-license --version             # one identity line
gel-license --license             # the identity line, the license lines and the third-party line
```

From a checkout: `cargo run --locked --offline -p gel-live-lab --bin gel-license -- check .`

| Exit | Meaning |
|---|---|
| 0 | `check`: `DISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT`; `use`: a verdict was printed; `--version`, `--license`. Exit 0 is not a compliance result. |
| 3 | `check`: `DISTRIBUTION_NOTICES=LISTED_ITEMS_MISSING` |
| 4 | `check`: `DISTRIBUTION_NOTICES=NO_GEL_NOTICES_FOUND` |
| 2 | refused (`CHECK=REFUSED reason=...`, `USE_VERDICT=REFUSED reason=...`) or invalid arguments |

## Checking a distribution

LICENSE section 8 says that a recipient of an unmodified copy receives the
license, all copyright notices and all plain-text lines beginning with
`Required Notice:` supplied by the Licensor; section 3 lets You share the
Software or a Modification for Noncommercial Purposes provided You preserve the
license and the notices required by section 8. `check DIR` reads, mechanically,
only some of those items: the LICENSE file, the `Required Notice:` line and the
GEL RAM source headers, and it lists the GEL RAM crate names it finds:

```text
LICENSE_FILE=EXACT
REQUIRED_NOTICE=KEPT
GEL_HEADER_FILES=0 HEADER_INTACT=0 HEADER_ALTERED=0
GEL_CRATES=gel-core,gel-source
SCAN_FILES=12 SKIPPED_SYMLINKS=0 SKIPPED_OVERSIZE=0
DISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT
NOTE=this reads only the items listed above; it does not show that every copyright and third-party notice was kept, nor that any use is licensed
CONTACT=gelram.licensing@gmail.com
LEGAL_CERTIFICATION=NO
```

| Line | What is read |
|---|---|
| `LICENSE_FILE` | the regular file `DIR/LICENSE`: `EXACT` when byte-equal to this project's LICENSE (the program carries its SHA-256, listed in [license pins](LICENSE-PINS.md)), `CHANGED` when it differs in any byte, `MISSING` when there is none. In a tree with no GEL RAM notice, `CHANGED` usually just means another project's license. |
| `LICENSE_DETAIL` | printed only as `LICENSE_DETAIL=line_endings`, after `LICENSE_FILE=CHANGED`, when the only difference is CR LF line endings (for example after a Windows checkout); such a copy is still `CHANGED` |
| `REQUIRED_NOTICE` | `KEPT` when `DIR/LICENSE` or `DIR/NOTICE` holds the line `Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)` as a whole line |
| `GEL_HEADER_FILES`, `HEADER_INTACT`, `HEADER_ALTERED` | `.rs` files one of whose first two lines, after an optional byte order mark, looks like a header line: a plain `//` comment whose text opens with `GEL RAM` or `Required Notice` (the rule of the source-header gate). Intact when the file starts with exactly the two header lines below, each ended by LF; a byte order mark or CR LF line endings make the header altered. |
| `GEL_CRATES` | the package and dependency names of this workspace's crates (`gel-bench`, `gel-cli`, `gel-core`, `gel-history`, `gel-kernel`, `gel-live-lab`, `gel-orb`, `gel-phase-quad`, `gel-physics`, `gel-reader`, `gel-source`, `gel-store`, `gel-structural`) declared in any Cargo.toml, read as text, including dotted keys such as `gel-core.workspace = true`, or `none`. Another crate whose name starts with `gel-` is not listed. |
| `SCAN_FILES`, `SKIPPED_SYMLINKS`, `SKIPPED_OVERSIZE` | regular files seen, symbolic links not followed, and Cargo.toml, LICENSE or NOTICE files larger than the read limit |
| `NOTE`, `CONTACT` | always the same two lines |

The source header of a GEL RAM-owned `.rs` file is:

```text
// GEL RAM — GEL RAM Noncommercial Reciprocal License 1.0; see LICENSE and NOTICE.
// Required Notice: Copyright (C) 2026 RR — GEL RAM Project (gelram.licensing@gmail.com)
```

The header carries the license name and the Required Notice and should stay
with the file.

The verdict, in this order:

| Found | `DISTRIBUTION_NOTICES` |
|---|---|
| none of: an exact LICENSE, a kept Required Notice, a header file, a crate of this workspace, a top-level LICENSE or NOTICE naming GEL RAM | `NO_GEL_NOTICES_FOUND` |
| `LICENSE_FILE=EXACT`, `REQUIRED_NOTICE=KEPT` and `HEADER_ALTERED=0` | `LISTED_ITEMS_PRESENT` |
| anything else | `LISTED_ITEMS_MISSING` |

`LISTED_ITEMS_MISSING` names listed items that were not found, or a GEL RAM
header that differs from the project's header; it is not a finding that the
license was breached. `LISTED_ITEMS_PRESENT` is not a finding that it was
followed: it does not show that every copyright notice of section 8, or every
third-party notice, was kept.

Bounds: symbolic links are not followed and `.git` and `target` directories are
skipped; at most 200,000 directory entries are read (a larger tree is refused,
check a smaller directory); at most 1 MiB of a Cargo.toml, LICENSE or NOTICE
is read, and only the first 4 KiB of a `.rs` file. A path that is not a
regular file is not opened.

Scope:

- `check` reads only the license, notice, header and crate-name items listed
  above; a file without a header is counted as a file without one;
- it does not read other copyright notices, third-party license texts (for
  example `crates/gel-source/LICENSE-UNICODE.txt`), media notices, whether a
  file is GEL RAM-owned or Third-Party Material (LICENSE section 2), or the
  purpose of a distribution;
- LICENSE and NOTICE are read at the top of `DIR` only: for a vendored copy
  inside another project, check the vendored directory too;
- it does not say whether the license calls for these items in a given tree,
  for example one that only declares a dependency on a crate of this workspace.

## Describing a use

`gel-license use --template` prints seven questions. Each quotes the
definitions of "Monetized Use", "Evaluation" and "Noncommercial Purpose" in
LICENSE section 2; answer each with `yes`, `no` or `unsure`:

```text
money=no
customer_production=no
hosted=no
paid_work=no
cost_reduction=no
evaluation_only=yes
noncommercial_purpose=yes
```

| Key | The question quotes |
|---|---|
| `money` | "use of the Software or a Modification, other than permitted Evaluation, that directly or indirectly receives, earns, enables, supports, protects, or is intended to produce money or other material financial benefit for You or another person", the examples "sale or licensing of a product; paid subscriptions;" and "sponsorship tied to access or operation", and the sentences on donations, tips, crowdfunding and sponsorships and on an unrelated personal gift |
| `customer_production` | "customer-facing production systems", "deployment in revenue-generating infrastructure" |
| `hosted` | "SaaS or hosted services", "advertising-supported services" |
| `paid_work` | "paid consulting or contract work"; "billable work" and "generation of deliverables for a paying customer", which section 2 excludes from Evaluation |
| `cost_reduction` | "production use that materially reduces the operating cost of a commercial activity" |
| `evaluation_only` | the definition of Evaluation and the uses it does not include |
| `noncommercial_purpose` | the definition of Noncommercial Purpose, "use that does not generate, enable, support, or form part of a Monetized Use", and the words "other than permitted Evaluation" of the definition of Monetized Use |

The first five keys are the items of Monetized Use. Every key must appear
exactly once with `yes`, `no` or `unsure`; a missing, unknown or repeated key,
or any other value, is refused. A leading byte order mark is ignored. The
verdict, in this order:

| Answers | `USE_VERDICT` | `LICENSE_SECTIONS` |
|---|---|---|
| 1. any of `money`, `customer_production`, `hosted`, `paid_work`, `cost_reduction` is `yes` | `COMMERCIAL_AGREEMENT_REQUIRED` | 2,5 |
| 2. otherwise any key is `unsure` | `ASK_THE_LICENSOR` | 2,19 |
| 3. otherwise `evaluation_only` is `yes` and `noncommercial_purpose` is `no` | `ASK_THE_LICENSOR` | 2,19 |
| 4. otherwise `noncommercial_purpose` is `no` | `COMMERCIAL_AGREEMENT_REQUIRED` | 2,5 |
| 5. otherwise `evaluation_only` is `yes` | `EVALUATION` | 2,4 |
| 6. otherwise (`noncommercial_purpose` is `yes`) | `NONCOMMERCIAL` | 2,3 |

Row 3: section 2 defines Monetized Use as use "other than permitted
Evaluation", so `evaluation_only=yes` together with `noncommercial_purpose=no`
conflict; the report adds a `NOTE=` line saying so. Row 4 follows the
definition: a use that is not a Noncommercial Purpose is one that generates,
enables, supports or forms part of a Monetized Use.

The report prints `LICENSE_QUOTE=` with the cited sentences word for word:
for `NONCOMMERCIAL`, the grant of section 3 and its sharing sentence, which
ends "provided You preserve this license and the notices required by section
8" (sentences that are not next to each other are joined by ` … `); for
`EVALUATION`, the last two paragraphs of section 4, including "the Software or
a Modification may not be deployed as part of a paid plan, customer-facing
production environment, revenue-generating service, or other Monetized Use
under this section"; for `COMMERCIAL_AGREEMENT_REQUIRED`, the first two
sentences of section 5; for `ASK_THE_LICENSOR`, the contact line of section
19. When an item of Monetized Use and `evaluation_only` are both `yes`, one
more line follows:
`NOTE=evaluation_only=yes and an item of Monetized Use answered yes: a Monetized Use is not Evaluation; section 2 defines Monetized Use as use "other than permitted Evaluation"`.
When `evaluation_only` is `yes`, `noncommercial_purpose` is `no` and no item of
Monetized Use is `yes`, this line follows instead:
`NOTE=evaluation_only=yes and noncommercial_purpose=no: section 2 defines Monetized Use as use "other than permitted Evaluation", so these answers conflict`.
Then come `LEGAL_CERTIFICATION=NO`, `CONTACT=gelram.licensing@gmail.com` and
`NOTE=answers are your own; this program reads the license, it does not interpret your facts`.
The program does not decide whether your facts make an answer `yes`; you do.
If you do not know, answer `unsure`.

## Worked cases

The rows below show which answers and which LICENSE sentences this guide's
rules pair with each case. They illustrate the program's rules; they are not
the Licensor's reading of the LICENSE, not a waiver and not additional terms.
Where the text does not settle the case, the table says so: ask the Licensor
before relying on any reading. "Five `no`" means the five items of Monetized
Use are answered `no`; an `unsure` answer prints `ASK_THE_LICENSOR`.

| Case | Answers | `use` prints | LICENSE |
|---|---|---|---|
| Personal study or a hobby project, with no money or financial benefit tied to it | five `no`, `evaluation_only=no`, `noncommercial_purpose=yes` | `NONCOMMERCIAL` | section 2 "Noncommercial Purpose"; section 3 (grant; "hobby work") |
| Classroom teaching with GEL RAM where no fee, sponsorship or other payment is tied to the GEL RAM teaching | five `no`, `evaluation_only=no`, `noncommercial_purpose=yes` | `NONCOMMERCIAL` | section 3 lists "teaching" among the ways of sharing for Noncommercial Purposes; section 2 "Noncommercial Purpose". Teaching in a fee-charging course or institution: **ask the Licensor** |
| A paid course or workshop about GEL RAM | **ask the Licensor**: section 2 lists "similar payments received specifically because of a GEL RAM-based activity" | depends on the `money` answer | section 2 "Monetized Use" |
| A company tests GEL RAM in a non-production environment to decide whether to obtain commercial rights | five `no`, `evaluation_only=yes`, `noncommercial_purpose=yes` | `EVALUATION` | section 2 "Evaluation" ("including a for-profit entity"); section 4 |
| That test starts serving real customers | `customer_production=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` (with the `NOTE` line if `evaluation_only=yes` too) | section 4 ("If Evaluation becomes a Monetized Use, commercial rights are required before that Monetized Use begins"); sections 2, 5 |
| An internal production tool at a company that materially reduces the operating cost of its commercial activity | `cost_reduction=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` | section 2 "Monetized Use"; section 5. Whether an effect is material: **ask the Licensor** |
| Internal production use at a company without a material effect on operating cost | **ask the Licensor** | depends on the answers; `unsure` prints `ASK_THE_LICENSOR` | section 2 (Evaluation does not include "production operation"; "Noncommercial Purpose" means "use that does not generate, enable, support, or form part of a Monetized Use") |
| Using GEL RAM to train or tune a production system | **ask the Licensor**, unless the system is part of a Monetized Use (then that item is `yes`) | depends on the answers; `unsure` prints `ASK_THE_LICENSOR` | section 2 (Evaluation does not include "use of GEL RAM to train or tune a production system") |
| GEL RAM in a paid, subscription, advertising-supported or otherwise revenue-linked SaaS or hosted service | `hosted=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` | section 2 ("SaaS or hosted services"; "paid subscriptions"; "advertising-supported services"); section 5 |
| A free, noncommercial public demo hosted by one person | **ask the Licensor**: "SaaS or hosted services" is listed as Monetized Use, and the definition speaks of money or other material financial benefit | depends on the `hosted` answer | section 2 |
| A consultant or freelancer billing a client for work that uses GEL RAM | `paid_work=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` | section 2 ("paid consulting or contract work"); section 5 ("regardless of whether You are an individual, freelancer, sole trader, nonprofit, company, or other organization") |
| A consultant billing a client for an Evaluation of GEL RAM for that client | `paid_work=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` (with the `NOTE` line if `evaluation_only=yes` too) | section 2 (Evaluation does not include "billable work"; "paid consulting or contract work"); section 5. The client's own unpaid Evaluation is a separate use |
| Grant-funded research using GEL RAM | **ask the Licensor**: the text does not settle whether a grant is a payment received specifically because of a GEL RAM-based activity | depends on the `money` answer | section 2 "Monetized Use" (donations, sponsorships "or similar payments"); section 3 |
| A free project sharing a modified GEL RAM that takes donations for that work | `money=yes` | `COMMERCIAL_AGREEMENT_REQUIRED` | section 2 ("Donations, tips, crowdfunding, sponsorships, or similar payments received specifically because of a GEL RAM-based activity are Monetized Use when they fund, reward, or support that activity"); section 5 |
| The same project without donations or other money | five `no`, `evaluation_only=no`, `noncommercial_purpose=yes` | `NONCOMMERCIAL` | section 3 (sharing a Modification, preserving the license and notices); section 8 |
| An unrelated personal gift to a person who also uses GEL RAM | for that gift: five `no`, `evaluation_only=no`, `noncommercial_purpose=yes` | `NONCOMMERCIAL` | section 2 ("An unrelated personal gift that is not connected to GEL RAM use is outside this definition") |
| A nonprofit's own production service that charges nothing | **ask the Licensor**: section 2 lists "SaaS or hosted services" and "customer-facing production systems" as Monetized Use, Evaluation does not include "production operation", and section 5 applies the same rule to a nonprofit; the text does not settle an unpaid production service | depends on the `hosted`, `customer_production` and `money` answers | sections 2, 5 |
| Sharing an unmodified copy | `check` the copy | `check`: `LISTED_ITEMS_PRESENT` when the exact license and the Required Notice line are found and no header is altered. Section 8 also names "all copyright notices", and THIRD-PARTY-NOTICES.md section 3 the third-party notices, which `check` does not read; `LISTED_ITEMS_PRESENT` is not a finding that section 8 is met | section 8 |
| Sharing a Modification | `check` the tree | `check` lists what it found | sections 3, 8 ("You must not represent the Modification as granting commercial rights that You do not have") |

A Commercial Agreement is a separate written agreement signed by the Licensor
(LICENSE section 2); the commercial options are described in
[COMMERCIAL-LICENSE.md](../COMMERCIAL-LICENSE.md), which is itself not a grant.

## Where the rules are

The rules of both commands are in one place,
`crates/gel-live-lab/src/license_check.rs`; the program only prints them.
`xtask verify` runs `gel-license check` on this repository and requires
`LICENSE_FILE=EXACT`, `REQUIRED_NOTICE=KEPT` and
`DISTRIBUTION_NOTICES=LISTED_ITEMS_PRESENT`, no altered header, and as many
intact headers as the source-header gate counts.
