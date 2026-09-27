# Supplied-source decision diagnostic — author-reported

27 September 2026. Separate private GEL adapter and external models; not
performance of this public release. No private engine or credentials included.
Raw private logs are retained by the author, not published here. The numbers
below are a reviewed transcription, not independently reproducible evidence
of model quality from this checkout.

## Contract and profiles

Two public Wikipedia passages, six claims each, one batch per language/profile.
Working labels: SUPPORTED, REFUTED, UNKNOWN. Missing information is not
contradiction. The working gold is development-exposed, not independently
double-annotated. Neither side retrieves the supplied evidence from a corpus.

Returned model IDs in retained API responses:
`qwen/qwen3.8-27b`, `openai/gpt-oss-20b`,
`openai/gpt-oss-120b`. These identify observed runs, not claims about current
provider availability. R0 uses a 1024 completion-token cap. GPT-OSS R1 uses
`reasoning_effort=low` and a 4096 cap, selected after seeing R0 truncation.
This is diagnostic adaptation, not a frozen head-to-head ranking.

GEL's offline CPU adapter loads a one-source corpus for each language before
timing. It accepts the batch format, but all twelve claims hit its unsupported
grammar fallback. UNKNOWN is a syntactically valid answer; it is not evidence
that the question was understood. No LLM is used by the GEL adapter.

## Exact observations

Times in integer nanoseconds; one batch per row. HTTP timings include network.
GEL timings exclude source loading and disk I/O. No TTFT was recorded.

```text
system_profile,language,elapsed_ns,completion_status
gel_adapter_r0,pl,85851,6_UNKNOWN
gel_adapter_r0,en,78057,6_UNKNOWN
groq_qwen_r0,pl,537133987,stop
groq_qwen_r0,en,552672273,stop
groq_20b_r0,pl,1383913852,length
groq_120b_r0,pl,2407527820,length
groq_20b_r1,pl,818244816,stop
groq_20b_r1,en,879478351,stop
groq_120b_r1,pl,1219563992,stop
groq_120b_r1,en,1035559971,stop
```

Eight HTTP calls report 7063 total tokens. Two generations are incomplete,
not successful abstentions. Their EN requests were not executed. No account
billing or energy measurement is inferred from the user's free-tier account.

## Quality versus formatting

The strict output contract requires claim IDs, labels and exact source quotes
for S/R. The 20B R1 EN extra header rejects the batch protocol; it does not
prove six semantic errors. Recognizable labels after an identical, explicitly
post-hoc formatting normalization match 5/6 PL and 6/6 EN for all three models.
The strict results in README are not replaced by that normalization.

In PL, each model incorrectly refutes a universal claim from a passage saying
only “some”; that passage does not settle whether “all” is true. UNKNOWN is
the working gold. Other failures include nonliteral evidence quotations.
An exact quote and matching label still require review of semantic sufficiency.

GEL has 0/6 S/R coverage per language, not perfect precision. Its S/R precision
is undefined because it issued no S/R decisions. Do not divide LLM time by
GEL abstention time to claim faster successful reasoning.

## Identity of frozen inputs

The same prompt/source pins were checked locally for the native adapter and
the API requests. Hashes identify bytes; they do not prove truth or independence.

| Language | Prompt SHA256 | Source SHA256 |
|---|---|---|
| PL | cf21c81d39b8e51515d99280b73ba1d87a814fcc4ebd17742026a66ba37e76d9 | 331d478d4dce78ffbbf330cd7828e2304d4dda47b22d03edd73c9e803249ff88 |
| EN | 20a3da24b74149c1759fa1c58bac442a5cbd6f14a889530a85825b74f5f4ab5a | b3c0db82488508261d715a01f294d7f99867807b66d2e97f13b82f07fdc9944f |

These timing/quality observations are distinct from addressed RAM reads,
Single/Quad vector ranking and integrity-fault controls in README. They must
not be merged into one latency distribution or one overall accuracy number.
