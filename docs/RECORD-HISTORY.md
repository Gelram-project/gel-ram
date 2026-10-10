# Record history (`gel-history`)

```sh
cargo test --locked --offline -p gel-history
cargo run --locked --offline --release -p gel-history --example record_history
```

`gel-history` keeps every state appended to one 128-byte ORB128 record
([format](FORMAT.md)). Each appended state is stored as a literal copy or as
the XOR residual from the state appended just before it, using the sparse
residual of the [structural codec](STRUCTURAL-CODEC.md). Any stored state is
rebuilt from one literal and at most two residuals (`MAX_RESIDUAL_DEPTH`); the
rebuilt bytes equal the appended bytes bit for bit. A call returns the exact
state at an index, or an error. Nothing is searched, ranked or answered.
Storage grows with every appended state.

## What is stored

| Entry | Bytes | Stored when |
|---|---|---|
| Literal | 129: tag 0 and the 128-byte state | the first state; the entry before it is already two residuals from its literal; or a residual would not be smaller |
| Residual | 13 + ceil(10·k/8): tag 1, parent index (4), depth (1), form 0 (1), count (2), length (4) and k packed 10-bit positions | k ≤ 92 bits differ from the state before, and the entry before it is fewer than two residuals from its literal |

A GELHIS01 file is a 48-byte header followed by the entries. All integers are
little-endian.

| Offset | Bytes | Field |
|---:|---:|---|
| 0 | 8 | magic `GELHIS01` |
| 8 | 4 | version = 1 |
| 12 | 4 | reserved = 0 |
| 16 | 8 | entry count |
| 24 | 8 | payload length |
| 32 | 8 | payload CRC64-ECMA |
| 40 | 8 | CRC64-ECMA of bytes 0 to 39 |

Format facts, frozen by tests:

- 92 changed bits make a 128-byte residual; 93 or more are stored as a
  129-byte literal.
- At most two residuals follow a literal, so at least one state in three is
  stored in full. Three unchanged states take 129 + 13 + 13 = 155 bytes, where
  three full copies take 384; no history stores less than that per three states,
  header excluded.
- Unrelated states are all literals: 129 bytes for every 128-byte state.
- The parent of a residual is always the entry before it.
- One history has one file. The decoder accepts only the bytes the encoder
  writes for the same states: it refuses a literal where a residual fits, a
  residual larger than a literal, a parent other than the entry before, a dense
  residual and a depth byte that does not continue its chain.

## Bytes on synthetic walks

3,000 states; each differs from the one before in exactly k bits chosen by
splitmix64, and the last row uses unrelated states. Stored bytes include the
48-byte header; 3,000 full copies take 384,000 bytes. The example checks every
row against the count above, rebuilds every state before and after reopening the
bytes, and exits nonzero on any difference. Synthetic walks show the format, not
how real records change.

| k | literals | residuals | stored bytes |
|---:|---:|---:|---:|
| 0 | 1,000 | 2,000 | 155,048 |
| 1 | 1,000 | 2,000 | 159,048 |
| 8 | 1,000 | 2,000 | 175,048 |
| 32 | 1,000 | 2,000 | 235,048 |
| 64 | 1,000 | 2,000 | 315,048 |
| 92 | 1,000 | 2,000 | 385,048 |
| 93 | 3,000 | 0 | 387,048 |
| unrelated | 3,000 | 0 | 387,048 |

## Reading back

`reconstruct(i)` walks from entry i to its literal in at most two hops and
applies the residuals in order. The decoder counts the depth of every residual
from the chain the earlier entries form; a parent that does not precede its
child, a depth byte that does not continue its parent's depth, or a chain longer
than two is an error, whatever the depth byte says.

## Saving and reopening

`write_atomic` writes the whole history to a sibling temporary file
`<path>.tmp-<pid>-<n>`, created exclusively (a name that already exists is
skipped), flushes it, renames it over the target and, on Unix, flushes the
directory. On Unix a new file has mode 0600 and a replaced file keeps its mode.
An error from that final directory flush means the file was already replaced.

`from_bytes` and `open_verified` check the caller's entry and payload limits
before allocating, then refuse a wrong magic, version or reserved field, either
CRC64 mismatch, a length that differs from the header, trailing bytes, an entry
that cannot be rebuilt and any bytes not in canonical form. `open_verified`
reads one byte more than the header declares, so a longer file is refused. On a
1,678-byte file of 30 states, each of the 13,424 single-bit changes, each of the
1,678 truncations and one appended byte were refused, each with its stated
reason.

## What it does not establish

- CRC64 detects accidental change; whoever rewrites the file can recompute it.
  Keep the SHA-256 of a saved file to show these are the bytes you kept.
- There is no generation: a valid older, shorter history opens without
  complaint.
- Saving rewrites the whole file; nothing is appended to a file on disk.
- In memory an entry takes 256 bytes on 64-bit targets, more than the 128-byte
  state it holds; the smaller size holds for the file only.
- A residual names its parent in 32 bits; an append that needs a larger parent
  index fails.
- The format is outside the crash series and the
  [format mutation matrix](MUTATION-MATRIX.md); a process kill or a power cut
  during saving is not measured.
- Nothing here concerns meaning, ranking or answers.

## Tests

In `crates/gel-history/tests/history.rs`:

- `entries_follow_literal_residual_residual_and_the_length_is_exact`
- `ninety_two_changed_bits_are_a_residual_and_ninety_three_a_literal`
- `three_states_give_the_hand_built_file`
- `an_empty_history_is_a_48_byte_file_without_a_latest_state`
- `append_returns_each_index_and_latest_is_the_last_state`
- `long_history_is_exact_and_residual_depth_never_exceeds_two`
- `an_unrelated_state_is_stored_literally`
- `every_bit_flip_and_truncation_of_a_30_state_file_is_refused`
- `a_depth_byte_that_lies_is_refused`
- `files_the_encoder_does_not_write_are_refused_as_not_canonical`
- `structurally_broken_entries_are_refused_with_their_reason`
- `the_header_count_and_fields_must_match_the_payload`
- `open_limits_hold_at_their_boundary_and_come_before_the_length`
- `an_older_shorter_history_still_opens`
- `saved_history_reopens_exactly_and_rejects_a_changed_payload`

In `crates/gel-history/src/lib.rs`:

- `the_residual_depth_limit_equals_the_structural_codec_limit`
- `an_entry_in_memory_takes_256_bytes_on_64_bit_targets`
- `rebuilding_stops_at_the_hop_bound_even_when_entries_break_it`
- `rebuilding_refuses_a_parent_that_does_not_precede_its_child`
- `a_dense_residual_is_never_encoded`
- `a_leftover_temporary_file_does_not_block_saving`

The [claim registry](CLAIMS.md) lists `record-history-exact` as an executable
check and `record-history-durability` as `NOT_ESTABLISHED`.
