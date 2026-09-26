# Public precision matrix: F32, F16 and affine Q1–Q16

A transparent public reference for numeric precision, **not the private GEL
codec**. The private Q2.5 mechanism is not implemented here and cannot be
expressed by this format: widths are whole bits 1–16. Timing is not measured;
the historical timed campaign remains [quantization_matrix](EVIDENCE-CAMPAIGN.md)
and its pinned source is unchanged.

```sh
cargo run --locked --offline --release -p gel-cli --example precision_matrix
cargo test --locked --offline -p gel-cli --example precision_matrix
```

## Format GPMX v1

All integers and floats are little-endian. Header, 16 bytes:
`"GPMX"` | version u16 = 1 | kind u8 | bits u8 | count u32 | reserved u32 = 0.

| kind | bits | Body | Metadata |
|---|---|---|---|
| 0 F32 | 32 | count × f32 | none |
| 1 F16 | 16 | count × IEEE binary16 | none |
| 2 affine | 1–16 | count/32 blocks × (lo f32, hi f32, 4·bits payload bytes) | 8 B per 32 values |

- **Bit packing:** value *i* of a block occupies bits [i·bits, (i+1)·bits) of an
  LSB-first bitstream. 32·bits bits are exactly 4·bits bytes: **padding is always 0**.
- **Rounding:** code = round_ties_even((x − lo)/(hi − lo)·(2^bits − 1)) in f64;
  value = lo + (hi − lo)·code/(2^bits − 1) in f64, then rounded to f32.
  A constant block (lo == hi) reconstructs lo exactly and must carry only zero
  codes; other codes are rejected as non-canonical. F16 uses IEEE 754
  round-to-nearest-even from f32.
- **Exceptional values:** NaN and ±Inf are rejected on encode and decode.
  F16 rejects values that round beyond ±65504. Sign of zero is exact in F32,
  F16 and all-(−0) affine blocks; it is not preserved in affine blocks that mix
  −0 and +0.
- **Limits:** 1 ≤ count ≤ 2^24; affine count is a multiple of 32; the body
  length must match the header exactly (no truncation, no trailing bytes).
- **Error budget per value:** affine (hi − lo)/(2·(2^bits − 1)) +
  max(|lo|, |hi|)·ε32; F16 half an F16 ulp of the value's binade (2^-25 in the
  subnormal range).

Affine storage costs **bits + 2 bits per value** (64 metadata bits per 32
values). Affine Q16 therefore uses 18 bits per value, more than F16.

## Independent checks (10 tests)

| Test | What it establishes |
|---|---|
| codes_known_by_construction_every_width | For Q1–Q16, blocks with lo = 0, hi = 2^bits − 1 and random integer codes: packed bytes equal a separate bool-per-bit packer; decode is bit-exact |
| halfway_values_round_to_even_code_every_width | Genuine ties (checked to occur in the encoder's f64 arithmetic) choose the even code at every width |
| chosen_code_is_a_nearest_level | On random data at three scales, no neighbouring code reconstructs closer, within f32 rounding |
| affine_error_budget_all_widths_all_datasets | The declared affine error budget holds for all 16 widths and 6 datasets |
| f16_exhaustive_roundtrip_ties_and_overflow | All 63 488 finite F16 codes roundtrip; every midpoint rounds to even and its f32 neighbours to the nearer code; overflow at 65520 |
| f16_matches_table_search_oracle | 40 000 values agree with a separate sorted-table nearest search |
| container_save_load_every_format | All 18 formats: write, sync, read back and decode bit-identically; existing files are not overwritten |
| malformed_containers_and_inputs_rejected | Magic, version, kind/bits, reserved, count, length, block range, non-canonical constant block, non-finite F16/F32 |
| sign_of_zero_boundaries | −0 behaviour as documented |
| historical_q1_q2_q4_q8_rows_reproduced_exactly | All 24 affine rows of the historical [quantization-matrix.txt](evidence-collection/quantization-matrix.txt) are reproduced: body size, RMSE, max error and bit-exact count |

Local mutation check (not part of CI): rounding ties away from zero, inverting
the F16 tie parity and reversing packed bit order each made 1–5 of these tests fail.

## Result (Linux x86_64, Rust 1.85.0, release)

Full output: [precision-matrix-r1.txt](evidence-precision/precision-matrix-r1.txt),
108 rows, 108 file roundtrips. RMSE per value, 32 768 values per dataset:

| dataset | F16 | Q4 | Q8 | Q12 | Q16 |
|---|---:|---:|---:|---:|---:|
| silence | 0 | 0 | 0 | 0 | 0 |
| constant | 0 | 0 | 0 | 0 | 0 |
| image_gradient | 1.063e-4 | 2.302e-3 | 1.354e-4 | 8.433e-6 | 5.249e-7 |
| audio_tones | 9.305e-5 | 1.278e-2 | 7.673e-4 | 4.705e-5 | 2.955e-6 |
| small_signal_outliers | 1.905e-7 | 1.719e-3 | 1.719e-3 | 1.719e-3 | 1.719e-3 |
| alternating_extremes | 0 | 0 | 0 | 0 | 0 |

- **small_signal_outliers loses the weak signal at every affine width, Q1 to
  Q16**: one 1000.0 value per block makes each step ≥ 0.015, larger than the
  whole ±0.0015 signal. F16 keeps it. More bits under a shared block range do
  not protect small values; a task error budget must decide the format.
- image_gradient is bit-exact at Q5, Q10 and Q15 because 31 divides
  2^bits − 1 for those widths and each block spans exactly 31 steps of 1/255.
  This is a property of the synthetic input, not a lossless claim.

## What this does not prove

Not the private GEL codec, the private Q2.5 mechanism, phase-Q8, semantic
accuracy, speed, compression of real media, or any "4×" capacity. A lossless
byte roundtrip of the container does not make lossy quantization reversible.
Source evidence must keep its original bytes. Output digits for datasets built
with f64 `sin` may differ in the last place on other platforms' math libraries.
