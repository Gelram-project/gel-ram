# License texts in force

The names below refer only to these exact files in this tree. A file with the same
name and a different SHA-256 is not that license text. Earlier staged proposal
documents are not part of this tree; they were never an additional current grant.

| Name | File | SHA-256 |
|---|---|---|
| GEL RAM Noncommercial Reciprocal License 1.0 | [LICENSE](../LICENSE) | c0b560ffc53ad735c4a6236356ff0cd47d92cf671181e799917975b071e8cd40 |
| GEL RAM Contributor License Agreement 2.0 | [CLA.md](../CLA.md) | c8212f8637bff1a1a23c1c1c268f440af79f27fde10c40bf677211443edcda41 |
| CLA privacy notice | [CLA-PRIVACY.md](../CLA-PRIVACY.md) | c281415568a69cf93d396e8eaa9017ac844e5012cc224c739f7af2114f94c723 |
| Commercial licensing | [COMMERCIAL-LICENSE.md](../COMMERCIAL-LICENSE.md) | 5479686f66f248f12156958c2b30c30d95d9185b9fd30505bd9f7bb7a42afc13 |

`cargo test --locked --offline -p xtask --test license_pins` fails when any file
stops matching its line. A changed license text gets a new name or version and a
new line here; it is never edited under the same name.
