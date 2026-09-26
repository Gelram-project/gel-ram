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
  F16 uses IEEE 754 round-to-nearest-even from f32.
- **Canonical blocks:** a constant block (lo == hi) reconstructs lo exactly and
  must carry only zero codes. A non-constant block (lo < hi) must contain code 0
  and code 2^bits − 1: the encoder stores the block minimum as lo with code 0
  and the maximum as hi with code 2^bits − 1. Blocks breaking either rule are
  rejected as non-canonical, so a stored range that no code reaches is refused.
  The rule does not make the bytes unique: a zero endpoint of a non-constant
  block stored as −0 or +0, and codes whose levels round to the same f32,
  decode to identical values.
- **Exceptional values:** NaN and ±Inf are rejected on encode and decode.
  F16 rejects values that round beyond ±65504.
- **Sign of zero:** exact in F32, F16 and affine blocks whose 32 values are
  zeros of one sign. A block of only −0 and +0 is constant (lo == hi) and
  decodes every value with the sign of the stored lo; which zero the encoder's
  minimum returns for equal zeros is not specified by IEEE minNum. A
  non-constant block stores no sign per value, so a decoded zero has the sign
  of its f64 reconstruction, not of the input. −0 decodes as +0 whenever that
  reconstruction is exactly 0, always when −0 is the block minimum
  (−0 + 0 = +0). A reconstruction in [−2^-150, 0) decodes as −0 even for a +0
  input (lo = −2^-149, hi = 5·2^-149, Q3: −2^-149 + 6·2^-149/7).
- **Limits:** 1 ≤ count ≤ 2^24; affine count is a multiple of 32; the body
  length must match the header exactly (no truncation, no trailing bytes). The
  count is checked before the body length and before any allocation.
- **Error budget per value:** with M = max(|lo|, |hi|) and ε32 = 2^-23, affine
  (hi − lo)/(2·(2^bits − 1)) + max(M·ε32, 2^-149); F16 half an F16 ulp of the
  value's binade (2^-25 in the subnormal range). The first affine term is half
  a quantization step. The second covers rounding the f64 reconstruction to
  f32, at most half an f32 ulp, max(M·2^-24, 2^-150), plus the f64 rounding in
  choosing the code and reconstructing it, below 16·2^-53·M = 2^-49·M. Their
  sum stays under M·ε32 when M ≥ 2^-126 and under 2^-149 below it, where f32
  spacing is a fixed 2^-149. Without the 2^-149 floor the budget fails for
  subnormal blocks: lo = 0, hi = 5·2^-149, value 2^-149, Q2 decodes to
  2·2^-149, an error of 2^-149 against 0.83·2^-149.

Affine storage costs **bits + 2 bits per value** (64 metadata bits per 32
values). Affine Q16 therefore uses 18 bits per value, more than F16.

## Independent checks (12 tests)

| Test | What it establishes |
|---|---|
| codes_known_by_construction_every_width | For Q1–Q16, blocks with lo = 0, hi = 2^bits − 1 and random integer codes: packed bytes equal a separate bool-per-bit packer; decode is bit-exact |
| halfway_values_round_to_even_code_every_width | Genuine ties (checked to occur in the encoder's f64 arithmetic) choose the even code at every width |
| chosen_code_is_a_nearest_level | On random data at three scales, no neighbouring code reconstructs closer, within f32 rounding |
| affine_error_budget_all_widths_all_datasets | The declared affine error budget holds for all 16 widths on the 6 datasets plus subnormal blocks (every multiple of 2^-149 for spans below 200·2^-149) and mixed blocks up to 2^-126, 2^-125 and 1.0 with random signs. Not loose: a tie reaches the quantization term exactly at every width, some value exceeds it by 0.496 of the rounding term (at least 0.4 is required), and the budget without the 2^-149 floor fails |
| f16_exhaustive_roundtrip_ties_and_overflow | All 63 488 finite F16 codes roundtrip; every midpoint rounds to even and its f32 neighbours to the nearer code; overflow at 65520 |
| f16_matches_table_search_oracle | 40 000 values agree with a separate sorted-table nearest search |
| container_save_load_every_format | All 18 formats: write, sync, read back and decode bit-identically; existing files are not overwritten |
| malformed_containers_and_inputs_rejected | Magic, version, kind/bits, reserved, count 0, length, block range, non-canonical constant block, non-finite F16/F32 |
| sign_of_zero_boundaries | −0 kept in constant blocks and F16; lost in non-constant blocks at every width, as the minimum and inside the block; a +0 and a −0 input both decoding as −0; a block of only ±0 decoding as the stored lo |
| count_limit_checked_before_body | Header-only containers with count 2^24 + 32 and 2^32 − 32 (affine) and 2^24 + 1 (F32, F16) are refused as out of range; count 2^24 passes the limit and fails only on the missing body |
| non_constant_block_needs_bottom_and_top_codes | At every width a non-constant block without code 0, without code 2^bits − 1 or with neither is refused; a block with both decodes |
| historical_q1_q2_q4_q8_rows_reproduced_exactly | All 24 affine rows of the historical [quantization-matrix.txt](evidence-collection/quantization-matrix.txt) are reproduced: body size, RMSE, max error and bit-exact count |

Local mutation check (not part of CI): rounding ties away from zero made 1 of
these 12 tests fail, inverting the F16 tie parity 2, and reversing the packed
bit order in both encoder and decoder 3.

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
