# Public codec and correctness boundaries

Names used in different experiments do not imply interchangeable file formats.

| Public component | Contract | What it does not prove |
|---|---|---|
| Source bundles / collections | Exact UTF-8 bytes, pinned reopen, source offsets | Encryption, semantic truth, AI comprehension |
| Structural F2 codec | Exact reconstruction using binary predictors and residuals | F16 numeric precision or universal 4× compression |
| Affine minmax32 example (historical) | Q1, Q2, Q4, Q8 timed campaign, 32-value blocks, 8-byte endpoint metadata; pinned source | Widths other than 1/2/4/8 or the private Q2.5 mechanism |
| Precision matrix GPMX v1 | F32, IEEE F16 and affine Q1–Q16 with versioned container, oracles and file roundtrip; see [PRECISION-MATRIX.md](PRECISION-MATRIX.md) | Private GEL codec, Q2.5, speed or lossless quantization |
| F16 reference in data_integrity | Finite half-value roundtrips and rounding-boundary checks | Lossless conversion of arbitrary F32 |
| Symmetric Q8 numeric reference | Separate bounded-error experiment | Same format as phase-Q8 or affine minmax32 |
| Phase Q8 Quad | Four reversible coordinate views of one public record | Four independent datasets at unchanged capacity |

The public reference matrix covers F32, F16 and affine Q1–Q16 only. Q2.5 and
other private formats are not publicly implemented and require a separate
disclosure decision. Unsupported formats remain unsupported; adding a label or
a passing transport test does not implement a codec.

Before adding a format, specify byte order, bit packing, padding, metadata,
rounding, exceptional values, limits, version and error budget. Require an
independent oracle, boundary cases and save/load tests. Lossless source evidence
must retain its original bytes rather than be reconstructed from a lossy value.

## Four different correctness questions

1. **Bytes:** did transport/reconstruction return the exact stored bytes?
2. **Values:** how much numeric error did quantization introduce before storage?
3. **Ranking:** did retrieval return the oracle's required candidates/decision?
4. **Context:** did the displayed passage retain relevant qualifications, or
   clearly identify omissions?

A successful byte roundtrip does not answer the other three. UNKNOWN is a
boundary response, not automatic semantic success. The authored document
assessment is not a universal-language test.

The affine small_signal_outliers case explicitly exposes small-signal loss at
Q8 and, in the precision matrix, at every width through Q16 while F16 keeps it.
Keep that negative result when evaluating adaptive precision.
An adaptation succeeds only when it meets the declared task error budget, not
merely when its payload is smaller.

References: [quantization/collection protocol](EVIDENCE-CAMPAIGN.md),
[structural format](STRUCTURAL-CODEC.md), [data integrity](DATA-INTEGRITY.md),
[quote context](QUOTE-CONTEXT.md).
