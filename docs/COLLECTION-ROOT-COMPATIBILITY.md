# Streaming collection root hashing

Collection mutations now feed the existing canonical serialization fields
directly into SHA256. The byte order, field widths, document order and root
meaning remain unchanged. Serialization for explicit save/export is unchanged.

This removes the temporary full-collection Vec used only to compute a mutation
root. It does **not** make root updates incremental: hashing remains O(total
serialized bytes). No end-to-end timing gain, peak-RSS reduction or WAVE
performance multiplier has been measured for this change.

A regression performs 240 add/replace/remove/reload steps, checking the streamed
root against SHA256 of the canonical serialized bytes after every step.
Existing transaction rejection, malformed-input, citation and snapshot tests
also pass locally. These tests do not simulate physical power failure.

The historical measured implementation is preserved in
[collection.measured.rs.txt](evidence-collection/collection.measured.rs.txt).
Historical campaign pins and results were not regenerated to hide this change.
