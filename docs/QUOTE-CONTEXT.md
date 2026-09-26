# Exact matches and bounded source context

Phrase matching still returns the original matching-line byte range.
Presentation now supplies a separate context range, with up to 512 source
bytes before and after that match. UTF-8 boundaries are preserved. Blank outer
padding may be omitted, but matched bytes are never trimmed or rewritten.

The context belongs to the same authenticated document. It is not assembled
from different documents and is not a generated paraphrase. Display escaping
does not change the underlying offsets.

The source_find example and Live Lab expose match/context ranges and omission
flags. Evidence Lab retains the original QUOTE and additionally shows CONTEXT.
The standalone Live Lab paginates the context; use its existing page controls.
Context generation/rendering is outside the reported search-only timing.

This fixes the demonstrated loss of a preceding negation in the ownership
fixture. A finite window cannot guarantee that every distant condition is
visible. The display therefore explicitly says bounded context and reports
omitted text; it does not claim to show a complete sentence or prove truth.

Tests cover preceding negations, conditions, units and headings, mixed line
endings, UTF-8 clipping, long text, invalid ranges, and source readout after
restart across storage-part boundaries. Existing snapshots and historical
measurement sources were not rewritten.
