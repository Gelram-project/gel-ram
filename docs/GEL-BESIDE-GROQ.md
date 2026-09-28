# GEL RAM beside three language models — 80 frozen questions

Recorded on 2026-09-28 in one run. Author-run: the GEL implementation and its
bank are private, and the model calls need a Groq account, so this cannot be
re-run from this checkout. The [claim registry](CLAIMS.md) lists it as
`MEASURED_LOCAL`.

[![Each of the 80 questions as one cell per system. GEL RAM: 11 correct, 0 wrong, 69 UNKNOWN. GPT-OSS-120B: 10 correct, 21 wrong, 49 UNKNOWN. GPT-OSS-20B: 8 correct, 28 wrong, 44 UNKNOWN. Qwen3.8-27B: 6 correct, 11 wrong, 63 UNKNOWN.](../media/beside-groq/all-80-answers.png)](../media/beside-groq/GEL-BESIDE-GROQ-80-QUESTIONS-EN.mp4)

## What was compared

- **The same 80 questions** (40 Polish, 40 English), written in natural language
  for 80 passages drawn at random from the GEL bank of about one million
  Wikipedia passages. Questions, expected answers, prompts and the protocol were
  frozen by SHA-256 before any model was called.
- **GEL RAM** answers from its local bank. When its best passage clearly leads,
  it returns that source passage; otherwise it says UNKNOWN. Same configuration
  as the [natural-question result](MEASURED-PROGRESS.md#natural-questions-on-the-same-bank),
  re-run in the same session with byte-identical output.
- **GPT-OSS-120B, GPT-OSS-20B and Qwen3.8-27B** (`openai/gpt-oss-120b`,
  `openai/gpt-oss-20b`, `qwen/qwen3.8-27b`) were called through the Groq API
  **closed book**: no passages and no tools. Temperature 0, at most 4096 output
  tokens, reasoning effort `low` for the GPT-OSS models, one question per
  request, 4 s between requests. Rate-limit replies (HTTP 429, seven for Qwen)
  were retried as the protocol allowed; waiting time is not in the timings.
- **One scoring rule for everyone.** An answer is correct when it contains the
  fact stated in the question's source passage. Accepted spellings were written
  from the passage before the run; for list questions one listed item is
  enough. Anything else is wrong. UNKNOWN is counted separately, never as wrong.
  A manual review, whose two permitted corrections were fixed in the protocol,
  changed two verdicts; both are listed below.

The English prompt, verbatim (the Polish one says the same and names Polish
Wikipedia):

```text
Answer the question from your own knowledge. The question refers to a fact stated in an English Wikipedia article. Give one short answer (at most 12 words) and no explanation. If you are not sure of the answer, reply with exactly: UNKNOWN

Question: {question}
```

## Results

| System | Conditions | Answered | Correct | Wrong | UNKNOWN | Errors | Correct among answers (95% Wilson) |
|---|---|---:|---:|---:|---:|---:|---|
| GEL RAM | local bank, answers with a source passage | 11 | 11 | 0 | 69 | 0 | 11/11 (74–100%) |
| GPT-OSS-120B | Groq API, closed book | 31 | 10 | 21 | 49 | 0 | 10/31 (19–50%) |
| GPT-OSS-20B | Groq API, closed book | 36 | 8 | 28 | 44 | 0 | 8/36 (12–38%) |
| Qwen3.8-27B | Groq API, closed book | 17 | 6 | 11 | 63 | 0 | 6/17 (17–59%) |

| System | PL: correct / wrong / UNKNOWN | EN: correct / wrong / UNKNOWN |
|---|---|---|
| GEL RAM | 6 / 0 / 34 | 5 / 0 / 35 |
| GPT-OSS-120B | 4 / 8 / 28 | 6 / 13 / 21 |
| GPT-OSS-20B | 4 / 14 / 22 | 4 / 14 / 22 |
| Qwen3.8-27B | 4 / 8 / 28 | 2 / 3 / 35 |

What stands out:

- **GEL gave no wrong answer.** The models were also allowed to say UNKNOWN,
  yet 11–28 of their answers were wrong: 65–78% of what each model answered.
- **The two sides knew different questions.** On 10 of the 11 questions GEL
  answered, no model was correct. On question 41 all three models gave the same
  wrong reason (the MP died); the source says John Hare was elevated to the
  House of Lords.
- **The models knew facts GEL did not find.** On 12 questions at least one model
  was correct where GEL said UNKNOWN. Answering only 11 of 80 remains GEL's open
  problem.
- **Recorded times, not a race.** GEL answered the 80 questions locally in one
  batch of 0.25 s (0.67 s including loading the bank). Over the network the
  median time per question was 348 ms (GPT-OSS-120B), 371 ms (GPT-OSS-20B) and
  59 ms (Qwen3.8-27B); the server-reported medians were 134, 123 and 12 ms.

### Manual review

| Model | Question | Automatic | Reviewed | Reason |
|---|---:|---|---|---|
| GPT-OSS-120B | 38 | correct | wrong | Only the generic word "well" matched; the answer names Beersheba, the source names Beer-lahai-roi |
| Qwen3.8-27B | 25 | wrong | correct | "Victoria" is the short name of Victoria Peak, one of the two summits the source names |

Automatic totals: GPT-OSS-120B 11 correct / 20 wrong, Qwen3.8-27B 5 correct /
12 wrong; the others are unchanged. No new rule was added after the run, so
some answers stay wrong although they are not simply false:

- true but less specific than the source: GPT-OSS-20B "Los Angeles" for Century
  City (question 29) and "1923" for 19 April (question 65);
- a different fact that other sources give: GPT-OSS-120B derives the name
  Abelard from Adalhard (question 3), and both GPT-OSS models name Rochester
  instead of Berwick (question 61).

Counting none of these as wrong would still leave 19, 25 and 11 wrong answers.


## Films

- [All 80 questions, side by side (5 min 7 s)](../media/beside-groq/GEL-BESIDE-GROQ-80-QUESTIONS-EN.mp4)
- [Summary (30 s)](../media/beside-groq/GEL-BESIDE-GROQ-SUMMARY-EN.mp4)

Both are silent 1920×1080 replays rendered from the recorded run files, not
screen captures. Every question, answer, verdict and time on screen comes from
those files; the pacing (3.5 s per question) is not execution time.

![Scoreboard frame from the summary film](../media/beside-groq/scoreboard.png)

## What this shows — and what it does not

- The two sides do different jobs. GEL looks facts up in a collection it holds;
  the models answer from what they learned in training. This is not a
  retrieval-augmented setup and not a comparison of engines.
- Several questions are underspecified without the collection ("the choir", "the
  company", "the regiment"). For a closed-book model UNKNOWN is the right reply.
- The questions and expected answers were written by the project's AI coding
  assistant for passages in the bank; they were not written or reviewed
  independently. GEL returns stored passages, so its correct answers measure
  whether it chose the right passage.
- Eighty questions give wide intervals. No 99% precision is claimed for anyone.
- Times are recorded values under different conditions: GEL ran locally in one
  batch, the models were called over the network one question at a time. They
  are not a speed comparison.
- Not claimed: general question answering, semantic understanding, superiority
  over language models, or a speed advantage.

## Evidence

The full record — question, expected answer, GEL verdict with the source
excerpt, and each model's verdict, answer and time — is
[side-by-side.txt](evidence-side-by-side/side-by-side.txt) (tab-separated).
Raw requests and responses are retained privately; the SHA-256 values identify
them but do not make them reproducible from this checkout.

| Artifact | SHA-256 |
|---|---|
| Questions (frozen) | 9a1b13450bb4767bfdb43071d00ed0506e0a7ca718fb780b41c5ee35bb103e4a |
| Expected answers and accepted spellings (frozen) | 81e1d75dfe2358eda4c4d59ad46a660422b53cf08364c43d8592f663d071d670 |
| Polish prompt | 73204ceb1635449a5a3a8bd2db657eb33744174f484c1c37b818db9a3f1605dd |
| English prompt | 5880eedb70c87a34565bda47fd7c542036edf0f3e1ee8b170136f03a1316c705 |
| Protocol (private, frozen before the run) | 44f93b01b809f351c47c2e2019b124472a2d88e4f83820f9c71279e0d5e6cec0 |
| GEL output | 50714466b576a525a4627b24165740bbc1734d5bff9ab1710e5fd6fc6469797e |
| GPT-OSS-120B answers | 24b78585cb745cf24769984abb04f96dfa2696b23cf0eeb4be99419fd7cc4c93 |
| GPT-OSS-20B answers | de737ee6ed27ba66e70fdb5d54629753d128c487a27a5c57d11f0efc1ae28824 |
| Qwen3.8-27B answers | 8967431a606a615ca9c0ab2a504a0b256e1e9a2f10d3074ce29e5dc93bf4a983 |
| Automatic scoring | 96efd5054d8b0601f6af9a4d69a70ab921fc847c2ff22e43fcf298b0e4f093df |
| Manual review | 0256bc78aa9c5a90037703ae1511efdfc966669f38e7cb99c20dc28f6626046d |
| Reviewed scoring | 77cb7c0654b3e86419761a8ab7f043964056f3fd7d2537f05ca900e5ea3c1762 |

Source passages quoted here and in the films are from Wikipedia and are
available under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/);
the questions paraphrase them. Model answers are reproduced as returned.

## All 80 questions

✅ correct · ❌ wrong · ◻️ UNKNOWN · ⚠️ technical error. GEL cells quote the returned source passage around the expected answer.

| # | Question | Expected (from the source) | GEL RAM | GPT-OSS-120B | GPT-OSS-20B | Qwen3.8-27B |
|---:|---|---|---|---|---|---|
| 1 | Którzy znani dziennikarze odeszli z Polskiego Radia, gdy słuchalność spadła do rekordowo niskiego poziomu? | Hirek Wrona, Artur Andrus, Wojciech Mann, Marcin Kydryński, Marek Niedźwiecki | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 2 | Kto skomponował muzykę do filmu „Święte serce”? | Andrea Guerra | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 3 | Od jakiego słowa pochodzi imię Abelard i co ono oznaczało? | prowansalskie „abelha” – „pszczelarz” | ◻️ UNKNOWN | ❌ Od imienia Adalhard, oznaczającego „szlachetny, mocny”. | ❌ Abel – „oddech, para” (z hebrajskiego słowa oznaczającego „oddech”). | ❌ Od hebrajskiego abel, co oznaczało „dym”. |
| 4 | W którym klubie zaczynał karierę brazylijski piłkarz, który potem grał w Botafogo i Kashimie Antlers? | Goiás EC | ◻️ UNKNOWN | ❌ Botafogo. | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 5 | Czym jest blasteza w geologii? | krystalizacja lub rekrystalizacja skał w stanie stałym | ◻️ UNKNOWN | ❌ Rodzaj brekcji – skała z ostrych, zrośniętych fragmentów. | ◻️ UNKNOWN | ❌ Blasteza to szczelina w skale wypełniona krzemieniem. |
| 6 | Jak długo trwa utwór „Julia” na płycie z piosenką „Sexcrime (Nineteen Eighty-Four)”? | 6:40 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 7 | Ile punktów zdobył Appelman w meczu z 10 lutego 2011 roku? | 9 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 8 | Ile meczów w reprezentacji Albanii rozegrał piłkarz, który grał w Liteksie i APOEL-u? | 68 | ◻️ UNKNOWN | ❌ 78 | ❌ 30 | ◻️ UNKNOWN |
| 9 | W jakim powiecie leży gmina Altenglan? | powiat Kusel | ✅ “…Nadrenia-Palatynat, w powiecie Kusel Altenglan – dawna gmina związkowa w kraju…” | ◻️ UNKNOWN | ❌ Kreis Bad Dürkheim. | ❌ Powiat Bad Dürkheim |
| 10 | Kto zagrał Grendela w filmie o Beowulfie z Gerardem Butlerem? | Ingvar Eggert Sigurðsson | ◻️ UNKNOWN | ❌ Robin Atkin Downes | ◻️ UNKNOWN | ❌ Ralph Fiennes |
| 11 | Z jakim wynikiem Phil Taylor pokonał Andy'ego Jenkinsa w drugiej rundzie? | 3:0 | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ 6-0 | ◻️ UNKNOWN |
| 12 | Z jakich białek składa się aktomiozyna? | aktyna i miozyna | ✅ “…w wodzie kompleks dwóch białek: aktyny i miozyny. Powstaje w czasie skurczu mięśnia,…” | ✅ actin i myosin | ✅ Actomyosin consists of actin and myosin proteins. | ✅ Aktyny i miozyny |
| 13 | Gdzie urodził się Andrzej Krzysztof Łuczak i jakie muzeum założył? | Łódź; Muzeum im. Leokadii Marciniak | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 14 | Za jaki film przyznano nagrodę za scenariusz na Lubuskim Lecie Filmowym w Łagowie w 1973 roku? | „Wesele” | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 15 | Kiedy i gdzie po raz pierwszy stwierdzono A. conica w Europie? | 2003, okolice Padwy | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 16 | W jakim biurze zaprojektowano okręty typu 209/1200, takie jak „San Luis”? | Ingenieurkontor Lübeck | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ Lürssen |
| 17 | W którym roku wybudowano nowy budynek WTC7? | 2006 | ◻️ UNKNOWN | ✅ 2006 | ❌ 2001 | ◻️ UNKNOWN |
| 18 | W którym roku Łukasiewicz uprościł aksjomat Nicoda? | 1925 | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ 1920 | ❌ 1930 |
| 19 | Kto dostał nagrodę za rolę kobiecą w sekcji „Un Certain Regard”? | Dorotheea Petre | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 20 | Jak nazywa się praca przedstawiająca sprzedawcę herbaty z Kairu? | The Tea Seller (Souvenir of Cairo, 1862) | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 21 | Z jakim wynikiem dwumeczu zakończyła się rywalizacja z CD Primeiro de Agosto w Pucharze Zdobywców Pucharów w 1991 roku? | 1–9 | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ 3-2 aggregate. | ◻️ UNKNOWN |
| 22 | Ilu zabitych i rannych stracili Niemcy w drugiej bitwie pod Narwikiem? | 128 zabitych, 67 rannych | ◻️ UNKNOWN | ❌ 0 zabitych, 2 rannych. | ❌ 1,000 zabitych i 2,000 rannych. | ◻️ UNKNOWN |
| 23 | Kto zastąpił Imre Senkeya na stanowisku trenera Romy w sezonie 1947/1948? | Luigi Brunella | ◻️ UNKNOWN | ❌ Józef Kałuża. | ❌ Giuseppe Viani. | ◻️ UNKNOWN |
| 24 | Kto śpiewał piosenkę „Syberiada polska” do filmu o tym samym tytule? | Anna Wyszkoni i Piotr Cugowski | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ Czesław Niemen. | ❌ Zbigniew Wodecki |
| 25 | Jaki jest najwyższy szczyt Belize? | Victoria Peak lub Doyle’s Delight | ◻️ UNKNOWN | ✅ Doyle’s Delight. | ✅ Doyle's Delight, 1,124 m. | ✅ Victoria |
| 26 | Kiedy przyjął święcenia kapłańskie duchowny z archidiecezji Gitega? | 25 lipca 1981 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 27 | Kto nagrał album dziecięcy „Flying High!”? | Caspar Babypants | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 28 | Kto był prezesem zarządu chóru w latach 1996–2000? | Fabian Cieślik | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 29 | Gdzie odbyła się 66. ceremonia wręczenia nagród Gildii Amerykańskich Reżyserów? | Hyatt Regency Century Plaza, Century City | ✅ “…25 stycznia 2014 r. w Hyatt Regency Century Plaza, w Century City w stanie Kalifornia.” | ◻️ UNKNOWN | ❌ Los Angeles. | ❌ Hotel Beverly Hilton |
| 30 | Na jakich uniwersytetach wykładał współtwórca porównawczej metody badania liturgii? | Bonn, Nijmegen, Utrecht, Münster | ✅ “…honorowym na uniwersytecie w Bonn, w 1923 w Nijmegen, 1926-1940 w Utrechcie a w…” | ◻️ UNKNOWN | ❌ Uniwersytet Warszawski i Uniwersytet Jagielloński. | ◻️ UNKNOWN |
| 31 | Jakim wynikiem zakończył się mecz ASVEL z Mazowszanką Pruszków 16 września 1997 roku? | 83:65 | ✅ “…w rozgrywkach grupowych. 16 września 1997 roku, Villeurbanne – ASVEL wygrywa 83:65” | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 32 | Który kronikarz opisał małżeństwo księżniczki szczecińskiej z Henrykiem II z rodu Werlów? | Ernest Kirchberg | ✅ “…XIV-wiecznego kronikarza Ernesta Kirchberga, który na kartach swej kroniki napisał o…” | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 33 | Ile punktów zdobył Kleibrink w meczu z 26 stycznia 2012 roku? | 8 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 34 | Czyją fałszywą biografię napisał Clifford Irving w filmie? | Howard Hughes | ◻️ UNKNOWN | ✅ Howard Hughes | ✅ Howard Hughes. | ✅ Howarda Hughesa |
| 35 | Kiedy Ben Malango zadebiutował w reprezentacji? | 11 sierpnia 2017 | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ 2015 | ◻️ UNKNOWN |
| 36 | Kto wyreżyserował film „Gniazdo ryjówek”? | Juanfer Andrés i Esteban Roel | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 37 | Kim byli różni ludzie o nazwisku Anthony Hamilton? | ojciec Lewisa Hamiltona; snookerzysta; wokalista R&B i soul; aktor (Antony) | ◻️ UNKNOWN | ◻️ UNKNOWN | ✅ Anthony Hamilton: amerykański wokalista soul (1979) i amerykański pił… | ◻️ UNKNOWN |
| 38 | Gdzie według Księgi Rodzaju Hagar zobaczyła anioła? | Beer-Lachaj-Roj | ◻️ UNKNOWN | ❌ przy studni na pustyni Beerseb | ❌ w pustyni. | ✅ Przy studni na pustyni |
| 39 | Jakie firmy wymieniono wśród klientów przedsiębiorstwa obok Volkswagena? | m.in. British American Tobacco, PGE, Samsung, Orange, Coca-Cola HBC | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 40 | Jak zakończył się mecz Füchse Berlin z SC Magdeburg 3 marca 2009 roku? | 21:28 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 41 | Why was the 1963 Sudbury and Woodbridge by-election held? | John Hare was elevated to the House of Lords | ✅ “…Hare was elevated to the House of Lords. The seat was held by the Conservative Party.…” | ❌ Because the incumbent MP died, triggering a by‑election. | ❌ Because the sitting MP died. | ❌ The death of the sitting Member of Parliament. |
| 42 | Who won first prize for poetry with "Crossworks"? | Cirilo F. Bautista | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 43 | Which aircraft replaced the C-130E Hercules in 1986? | C-130H Hercules | ◻️ UNKNOWN | ✅ C‑130H Hercules. | ✅ C‑130H. | ✅ C-130H Hercules |
| 44 | How many people died in the Voghera train crash? | 63 | ◻️ UNKNOWN | ❌ 6 | ❌ 4 | ◻️ UNKNOWN |
| 45 | On which TV network was Hockey Night in Canada broadcast in its fourth season? | CBC Television | ◻️ UNKNOWN | ✅ CBC. | ✅ CBC Television. | ◻️ UNKNOWN |
| 46 | To which team was Joaquín Andújar traded by the Reds in October 1975? | Houston Astros | ◻️ UNKNOWN | ❌ St. Louis Cardinals | ✅ Houston Astros | ◻️ UNKNOWN |
| 47 | Who was named top goaltender of the Winnipeg Rangers? | George Surmay | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 48 | Who won the first semifinal heat on 13 March running for Bulgaria? | Svetla Zlateva | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ Stanimir Stoyanov | ◻️ UNKNOWN |
| 49 | How many enlisted men of the regiment died of disease? | 104 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 50 | Where were the Eastern Conference playoff and the NFL Championship game played? | Yankee Stadium, New York City | ◻️ UNKNOWN | ❌ Polo Grounds, New York. | ❌ Cleveland (Eastern Conference playoff) and Los Angeles Memorial Colis… | ◻️ UNKNOWN |
| 51 | What is the regiment's war cry and what does it mean? | Durga Mata Ki Jai — Victory to the Mother Goddess Durga | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 52 | Where was the replay between FC Red Star Zürich and US Pro Daro played? | Lucerne | ✅ “…This was played on 3 July in Lucerne. Team 1 Score Team 2 FC Red Star Zürich 2–1…” | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 53 | Whom did Straus defeat at Mile End in the 1906 general election? | Levy-Lawson | ✅ “…year, Straus again faced Levy-Lawson at Mile End and managed to unseat him to…” | ❌ Sir Edward Clarke | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 54 | At whose salon was the Florentine Camerata formed? | Count Giovanni de' Bardi | ◻️ UNKNOWN | ❌ Giovanni de’ Medici (future Pope Leo X). | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 55 | Who finished first in the group with Fibak, Dibbs and Tanner? | Manuel Orantes | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 56 | Which Attorney General was made a Knight Bachelor together with Justice Luckhoo? | Shridath Surendranauth Ramphal | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 57 | Who was the Democratic candidate against Albert McIntire? | Charles S. Thomas | ◻️ UNKNOWN | ❌ William H. Adams | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 58 | Which player joined from Accrington Stanley in 1904? | Hugh Morgan | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ John H. Smith | ◻️ UNKNOWN |
| 59 | How many votes did George William Whittaker receive in St. Domingo? | 1,345 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 60 | Who were the unopposed councillors elected for Cardigan in 1979? | W. Jenkins, O.M. Owen, I.J.C. Radley | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 61 | Which town did King John's army sack during the First Barons' War? | Berwick-on-Tweed | ◻️ UNKNOWN | ❌ Rochester. | ❌ Rochester. | ◻️ UNKNOWN |
| 62 | Which horse lost its chance at the Triple Crown after fracturing a sesamoid bone? | Tim Tam | ◻️ UNKNOWN | ❌ Easy Goer | ❌ Cigar | ◻️ UNKNOWN |
| 63 | Who led the rebellion that burned Derry after the Flight of the Earls? | Sir Cahir O'Doherty | ◻️ UNKNOWN | ✅ Sir Cahir O'Doherty. | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 64 | Which vampire story did John William Polidori publish? | The Vampyre | ◻️ UNKNOWN | ✅ The Vampyre. | ✅ The Vampyre. | ✅ The Vampyre |
| 65 | When was the writer Lygia Fagundes Telles born? | 19 April | ✅ “…basketball player (died 1992) 19 April – Lygia Fagundes Telles, novelist and writer…” | ❌ 19 March 1923 | ❌ 1923 | ◻️ UNKNOWN |
| 66 | Where was the Scandinavian speedway round held on 5 June? | Speedway Center, Fredericia | ◻️ UNKNOWN | ❌ Måløy, Norway. | ❌ Gävle, Sweden. | ◻️ UNKNOWN |
| 67 | What was the score when Hearts beat St Bernard's in May 1895? | 5–0 (4 May) or 6–0 (25 May) | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 68 | What was the result of the game at Bethany in West Virginia? | 6–6 tie | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 69 | Which Sussex batsman had the highest batting average? | C. B. Fry | ◻️ UNKNOWN | ❌ Jack Hobbs | ❌ Jack Hobbs. | ◻️ UNKNOWN |
| 70 | Who was general manager and head coach of the 1966 Philadelphia Eagles? | Joe Kuharich | ◻️ UNKNOWN | ✅ Joe Kuharich | ❌ Mike McCormack | ❌ Joe Bailor |
| 71 | Who was the Prohibition candidate in the race with Charles Culberson? | J.M. Dunn | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 72 | Which night bomber division belonged to the army on 1 May 1945? | 262nd Night Bomber Aviation Division | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ 1st Night Bomber Division | ◻️ UNKNOWN |
| 73 | Why did Princeton forfeit its game against Harvard? | the Tigers were unable to participate | ✅ “…the game against Harvard on January 23 due to the Tigers being unable to participate.” | ◻️ UNKNOWN | ❌ Because Princeton used an ineligible player. | ◻️ UNKNOWN |
| 74 | When was a new senator elected in Mississippi after James Gordon's interim appointment? | February 23, 1910 | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 75 | Who managed the 1904 Boston Americans? | Jimmy Collins | ◻️ UNKNOWN | ✅ Jimmy Collins. | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 76 | How many strikeouts did Walter Thornton record? | 13 | ◻️ UNKNOWN | ❌ 0 | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 77 | Which Utah wide receiver was drafted by the Cincinnati Bengals in 1969? | Louis Thomas | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 78 | Which team won the Second Division Championship with 45 points? | Hull | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN |
| 79 | Who was selected first overall by the Detroit Pistons? | Bob Lanier | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ Isiah Thomas. | ◻️ UNKNOWN |
| 80 | What happened to the bodies of the passengers who fell into the nitric acid? | buried in a pit dug by villagers | ◻️ UNKNOWN | ◻️ UNKNOWN | ◻️ UNKNOWN | ❌ They were dissolved. |
