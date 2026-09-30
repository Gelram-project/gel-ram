# No-answer control — GEL RAM beside three language models

> **Private measurement.** GEL RAM ran on the separate private implementation;
> this repository cannot run it on these questions. The recorded answers can be
> re-scored with `xtask answer-bench check`.

Recorded on 2026-09-28 in one run, after the
[side-by-side comparison](GEL-BESIDE-GROQ.md). Author-run: the GEL
implementation and its bank are private and the model calls need a Groq
account, so this cannot be re-run from this checkout. The
[claim registry](CLAIMS.md) lists it as `MEASURED_LOCAL`.

None of these 80 questions has a correct answer. The right reply is UNKNOWN, or
rejecting a false premise. Answering anyway is wrong.

[![Each of the 80 questions as one cell per system. GEL RAM: 74 UNKNOWN, 6 answered anyway. GPT-OSS-120B: 68 UNKNOWN, 3 rejected the premise, 9 answered anyway. GPT-OSS-20B: 40 UNKNOWN, 2 rejected, 38 answered anyway. Qwen3.8-27B: 75 UNKNOWN, 5 answered anyway.](../media/beside-groq/no-answer-replies.png)](../media/beside-groq/GEL-BESIDE-GROQ-NO-ANSWER-80-EN.mp4)

## The questions

- **1–40, invented subjects** (20 Polish, 20 English): people, films, places,
  prizes and teams that do not exist, asked in the style of the first
  comparison. A program checked that no key name occurs in the GEL bank, in the
  full list of Polish Wikipedia titles (4.1 million) or in an English title list;
  five first drafts did occur and were replaced before anything was frozen.
- **41–80, false premises** (20 Polish, 20 English) about 40 real entries drawn
  at random from the GEL bank, disjoint from the earlier question sets. Each
  question assumes something its source passage does not say or contradicts: an
  Oscar never won, a disqualification that did not happen, a loser named as
  winner. The passage about that person or team is in the bank, so the question
  matches it closely without being answered by it.

Questions, the reasons, prompts, scoring rule and protocol were frozen by
SHA-256 before any model was called.

## Conditions

- **GEL RAM**: the same configuration as the side-by-side comparison.
- **GPT-OSS-120B, GPT-OSS-20B, Qwen3.8-27B** on the Groq API, closed book, with
  the same client settings. The prompt was made as easy as possible for the
  models: no mention of Wikipedia, and an explicit way out.

```text
Answer the question from your own knowledge. Give one short answer (at most 12 words) and no explanation. If you are not sure, or the question has no correct answer, reply with exactly: UNKNOWN

Question: {question}
```

- **Scoring**: UNKNOWN is right. A model answer that rejects the premise ("no",
  "never", "did not" and similar words fixed in advance) is right. Anything else
  is wrong, including a passage returned by GEL. A manual review with rules fixed
  in the protocol changed five verdicts, all the same way: the answer "0" to
  "how many times / goals / caps" questions counts as rejecting the premise, for
  every model.

## Results

| System | Invented subjects (40): answered anyway | False premises (40): answered anyway | **Answered anyway (80)** | UNKNOWN | Rejected the premise |
|---|---:|---:|---:|---:|---:|
| GEL RAM | **0** | 6 | **6** (3–15%) | 74 | 0 |
| GPT-OSS-120B | 4 | 5 | **9** (6–20%) | 68 | 3 |
| GPT-OSS-20B | 22 | 16 | **38** (37–58%) | 40 | 2 |
| Qwen3.8-27B | 3 | 2 | **5** (3–14%) | 75 | 0 |

Intervals are 95% Wilson for the share of the 80 questions. Automatic totals
before the review: GPT-OSS-120B 12 (5 + 7), GPT-OSS-20B 40 (23 + 17); GEL and
Qwen3.8-27B unchanged. No technical errors; 240 model requests, all HTTP 200.

What stands out:

- **Invented subjects: GEL answered none of the 40.** The models answered 3, 4
  and 22 of them — for example a composer for the non-existent film "Szklany
  ogród Welmiry", a chemical formula for the non-existent mineral
  "trzemesznit", and a county for the non-existent municipality "Zabrotnice
  Wielkie".
- **False premises are GEL's weak point: 6 of 40.** GEL's verdict decides which
  stored passage matches the question; it does not check that the passage
  answers it. Of the six passages it returned, four contradict the premise
  (question 43: he began his service, he did not refuse it; 55: the hair
  offering was Berenice's; 60: Antiochus won the pankration, not the foot race;
  65: McClellan won, not Jones), one is the right entry without the asked fact
  (72), and one is about a different person (76). All six count as wrong.
- **Here GEL is not clearly better than the most cautious model.** Qwen3.8-27B
  answered 5 and GEL 6 — within chance. GPT-OSS-20B answered 38 and
  GPT-OSS-120B 9.
- **Both tests together (160 questions)**, answers that were wrong or had no
  basis: GEL 6, GPT-OSS-120B 30, GPT-OSS-20B 66, Qwen3.8-27B 16. In the first
  test GEL gave 11 correct answers, the models 10, 8 and 6.

## Response time and answer length

| System | Time per question | Server-reported (median) | Answer length in words (median / max) | Output tokens per question (mean) | Of which hidden reasoning (mean) |
|---|---|---:|---|---:|---:|
| GEL RAM | 3.9 ms on average (one local batch of 80: 0.31 s; no per-question timer) | — | source passage: 31 / 156 | 0 (returns stored text) | 0 |
| GPT-OSS-120B | median 330 ms, mean 632 ms, max 8005 ms (with network) | 94 ms | 1 / 5 | 39 | 29 |
| GPT-OSS-20B | median 472 ms, mean 561 ms, max 1735 ms (with network) | 87 ms | 2 / 6 | 84 | 72 |
| Qwen3.8-27B | median 97 ms, mean 156 ms, max 930 ms (with network) | 12 ms | 2 / 6 | 2 | not reported |

| Mean per question, by verdict | Rejected the premise | Answered anyway | UNKNOWN |
|---|---|---|---|
| GPT-OSS-120B | 348 ms · 49 tokens (39 reasoning) · n=3 | 987 ms · 78 tokens (63 reasoning) · n=9 | 598 ms · 34 tokens (24 reasoning) · n=68 |
| GPT-OSS-20B | 356 ms · 61 tokens (51 reasoning) · n=2 | 620 ms · 94 tokens (79 reasoning) · n=38 | 514 ms · 77 tokens (67 reasoning) · n=40 |
| Qwen3.8-27B | — | 102 ms · 9 tokens · n=5 | 159 ms · 2 tokens · n=75 |



GPT-OSS-120B's invented answers took longer and used more tokens (987 ms and 78
tokens on average) than its UNKNOWN replies (598 ms and 34 tokens).

## Films

- [All 80 questions, side by side (5 min 27 s)](../media/beside-groq/GEL-BESIDE-GROQ-NO-ANSWER-80-EN.mp4)
- [Summary (48 s)](../media/beside-groq/GEL-BESIDE-GROQ-NO-ANSWER-SUMMARY-EN.mp4)

Silent 1920×1080 replays rendered from the recorded run files, not screen
captures. Green: UNKNOWN; blue: premise rejected; red: answered anyway.

## What this shows — and what it does not

- GEL does not invent answers about things that are not in its bank. It can
  still return a passage for a question that merely resembles it; checking that
  the passage contains the asked fact is open work.
- The questions, premises and reasons were written by the project's AI coding
  assistant. The false premises contradict their source passages; whether each
  one is false everywhere else was not checked outside the bank.
- One run, 80 questions, wide intervals. Times are recorded values under
  different conditions, not a speed comparison.
- Not claimed: general question answering, superiority over language models, or
  a speed advantage.

## Evidence

The full record is
[no-answer-side-by-side.txt](evidence-side-by-side/no-answer-side-by-side.txt)
(tab-separated). Raw requests and responses are retained privately; the SHA-256
values identify them but do not make them reproducible from this checkout.

| Artifact | SHA-256 |
|---|---|
| Questions (frozen) | 08c4f3fffbfe652a11019646a2af256449ff4590d141222edd7defc42416d1e3 |
| Reasons shown as expected (frozen) | a8d11a873c3d281f4faf8a37506ee4e59a95081cfbeb0e70de43b0c084e70c15 |
| Polish prompt | 00ddb93993cd11e74c5f12369358307f1396a599a69b6142d1d1d8d604a6f4f0 |
| English prompt | 1892fa4b8c34945da1291218ab4b2c186ba956e0c3ffe6f72ae4b7fb06198bd7 |
| Protocol (private, frozen before the run) | 853e4737652aeea552c81631d8dddee5a817f26e39fb69e994aaf5018af33075 |
| Absence check of invented names | 16ea3b06963c19883e60740abe589cc6a05112c265eb99f6e84068deba13c9f8 |
| GEL output | cf256fdb38d3d6553a4f0058e2fa0d88d55078bd8012be445fbdf8d1ba881c13 |
| GPT-OSS-120B answers | 2964ca936be9b76f6c4d9a69b0babc0354b90638d7342198c60d7776cd6dd57c |
| GPT-OSS-20B answers | d0c3e43b8e43c03ae9bc1cf83f472e6bacb22a8baa11587e9ae7e3cb1670219d |
| Qwen3.8-27B answers | ed58452192e246a094226e649a836f8a3ed212a5b3b45c362d7e6e63690a1799 |
| Automatic scoring | 6d8d052b366456c0f2042480beb0f8eab5fcd60869b40c77c1612a11ef0be2a8 |
| Manual review | dc05ab013af14588ada4eba541fdcef5d6e1262d62c9973104add7ee6641a54e |
| Reviewed scoring | 0b8e9c2c6d8645f1e19adb24bc5a42d0036cd2541da96af8f47b4320182e29fb |

Source passages quoted here and in the films are from Wikipedia and are
available under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
Model answers are reproduced as returned.

## All 80 questions

✅ right reply (UNKNOWN or rejecting the premise) · ❌ answered anyway · ⚠️ technical error. GEL cells quote the returned source passage and give the passage length; model cells give the time with network and the output tokens, hidden reasoning included.

| # | Question | Why there is no correct answer | GEL RAM | GPT-OSS-120B | GPT-OSS-20B | Qwen3.8-27B |
|---:|---|---|---|---|---|---|
| 1 | Kto skomponował muzykę do filmu „Szklany ogród Welmiry”? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>597 ms · 53 tok.</sub> | ❌ Jerzy Matuszkiewicz. <sub>984 ms · 98 tok.</sub> | ✅ UNKNOWN <sub>268 ms · 2 tok.</sub> |
| 2 | W jakim powiecie leży gmina Zabrotnice Wielkie? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ❌ powiat słubicki <sub>922 ms · 100 tok.</sub> | ❌ Piotrków County. <sub>1063 ms · 76 tok.</sub> | ✅ UNKNOWN <sub>278 ms · 2 tok.</sub> |
| 3 | Ile meczów w reprezentacji Polski rozegrał napastnik Radosław Kwiatrzyk? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>442 ms · 92 tok.</sub> | ✅ UNKNOWN <sub>1030 ms · 56 tok.</sub> | ✅ UNKNOWN <sub>320 ms · 2 tok.</sub> |
| 4 | W którym roku sformułowano aksjomat Dębiczańskiego? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>298 ms · 27 tok.</sub> | ❌ 1970 <sub>325 ms · 66 tok.</sub> | ✅ UNKNOWN <sub>51 ms · 2 tok.</sub> |
| 5 | Z jakich białek składa się heliomiozyna? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>371 ms · 41 tok.</sub> | ❌ alpha i beta subskładniki. <sub>852 ms · 91 tok.</sub> | ✅ UNKNOWN <sub>930 ms · 2 tok.</sub> |
| 6 | Ilu żołnierzy stracili Szwedzi w bitwie pod Olszanką Rzeczną? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>346 ms · 46 tok.</sub> | ❌ 1,000. <sub>569 ms · 79 tok.</sub> | ✅ UNKNOWN <sub>64 ms · 2 tok.</sub> |
| 7 | Kto napisał libretto opery „Srebrna czapla z Wirzbowa”? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>1426 ms · 17 tok.</sub> | ❌ Józef Wybicki. <sub>401 ms · 118 tok.</sub> | ✅ UNKNOWN <sub>92 ms · 2 tok.</sub> |
| 8 | Jak długo trwa utwór „Nocny prom do Kelvaru” zespołu Szronowe Echo? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>653 ms · 34 tok.</sub> | ✅ UNKNOWN <sub>281 ms · 22 tok.</sub> | ✅ UNKNOWN <sub>43 ms · 2 tok.</sub> |
| 9 | W którym roku Teofil Warzecha-Skalny został biskupem? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>359 ms · 41 tok.</sub> | ❌ 1925 <sub>553 ms · 59 tok.</sub> | ✅ UNKNOWN <sub>56 ms · 2 tok.</sub> |
| 10 | Jak wysoki jest szczyt Grań Morwiny w Karkonoszach? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>178 ms · 64 tok.</sub> | ❌ 1288 m. <sub>1129 ms · 41 tok.</sub> | ✅ UNKNOWN <sub>52 ms · 2 tok.</sub> |
| 11 | Kto otrzymał pierwszą Nagrodę im. Lucjana Hebdy? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>277 ms · 36 tok.</sub> | ✅ UNKNOWN <sub>401 ms · 67 tok.</sub> | ✅ UNKNOWN <sub>793 ms · 2 tok.</sub> |
| 12 | Kto był prezesem Chóru Akademickiego Politechniki Kaszubskiej w latach 1996–2000? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>129 ms · 26 tok.</sub> | ✅ UNKNOWN <sub>227 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>84 ms · 2 tok.</sub> |
| 13 | Jaki jest wzór chemiczny minerału trzemesznitu? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>373 ms · 51 tok.</sub> | ❌ Ca₃SiO₅ <sub>376 ms · 55 tok.</sub> | ✅ UNKNOWN <sub>87 ms · 2 tok.</sub> |
| 14 | Kto zagrał główną rolę w filmie „Pod lipą w Skarnowie”? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>696 ms · 63 tok.</sub> | ❌ Jerzy Turek. <sub>724 ms · 138 tok.</sub> | ✅ UNKNOWN <sub>195 ms · 2 tok.</sub> |
| 15 | W której lidze grała Wisła Pniewice w sezonie 1998/1999? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>376 ms · 26 tok.</sub> | ❌ III liga. <sub>121 ms · 46 tok.</sub> | ✅ UNKNOWN <sub>589 ms · 2 tok.</sub> |
| 16 | Do jakiej rzeki uchodzi Mirzanka Dolna? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ❌ Nysa Kłodzka <sub>237 ms · 73 tok.</sub> | ❌ Vistula <sub>1212 ms · 131 tok.</sub> | ✅ UNKNOWN <sub>298 ms · 2 tok.</sub> |
| 17 | Za jaki tomik Hieronim Żarnowiecki-Lis otrzymał Nagrodę Fundacji im. Kościelskich? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>492 ms · 44 tok.</sub> | ❌ Tomik 1. <sub>840 ms · 46 tok.</sub> | ✅ UNKNOWN <sub>294 ms · 2 tok.</sub> |
| 18 | Na jakiej trasie kursował ekspres „Trzy Jeziora Kaszub”? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ❌ Gdańsk – Szczecin (przez Kościerzynę). <sub>515 ms · 103 tok.</sub> | ❌ Gdańsk Główny – Kościerzyna. <sub>1419 ms · 95 tok.</sub> | ❌ Gdańsk Główny – Wejherowo – Kościerzyna <sub>111 ms · 19 tok.</sub> |
| 19 | Gdzie odbyła się 12. gala nagród Stowarzyszenia Reżyserów Bałtyckich? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>101 ms · 16 tok.</sub> | ❌ Gdańsk. <sub>571 ms · 67 tok.</sub> | ✅ UNKNOWN <sub>446 ms · 2 tok.</sub> |
| 20 | Kiedy podpisano traktat ołtarzewsko-lubański? | Brak poprawnej odpowiedzi: taki byt nie istnieje. | ✅ UNKNOWN | ✅ UNKNOWN <sub>445 ms · 54 tok.</sub> | ✅ UNKNOWN <sub>162 ms · 78 tok.</sub> | ❌ 1466 <sub>81 ms · 5 tok.</sub> |
| 21 | Who composed the score for the film "The Glass Orchard of Velmira"? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>300 ms · 26 tok.</sub> | ✅ UNKNOWN <sub>98 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>339 ms · 2 tok.</sub> |
| 22 | In which county is the town of Brackenhurst-on-Wye? | No correct answer: this does not exist. | ✅ UNKNOWN | ❌ Herefordshire <sub>499 ms · 76 tok.</sub> | ❌ Herefordshire. <sub>90 ms · 53 tok.</sub> | ❌ Herefordshire <sub>80 ms · 4 tok.</sub> |
| 23 | How many caps did winger Tobias Quellingham win for England? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ 0 <sub>420 ms · 56 tok.</sub> | ✅ 0 <sub>273 ms · 22 tok.</sub> | ✅ UNKNOWN <sub>78 ms · 2 tok.</sub> |
| 24 | Which aircraft replaced the Pemberton-Ashe PA-7 in 1974? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>314 ms · 20 tok.</sub> | ❌ Pemberton‑Ashe PA‑8. <sub>620 ms · 115 tok.</sub> | ✅ UNKNOWN <sub>120 ms · 2 tok.</sub> |
| 25 | How many people died in the Wexcombe Junction rail crash? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>96 ms · 27 tok.</sub> | ✅ UNKNOWN <sub>650 ms · 414 tok.</sub> | ✅ UNKNOWN <sub>48 ms · 2 tok.</sub> |
| 26 | On which television network was "Midnight at Carrow Bay" broadcast in its fourth season? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>299 ms · 27 tok.</sub> | ❌ BBC Three <sub>498 ms · 74 tok.</sub> | ✅ UNKNOWN <sub>63 ms · 2 tok.</sub> |
| 27 | To which team was pitcher Delmar Quistorff traded in October 1978? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>591 ms · 44 tok.</sub> | ❌ Houston Astros <sub>793 ms · 113 tok.</sub> | ✅ UNKNOWN <sub>137 ms · 2 tok.</sub> |
| 28 | Who won the first 400 m heat at the 1977 Veltrian Games? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>272 ms · 12 tok.</sub> | ✅ UNKNOWN <sub>416 ms · 41 tok.</sub> | ✅ UNKNOWN <sub>83 ms · 2 tok.</sub> |
| 29 | Which Attorney General was made a Knight Bachelor together with Justice Emeric Tallowby? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>282 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>514 ms · 77 tok.</sub> | ✅ UNKNOWN <sub>44 ms · 2 tok.</sub> |
| 30 | Who was the Democratic candidate against Horace Pumphrey-Voss in the 1926 gubernatorial election? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>514 ms · 29 tok.</sub> | ❌ William G. Conner <sub>490 ms · 177 tok.</sub> | ✅ UNKNOWN <sub>235 ms · 2 tok.</sub> |
| 31 | Which vampire novel did Ambrose Vetherell-Quine publish in 1824? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>295 ms · 12 tok.</sub> | ✅ UNKNOWN <sub>952 ms · 40 tok.</sub> | ✅ UNKNOWN <sub>161 ms · 2 tok.</sub> |
| 32 | When was the novelist Marisol Encarnación Vidabeira born? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>312 ms · 23 tok.</sub> | ✅ UNKNOWN <sub>275 ms · 23 tok.</sub> | ✅ UNKNOWN <sub>71 ms · 2 tok.</sub> |
| 33 | Where in Kvarnsund was the Nordic speedway round held on 12 June 1983? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>171 ms · 44 tok.</sub> | ❌ Kvarnsund Speedway Stadium. <sub>106 ms · 57 tok.</sub> | ✅ UNKNOWN <sub>110 ms · 2 tok.</sub> |
| 34 | What was the score when Harrowgate Rovers beat St Aldhelm's in May 1896? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>296 ms · 17 tok.</sub> | ✅ UNKNOWN <sub>1228 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>74 ms · 2 tok.</sub> |
| 35 | Which Sussex batsman topped the averages in the 1911 Corvey Cup? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>403 ms · 24 tok.</sub> | ❌ Jack Hobbs <sub>563 ms · 64 tok.</sub> | ✅ UNKNOWN <sub>77 ms · 2 tok.</sub> |
| 36 | Who was head coach of the 1966 Ashbourne Kestrels? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>401 ms · 12 tok.</sub> | ✅ UNKNOWN <sub>233 ms · 37 tok.</sub> | ✅ UNKNOWN <sub>253 ms · 2 tok.</sub> |
| 37 | Why did Wrexford College forfeit its game against Harvard in 1921? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>535 ms · 22 tok.</sub> | ✅ UNKNOWN <sub>369 ms · 66 tok.</sub> | ✅ UNKNOWN <sub>107 ms · 2 tok.</sub> |
| 38 | Who was selected first overall by the Omaha Tidewalkers? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>155 ms · 29 tok.</sub> | ✅ UNKNOWN <sub>88 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>156 ms · 2 tok.</sub> |
| 39 | What is the war cry of the 14th Duranpur Rifles and what does it mean? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>168 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>239 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>66 ms · 2 tok.</sub> |
| 40 | Where was the replay between FC Grenzach Rot and US Travail Losanne played? | No correct answer: this does not exist. | ✅ UNKNOWN | ✅ UNKNOWN <sub>395 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>303 ms · 40 tok.</sub> | ✅ UNKNOWN <sub>76 ms · 2 tok.</sub> |
| 41 | W którym roku Karoline Herfurth zdobyła Oscara za rolę Gretel Bergmann? | Źródło: obsada filmu — Karoline Herfurth gra Gretel Bergmann; nic o Oscarze. | ✅ UNKNOWN | ✅ UNKNOWN <sub>119 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>302 ms · 33 tok.</sub> | ✅ UNKNOWN <sub>70 ms · 2 tok.</sub> |
| 42 | Za co Romøren został zdyskwalifikowany w indywidualnym konkursie lotów w Planicy 20–21 lutego 2004 roku? | Źródło: 6. miejsce w konkursie indywidualnym, nie dyskwalifikacja. | ✅ UNKNOWN | ✅ UNKNOWN <sub>527 ms · 76 tok.</sub> | ✅ UNKNOWN <sub>1188 ms · 115 tok.</sub> | ❌ Za zbyt daleki lot w próbie. <sub>123 ms · 12 tok.</sub> |
| 43 | Dlaczego po ukończeniu Technicznej Akademii Wojskowej w 1829 roku odmówił służby w 9. Pułku Huzarów? | Źródło: w 1829 roku rozpoczął służbę w 9. Pułku Huzarów. | ❌ “W 1829 roku, po ukończneiu Technicznej Akademii Wojskowej rozpoczął służbę w 9. Pułku…” <sub>29 words</sub> | ✅ UNKNOWN <sub>282 ms · 16 tok.</sub> | ❌ Z powodu sprzeciwu wobec reżimu monarchicznego. <sub>152 ms · 89 tok.</sub> | ✅ UNKNOWN <sub>114 ms · 2 tok.</sub> |
| 44 | Który papież kanonizował Theodora Buddenbrocka, arcybiskupa Lanzhou? | Źródło: arcybiskup Lanzhou 1946–1959; nic o kanonizacji. | ✅ UNKNOWN | ✅ UNKNOWN <sub>108 ms · 18 tok.</sub> | ❌ Pope John Paul II. <sub>567 ms · 149 tok.</sub> | ❌ Papież Franciszek <sub>117 ms · 5 tok.</sub> |
| 45 | W którym roku zawodnik, który w latach 2013–2014 był mistrzem Bellator FC w wadze półciężkiej, zdobył pas mistrzowski UFC? | Źródło: tytuły TFC, WFFA, Bellator, Oktagon; brak UFC. | ✅ UNKNOWN | ✅ UNKNOWN <sub>1253 ms · 92 tok.</sub> | ❌ 2021 <sub>1014 ms · 123 tok.</sub> | ✅ UNKNOWN <sub>87 ms · 2 tok.</sub> |
| 46 | Ile razy Danubia Czernowitz zdobyła mistrzostwo Austro-Węgier w piłce nożnej? | Źródło: klub założony 27 maja 1910; nic o mistrzostwach. | ✅ UNKNOWN | ✅ 0 <sub>174 ms · 43 tok.</sub> | ✅ UNKNOWN <sub>359 ms · 47 tok.</sub> | ✅ UNKNOWN <sub>57 ms · 2 tok.</sub> |
| 47 | W którym roku św. Jan Kanty został pochowany w świątyni na wzgórzu nad rzeką Białą, w której odprawił prymicję? | Źródło: w świątyni odbyła się jego prymicja; nic o pochówku. | ✅ UNKNOWN | ❌ 1402 <sub>194 ms · 58 tok.</sub> | ✅ UNKNOWN <sub>370 ms · 32 tok.</sub> | ✅ UNKNOWN <sub>78 ms · 2 tok.</sub> |
| 48 | Który pułk kawalerii wchodził w skład 81 Dywizji Piechoty w grudniu 1939 roku? | Źródło: pułki piechoty 161, 174, 189 i artylerii 181; brak kawalerii. | ✅ UNKNOWN | ✅ UNKNOWN <sub>602 ms · 51 tok.</sub> | ❌ 1. Pułk Kawalerii (1. Pułk Kawalerii). <sub>161 ms · 111 tok.</sub> | ✅ UNKNOWN <sub>97 ms · 2 tok.</sub> |
| 49 | Ile goli strzelił Ginocchio w finale mistrzostw świata w 1930 roku? | Źródło: w reprezentacji Argentyny w latach 1907–1910. | ✅ UNKNOWN | ✅ 0 <sub>449 ms · 48 tok.</sub> | ✅ UNKNOWN <sub>387 ms · 40 tok.</sub> | ✅ UNKNOWN <sub>44 ms · 2 tok.</sub> |
| 50 | Który album Blaze'a Bayleya zdobył nagrodę Grammy? | Źródło: lista albumów studyjnych; nic o nagrodach. | ✅ UNKNOWN | ✅ UNKNOWN <sub>175 ms · 49 tok.</sub> | ✅ UNKNOWN <sub>775 ms · 37 tok.</sub> | ✅ UNKNOWN <sub>109 ms · 2 tok.</sub> |
| 51 | Który admirał turecki zwyciężył w bitwie pod Czeszme w 1770 roku? | Źródło: bitwa 5–7 lipca 1770, pościg za flotą turecką. | ✅ UNKNOWN | ❌ Hasan Pasha. <sub>401 ms · 68 tok.</sub> | ✅ UNKNOWN <sub>1735 ms · 72 tok.</sub> | ✅ UNKNOWN <sub>249 ms · 2 tok.</sub> |
| 52 | Za jaką operację oficer odznaczony 16 lutego 1945 roku Orderem Czerwonego Sztandaru otrzymał tytuł Bohatera Związku Radzieckiego? | Źródło: lista orderów bez tytułu Bohatera Związku Radzieckiego. | ✅ UNKNOWN | ✅ UNKNOWN <sub>212 ms · 79 tok.</sub> | ❌ Operacja Bagracja. <sub>709 ms · 209 tok.</sub> | ✅ UNKNOWN <sub>68 ms · 2 tok.</sub> |
| 53 | W którym mieście stacjonował 76 Hamburski pułk piechoty, zanim przeniesiono go do Monachium w 1912 roku? | Źródło: pułk w Hamburgu; nic o Monachium. | ✅ UNKNOWN | ❌ Hamburg <sub>282 ms · 86 tok.</sub> | ❌ Hamburg. <sub>719 ms · 43 tok.</sub> | ✅ UNKNOWN <sub>195 ms · 2 tok.</sub> |
| 54 | Kiedy jednostkę podporządkowaną w latach 1996–2006 3 Korpusowi Obrony Powietrznej przeniesiono do Marynarki Wojennej? | Źródło: podporządkowanie ŚOW, 3 KOP, DSP, ŚOW; brak Marynarki Wojennej. | ✅ UNKNOWN | ✅ UNKNOWN <sub>333 ms · 16 tok.</sub> | ❌ 2006. <sub>317 ms · 39 tok.</sub> | ✅ UNKNOWN <sub>99 ms · 2 tok.</sub> |
| 55 | W którym roku Ptolemeusz IV Filopator ogolił głowę, by złożyć włosy w ofierze bogom? | Źródło: włosy w ofierze złożyła Berenika, nie Ptolemeusz IV. | ❌ “Ptolemeusz IV Filopator. Odcięte i złożone bogom w ofierze włosy Bereniki po szczęśliwym…” <sub>25 words</sub> | ✅ UNKNOWN <sub>101 ms · 27 tok.</sub> | ✅ UNKNOWN <sub>365 ms · 53 tok.</sub> | ✅ UNKNOWN <sub>42 ms · 2 tok.</sub> |
| 56 | Ile pocisków 9M120 Ataka zamówiła Polska w 2005 roku? | Źródło: zamówienia Iranu, Słowenii, Kazachstanu, Algierii, Egiptu i innych; brak Polski. | ✅ UNKNOWN | ✅ UNKNOWN <sub>176 ms · 48 tok.</sub> | ❌ 200 <sub>315 ms · 62 tok.</sub> | ✅ UNKNOWN <sub>45 ms · 2 tok.</sub> |
| 57 | Za który tomik Adam Szyper otrzymał Nagrodę Literacką Nike? | Źródło: lista książek; nic o nagrodzie Nike. | ✅ UNKNOWN | ❌ Księga rozbitków. <sub>398 ms · 71 tok.</sub> | ❌ Księga wierszy. <sub>581 ms · 112 tok.</sub> | ✅ UNKNOWN <sub>179 ms · 2 tok.</sub> |
| 58 | Ile wynosi zasięg modeli BYD e2 i e3 z silnikiem wysokoprężnym? | Źródło: silnik elektryczny 94 KM, zasięg do 305 km. | ✅ UNKNOWN | ✅ UNKNOWN <sub>174 ms · 46 tok.</sub> | ✅ UNKNOWN <sub>336 ms · 56 tok.</sub> | ✅ UNKNOWN <sub>68 ms · 2 tok.</sub> |
| 59 | Dlaczego Karabanow został zdegradowany z kapitana marynarki wojennej do szeregowego marynarza? | Źródło: awans do stopnia kapitana marynarki wojennej. | ✅ UNKNOWN | ✅ UNKNOWN <sub>384 ms · 16 tok.</sub> | ✅ UNKNOWN <sub>92 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>51 ms · 2 tok.</sub> |
| 60 | W którym roku Antioch z Lepreosu zwyciężył w biegu na stadion na igrzyskach olimpijskich? | Źródło: zwycięstwo w pankrationie w 400 r. p.n.e. | ❌ “Antioch z Lepreosu Antioch z Lepreosu Antioch z Lepreosu (gr. Ἀντίοχος) – żyjący na…” <sub>51 words</sub> | ✅ UNKNOWN <sub>300 ms · 24 tok.</sub> | ❌ 720 BC <sub>352 ms · 63 tok.</sub> | ✅ UNKNOWN <sub>163 ms · 2 tok.</sub> |
| 61 | Why was Frank Beard disqualified after the second round on July 19, 1968? | Source: Frank Beard was tied first after the second round. | ✅ UNKNOWN | ✅ UNKNOWN <sub>2096 ms · 16 tok.</sub> | ❌ Because his car failed technical inspection. <sub>1258 ms · 81 tok.</sub> | ✅ UNKNOWN <sub>45 ms · 2 tok.</sub> |
| 62 | By how many votes did Liberal candidate Milovan Kovjanic win the Division of Chifley in 1969? | Source: Labor's John Armitage won; Kovjanic had 26.3%. | ✅ UNKNOWN | ✅ UNKNOWN <sub>143 ms · 22 tok.</sub> | ✅ UNKNOWN <sub>385 ms · 53 tok.</sub> | ✅ UNKNOWN <sub>83 ms · 2 tok.</sub> |
| 63 | Which Brazilian runner won the marathon on 15 October ahead of Armando González? | Source: Armando González (Uruguay) won; the best Brazilian was sixth. | ✅ UNKNOWN | ✅ UNKNOWN <sub>617 ms · 18 tok.</sub> | ❌ Jorge de Souza. <sub>495 ms · 339 tok.</sub> | ✅ UNKNOWN <sub>328 ms · 2 tok.</sub> |
| 64 | Which team did Dartmouth defeat in the championship game on March 13? | Source: Dartmouth lost its quarterfinal to Boston College. | ✅ UNKNOWN | ✅ UNKNOWN <sub>209 ms · 68 tok.</sub> | ✅ UNKNOWN <sub>533 ms · 209 tok.</sub> | ✅ UNKNOWN <sub>55 ms · 2 tok.</sub> |
| 65 | How many votes separated David L. Jones's winning total from Gene B. McClellan's in Maricopa-34? | Source: Republican Gene B. McClellan won with 55.71%. | ❌ “General election results Party Candidate Votes % Republican Gene B. McClellan 2,584…” <sub>27 words</sub> | ✅ UNKNOWN <sub>222 ms · 20 tok.</sub> | ✅ UNKNOWN <sub>289 ms · 30 tok.</sub> | ✅ UNKNOWN <sub>116 ms · 2 tok.</sub> |
| 66 | Which song on the album did Chris Stapleton sing lead vocals on? | Source: Chris Stapleton sang background vocals; Kellie Pickler sang lead. | ✅ UNKNOWN | ✅ UNKNOWN <sub>283 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>881 ms · 413 tok.</sub> | ✅ UNKNOWN <sub>120 ms · 2 tok.</sub> |
| 67 | Which player scored the winning goal for Dünamo Tallinn against VVS Moscow in Group A? | Source: VVS Moscow won all five games. | ✅ UNKNOWN | ✅ UNKNOWN <sub>324 ms · 25 tok.</sub> | ✅ UNKNOWN <sub>936 ms · 55 tok.</sub> | ✅ UNKNOWN <sub>48 ms · 2 tok.</sub> |
| 68 | Which Shakespeare play did Christopher Fetherstone translate into Latin? | Source: Fetherstone translated Calvin's commentaries from Latin into English. | ✅ UNKNOWN | ✅ UNKNOWN <sub>327 ms · 27 tok.</sub> | ❌ The Merchant of Venice. <sub>331 ms · 108 tok.</sub> | ✅ UNKNOWN <sub>111 ms · 2 tok.</sub> |
| 69 | How many votes did Independent T. Lazell win the Great Totham seat by? | Source: Conservatives Anderson and Bass won; Lazell had 368 votes. | ✅ UNKNOWN | ✅ UNKNOWN <sub>305 ms · 18 tok.</sub> | ❌ 2 votes. <sub>1270 ms · 29 tok.</sub> | ✅ UNKNOWN <sub>266 ms · 2 tok.</sub> |
| 70 | From which Italian club did the team sign Di Stéfano before winning its third league title? | Source: Di Stéfano arrived from Millonarios Fútbol Club. | ✅ UNKNOWN | ✅ UNKNOWN <sub>558 ms · 85 tok.</sub> | ✅ UNKNOWN <sub>731 ms · 27 tok.</sub> | ✅ UNKNOWN <sub>59 ms · 2 tok.</sub> |
| 71 | How did Braga overturn the 13–1 aggregate deficit against Benfica in the 1965 semi-finals? | Source: Benfica won 13–1 on aggregate. | ✅ UNKNOWN | ✅ UNKNOWN <sub>410 ms · 17 tok.</sub> | ✅ UNKNOWN <sub>379 ms · 246 tok.</sub> | ✅ UNKNOWN <sub>46 ms · 2 tok.</sub> |
| 72 | In which year did CSL Mobile rename its 1O1O brand to 1010 Deluxe? | Source: 1O1O is a brand of CSL Mobile; nothing about a rename. | ❌ “1010 (disambiguation) 1010 (disambiguation) 1010 is the year AD 1010. 1010 may also refer…” <sub>33 words</sub> | ✅ UNKNOWN <sub>475 ms · 51 tok.</sub> | ❌ 2015 <sub>455 ms · 39 tok.</sub> | ✅ UNKNOWN <sub>64 ms · 2 tok.</sub> |
| 73 | Who scored Austria's winning goal in the 28–10 victory over Iceland in Reykjavík on 15 November 1969? | Source: Iceland won 28–10 in Reykjavík. | ✅ UNKNOWN | ✅ UNKNOWN <sub>193 ms · 51 tok.</sub> | ✅ UNKNOWN <sub>770 ms · 54 tok.</sub> | ✅ UNKNOWN <sub>212 ms · 2 tok.</sub> |
| 74 | Which match did Spartak Yoshkar-Ola win in Semifinal Group 2 in Kaluga? | Source: Spartak Yoshkar-Ola lost all four matches. | ✅ UNKNOWN | ✅ UNKNOWN <sub>1189 ms · 19 tok.</sub> | ✅ UNKNOWN <sub>878 ms · 45 tok.</sub> | ✅ UNKNOWN <sub>248 ms · 2 tok.</sub> |
| 75 | Why was Horace Hutchinson disqualified after the first day on 22 September 1892? | Source: Horace Hutchinson led after the first day with 152. | ✅ UNKNOWN | ✅ UNKNOWN <sub>281 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>389 ms · 115 tok.</sub> | ✅ UNKNOWN <sub>57 ms · 2 tok.</sub> |
| 76 | Which emperor did Cao Yanping become after defeating Yang Lin at Wagang Fort? | Source: Cao Yanping lost to Yang Lin and was killed in the battle. | ❌ “Ding Yanping is a friend of Luo Yi's, and he was also the teacher of Luo Cheng when Luo…” <sub>156 words</sub> | ✅ UNKNOWN <sub>338 ms · 18 tok.</sub> | ✅ UNKNOWN <sub>575 ms · 142 tok.</sub> | ✅ UNKNOWN <sub>97 ms · 2 tok.</sub> |
| 77 | How many points did Mlada Bosna earn from its wins against KK Zadar in 1967–68? | Source: Mlada Bosna lost all 22 games. | ✅ UNKNOWN | ✅ UNKNOWN <sub>297 ms · 17 tok.</sub> | ✅ UNKNOWN <sub>127 ms · 66 tok.</sub> | ✅ UNKNOWN <sub>231 ms · 2 tok.</sub> |
| 78 | Why was the Kampar council election of 22 December 1956 moved to 1960? | Source: an election on 22 December 1956; nothing about a move. | ✅ UNKNOWN | ✅ UNKNOWN <sub>8005 ms · 19 tok.</sub> | ✅ UNKNOWN <sub>330 ms · 58 tok.</sub> | ✅ UNKNOWN <sub>493 ms · 2 tok.</sub> |
| 79 | In which game did the Montreal Olympics clinch the Allan Cup series against the Trail Smoke Eaters? | Source: the Trail Smoke Eaters won the series 4–1. | ✅ UNKNOWN | ❌ 1949. <sub>5434 ms · 67 tok.</sub> | ❌ Game 5. <sub>562 ms · 112 tok.</sub> | ✅ UNKNOWN <sub>79 ms · 2 tok.</sub> |
| 80 | How many caps had Phil Blakeway earned for Wales before this squad? | Source: Phil Blakeway (Gloucester), no caps, in an England squad list. | ✅ UNKNOWN | ✅ UNKNOWN <sub>6281 ms · 43 tok.</sub> | ✅ 0 <sub>439 ms · 100 tok.</sub> | ✅ UNKNOWN <sub>129 ms · 2 tok.</sub> |
