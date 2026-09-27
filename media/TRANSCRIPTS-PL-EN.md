# Descriptive transcript / Transkrypcja opisowa

The public Evidence Lab film is silent and its hashes are hard to read on a
small screen, so its meaningful commands, answers and limitations are
described below; this is not a verbatim record of every cursor movement. Times
mark when each event first appears on screen; a result stays visible
afterwards. They were read from every visually distinct frame (ffmpeg
mpdecimate) in an AI frame review on 26 September 2026, see
[media review](../docs/MEDIA-DECODE-REVIEW.md). That review is not a human
sign-off. The two private application previews are not transcribed; their
scope and limits are summarized at the end. Original videos are unchanged.

Publiczny film Evidence Lab jest niemy, a hashe są na małym ekranie trudne do
odczytania, dlatego poniżej opisano jego istotne polecenia, odpowiedzi
i ograniczenia, nie każdy ruch kursora. Czas oznacza chwilę, w której zdarzenie
pojawia się na ekranie; wynik pozostaje widoczny także później. Czasy odczytano
ze wszystkich wizualnie różnych klatek w przeglądzie wykonanym przez AI, nie
przez człowieka. Dwóch prywatnych podglądów nie transkrybujemy; ich zakres
i ograniczenia podsumowano na końcu.

## 1. Public Evidence Lab / Publiczne narzędzie — 70.35 s

[Video](GEL-EVIDENCE-LAB-EN.mp4) · [Reproduction and provenance](EVIDENCE-LAB-GUIDE.md)

| At / Chwila | English description | Polski opis |
|---|---|---|
| 5.7 s | A centered Linux terminal labels scripted typing, live output, offline CPU execution and phrase retrieval, NOT semantic conversation. The operator starts gel-evidence. | Wyśrodkowany terminal oznacza automatyczne wpisywanie i rzeczywiste wyniki. Uruchamiane jest wyszukiwanie fraz, nie rozmowa semantyczna. |
| 8.2 s, 10.1 s | add memory.txt and add unicode.txt import 169 and 139 bytes. | Dodanie dwóch dokumentów: łącznie 308 bajtów tekstu. |
| 12.2 s | find ram is volatile returns HIT: “RAM is volatile.” Document 1, bytes 0..16. | Trafienie z cytatem “RAM is volatile.” (po polsku: RAM jest pamięcią ulotną). Dokument 1, bajty 0..16. |
| 20.7 s | proof 1 returns CITATION=PASS with document and collection hashes. | Sprawdzenie powiązania cytatu ze źródłem, nie dowód prawdziwości treści. |
| 25.1 s | find imaginary evidence returns UNKNOWN. | Nieobecna fraza nie powoduje wygenerowania odpowiedzi. |
| 31.1 s | save checkpoint.gelset saves revision 2 and prints its SHA256 pin. | Jawny zapis snapshotu i wypisanie sumy kontrolnej do niezależnego zachowania. |
| 36.5 s, 39.5 s | exit ends the first process; a new process is started. | Pierwszy proces kończy się; uruchamiany jest nowy. |
| 45.9 s | load with the retained pin returns REOPEN=PASS, revision 2. | Odtworzenie zapisanej kolekcji z kontrolą przypiętej sumy. |
| 51.3 s, 59.8 s | The same find and proof return the same quote, byte range and hashes. | Ponowny odczyt zwraca ten sam cytat i jego identyfikację. |
| 66.2 s | exit closes the second process. Only explicitly saved snapshots persist. | Zakończenie. Trwałe są wyłącznie jawnie zapisane snapshoty. |

The header, the typed `$ gel-evidence` lines, the restart separator and the
closing lines come from the scripted recorder, not from gel-evidence. Its
closing words “citation verified” mean the hash correspondence reported as
CITATION=PASS, not that the quoted content is true.
Nagłówek, wpisywane `$ gel-evidence`, separator restartu i końcowe zdania
pochodzą z nagrywarki, nie z aplikacji. „citation verified” oznacza zgodność
skrótów (CITATION=PASS), nie prawdziwość cytowanej treści.

Exact command sequence and full hashes: [commands](evidence-lab/commands.txt),
[first process](evidence-lab/process-1.txt), [second process](evidence-lab/process-2.txt).

| Operation / Operacja | Recorded time / Zapisany czas |
|---|---:|
| First HIT / Pierwsze trafienie | 0.047389 ms |
| UNKNOWN / Brak trafienia | 0.045666 ms |
| Save / Zapis | 12.155075 ms |
| Load / Odtworzenie | 0.074550 ms |
| HIT after restart / Trafienie po restarcie | 0.066134 ms |

These are individual application measurements, not percentiles, ORB/s or a
million-record search. Typing, reading pauses and terminal rendering are outside
the timers. The historical film predates the newer bounded-context renderer.

To pojedyncze pomiary operacji na 308 bajtach wejścia, nie percentyle ani
wydajność Oceanu. Zegar nie obejmuje wpisywania, pauz ani renderowania terminala.
Film powstał przed dodaniem nowego sposobu wyświetlania otaczającego kontekstu.

## Private application previews / Prywatne podglądy aplikacji

The [continuous chat](GEL-RAM-CONTINUOUS-CHAT-EN-60s.mp4) and
[hardware and short answers](GEL-RAM-HARDWARE-EN-CENTERED-90s.mp4) films were
recorded on 13 September 2026 and show a separate private native Rust
application. Its engine, bank and logs are not in this repository, so nothing
in these films can be reproduced from this checkout. Displayed times are single
on-screen values, not a controlled performance comparison. The answers are
bounded and source-based, not produced by an unrestricted language model.
Neither preview establishes general understanding, persistent learning, network
operation, hardware-level memory computation or chat ORB/s. The public checkout
does not provide the private /chat. For reproduction, use the Evidence Lab
above.

Filmy z 13 września 2026 r. pokazują osobną, prywatną aplikację w Rust. Jej
silnika, banku ani logów nie ma w tym repozytorium, więc niczego z tych filmów
nie da się odtworzyć z tego checkoutu. Czasy na ekranie to pojedyncze odczyty,
nie kontrolowane porównanie wydajności. Odpowiedzi są ograniczone i oparte na
źródle, nie pochodzą z nieograniczonego modelu językowego. Podglądy nie
dowodzą ogólnego rozumienia, trwałego uczenia, działania sieciowego, obliczeń
na poziomie sprzętu pamięci ani przepustowości rozmowy w ORB/s. Publiczne
repozytorium nie zawiera prywatnego /chat.
