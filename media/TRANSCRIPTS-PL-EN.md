# Descriptive transcripts / Transkrypcje opisowe

All four recordings are silent. These descriptions include the meaningful
commands, answers and limitations; they are not verbatim transcripts of every
cursor movement. Times below mark when each event first appears on screen; a
result stays visible afterwards. They were read from every visually distinct
frame (ffmpeg mpdecimate) in an AI frame review on 26 September 2026, see
[media review](../docs/MEDIA-DECODE-REVIEW.md). That review is not a human
sign-off. Original videos are unchanged.

Wszystkie trzy nagrania są nieme. Opisy obejmują istotne polecenia, odpowiedzi
i ograniczenia, nie każdy ruch kursora. Czas oznacza chwilę, w której zdarzenie
pojawia się na ekranie; wynik pozostaje widoczny także później. Czasy odczytano
ze wszystkich wizualnie różnych klatek w przeglądzie wykonanym przez AI, nie
przez człowieka. Polskie tłumaczenia są pomocą dla czytelnika, nie dodatkowymi
odpowiedziami GEL.

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

## 2. Private continuous chat / Prywatny podgląd rozmowy — 60 s

[Video](GEL-RAM-CONTINUOUS-CHAT-EN-60s.mp4) · [Historical scope](DEMO-CONTINUOUS-CHAT.md)

From 0 s to 15.7 s the terminal identifies a private preview and displays hardware,
memory and “Session transcript logging ON”. At 15.7 s /chat clears the screen;
from then on no private-preview label is visible, although the application
remains the private preview.
Responses remain on screen as subsequent questions are entered, without clear.

Od 0 do 15,7 s widać oznaczenie prywatnego podglądu, sprzęt, pamięć i napis
„Session transcript logging ON”. Przy 15,7 s /chat czyści ekran i oznaczenie
znika, choć nadal jest to prywatna aplikacja. Kolejne
pytania nie czyszczą wcześniejszej rozmowy.

### 18.5 s asked, 20.2 s answered: What do we know about Erich Honecker?

The displayed source-bound summary gives birth date “25 August 1912”, birthplace
“Neunkirchen”, death date “29 May 1994” and place of death “Santiago”. It explicitly
says these are four biographical fields, not a complete biography; a missing
death date does not establish that a person is alive; source consistency is not
independent verification of truth. Visible latency: **5.107 ms**, source 30:0.

Podsumowanie źródłowe podaje cztery pola: urodzenie 25 sierpnia 1912 r.
w Neunkirchen oraz śmierć 29 maja 1994 r. w Santiago. Zastrzega, że to nie pełna
biografia, brak daty śmierci nie dowodzi życia osoby, a zgodność ze źródłem nie
stanowi niezależnego potwierdzenia prawdy.

### 30.2 s asked, 31.8 s answered: What do we know about Rami Malek?

The summary gives birth date “12 May 1981” and birthplace “Torrance”. It says no
unambiguous date or place of death was extracted from this sample, and repeats
the three limitations above. Visible latency: **16.974 ms**, source 1:0.

Odpowiedź podaje urodzenie 12 maja 1981 r. w Torrance. Data i miejsce śmierci
pozostają nierozstrzygnięte w tej próbce; zachowane są powyższe ograniczenia.

### 41.8 s asked, 43.4 s answered: What do we know about John Lennon?

“I do not have an unambiguous source-bound answer. Please clarify the subject
and relation.” Visible latency: **23.707 ms**, UNKNOWN, no source anchor.

„Nie mam jednoznacznej odpowiedzi opartej na źródle. Doprecyzuj podmiot i relację”.
John Lennon exists in the catalog; this route failed to resolve a supported
summary. UNKNOWN does not prove that the bank lacks information about him.
Podmiot jest w katalogu, lecz ta ścieżka nie potrafiła złożyć jednoznacznego
podsumowania. Nie jest to dowód braku wiedzy w banku.

At 53.7 s /status shows three completed turns. At 57.4 s /exit closes the
session: no data published, test conversation volatile. Session transcript logging was
enabled; volatile session context does **not** mean that no transcript logs exist.

Przy 53,7 s status pokazuje trzy tury. Zakończenie (57,4 s) informuje o ulotności rozmowy
i braku publikacji. Nie oznacza to usunięcia ani braku zapisanych logów rozmowy.

## 3. Private hardware and short answers / Sprzęt i krótkie odpowiedzi — 90 s

[Video](GEL-RAM-HARDWARE-EN-CENTERED-90s.mp4) · [Historical scope](DEMO.md)

From 4.1 s to 24.5 s the application displays hardware and memory together with
the “Private preview” and “Private demonstration” labels. At 24.5 s /chat opens;
from then until the end no private-preview label is on screen, although the
application remains the private preview described here.

Od 4,1 s do 24,5 s aplikacja pokazuje sprzęt i pamięć wraz z oznaczeniem
prywatnego podglądu. Od 24,5 s (otwarcie /chat) do końca oznaczenia nie ma na
ekranie, choć nadal jest to prywatna aplikacja.

| At / Chwila | Question and English answer | Polski opis | Visible latency |
|---|---|---|---:|
| 30.4 s | Where was Rami Malek born? — The source gives the birthplace as “Torrance”. | Miejsce urodzenia według źródła: Torrance. | 1.336 ms |
| 42.2 s | And when was he born? — The source gives the date of birth as “12 May 1981”. | Dopowiedzenie utrzymuje temat; data urodzenia: 12 maja 1981 r. | 0.233 ms |
| 54.6 s | Where did Erich Honecker die? — The source gives the place of death as “Santiago”. | Zmiana osoby; miejsce śmierci według źródła: Santiago. | 0.436 ms |
| 66.3 s | When did his father die? — I cannot resolve that relationship with the supported source rules. I will not guess. | Odmowa zgadywania: reguły nie rozstrzygają relacji z ojcem. | 0.140 ms |

At 77.6 s /status shows four completed turns; /exit closes the session at 86.5 s.
The first two answers use source 1:0, the third 30:0, the refusal has no source.
The refusal is shown in the same colour as the answers; its text, not its colour,
marks it as UNKNOWN.
Przy 77,6 s status pokazuje cztery tury, a /exit kończy sesję przy 86,5 s.
Odmowa ma ten sam kolor co odpowiedzi; rozpoznaje się ją po treści.

## Shared boundaries of the private previews / Granice prywatnych podglądów

Both were recorded on 13 September 2026. The owner reports MINISFORUM AI X1 Pro
with 128 GB installed RAM; Linux reports Ryzen AI 9 HX 370, 12 cores / 24 logical
CPUs and 93.91 GiB visible RAM. Each loads a bank file of 885,776 bytes with
256 source nodes. File bytes are not process RSS. Startup RSS is 66.57 MiB in
the 60 s film and 66.61 MiB in the 90 s film. Available RAM and CPU frequencies
are momentary readings, not per-answer measurements or proof of GPU inactivity.

Oba filmy pochodzą z 13 września 2026 r. Właściciel deklaruje 128 GB RAM,
system widzi 93,91 GiB. Bank to 885 776 bajtów i 256 węzłów; RSS procesu jest
osobną wielkością. Taktowanie i dostępny RAM są odczytami chwilowymi, nie
dowodem niewykorzystania GPU.

Visible latencies above are rounded screen values; the historical guides retain
the longer reported values. Private raw field logs and engine are not bundled.
Timers include IPC, native processing, English realization and decoding, but
exclude typing, reading holds and final display rendering. The two recordings
are not a controlled performance comparison. The answers use bounded native
source rules/templates, not an unrestricted language model.

Podane tutaj czasy są zaokrąglone jak na ekranie. Dokładniejsze wartości podają
historyczne przewodniki; prywatne logi i silnik nie są dołączone. Zegar obejmuje
komunikację z procesem, przetwarzanie i formułowanie odpowiedzi, nie pauzy czytania.
To ograniczone reguły i szablony źródłowe, nie ogólny model językowy.

Neither preview establishes general understanding, persistent learning, network
operation, hardware-level memory computation or chat ORB/s. Public checkout
does not provide the private /chat. For reproduction, use Evidence Lab above.

Podglądy nie dowodzą ogólnego rozumienia, trwałego uczenia, działania sieciowego,
obliczeń na poziomie sprzętu pamięci ani przepustowości rozmowy w ORB/s.
Publiczne repozytorium nie zawiera prywatnego /chat.



## 4. Update, stale citation, restart, corrupted copy / Aktualizacja, nieaktualny cytat, restart, uszkodzona kopia — 102.15 s

[Video](GEL-EVIDENCE-UPDATE-R2.mp4) · silent / bez dźwięku · SHA256 3d3fce4f…a11f673

EN — times mark first appearance, read from all 375 visually distinct frames in an AI review (not a human sign-off):
- 0–5.8 s: empty centered terminal, then recorder header (scripted typing, real application output, no LLM, phrase retrieval, synthetic document); gel-evidence starts at 5.8 s.
- 8.4–21.2 s: add original.txt (106 bytes); find open the valve → HIT, bytes 67..105 (10.7 s); proof 1 → CITATION=PASS, “correspondence, not truth” (15.2 s); invented phrase → UNKNOWN (18.3 s); save → revision 1, pin 88e7194b… (21.2 s).
- 28.9–35.3 s: replace 1 revised.txt → previous citations invalidated; proof 1 → REFUSED NO_CURRENT_RESULT (31.4 s); the old phrase → UNKNOWN (35.3 s).
- 38.3–45.7 s: find keep the valve closed → HIT, bytes 60..105; proof → PASS with new hashes (42.8 s); save → revision 2, pin 8c0cf090… (45.7 s).
- 49.6–55.0 s: exit; recorder flips one byte in a COPY of revision 1 (corrupt.gelset); a new process starts.
- 59.5–79.1 s: load with pins → REOPEN=PASS revision 2 (59.5 s), then revision 1 (71.7 s); each returns its own quote and hashes.
- 83.8–92.1 s: the corrupted copy with the revision-1 pin → REFUSED COLLECTION_INTEGRITY; find/proof still answer (87.6 s, 92.1 s).
- 96.1–102.15 s: exit and recorder summary, held until the last frame.
- Why title=original.txt after replace: replace keeps document id 1 and its title; the text and document_sha256 (7a9ba8cb…) change.
- Why answers continue after the refusal: nothing new was loaded, so revision 1 from 71.7 s stays active. The corrupted file was a copy of that same revision, so the answers look the same; only the REFUSED line shows that the copy was rejected.
Not shown: understanding or truth of the text (the QUOTE line alone drops “Do not” from the line above; CONTEXT shows it), general AI conversation, hardware-level memory computation, throughput or percentiles, network isolation (a setup claim, not visible). ns values are single measurements on one 106-byte synthetic document on a loaded host (load average 6.3–7.3, 24 CPUs).

PL — czas oznacza pierwsze pojawienie się na ekranie; odczytano go ze wszystkich 375 różnych klatek w przeglądzie AI (nie przez człowieka):
- 0–5,8 s: pusty, wyśrodkowany terminal, potem nagłówek nagrywarki (skryptowane wpisywanie, prawdziwe wyjście aplikacji, bez LLM, wyszukiwanie fraz, dokument syntetyczny); start gel-evidence w 5,8 s.
- 8,4–21,2 s: dodanie original.txt (106 B); fraza „open the valve” → trafienie, bajty 67..105 (10,7 s); proof → CITATION=PASS, „zgodność, nie prawda” (15,2 s); fraza nieobecna → UNKNOWN (18,3 s); zapis → rewizja 1, pin 88e7194b… (21,2 s).
- 28,9–35,3 s: zamiana dokumentu → stare cytaty unieważnione; proof → odmowa NO_CURRENT_RESULT (31,4 s); stara fraza → UNKNOWN (35,3 s).
- 38,3–45,7 s: fraza „keep the valve closed” → trafienie, bajty 60..105; proof z nowymi hashami (42,8 s); zapis → rewizja 2, pin 8c0cf090… (45,7 s).
- 49,6–55,0 s: koniec procesu; nagrywarka zmienia jeden bajt w KOPII rewizji 1 (corrupt.gelset); start nowego procesu.
- 59,5–79,1 s: wczytanie z pinem → REOPEN=PASS rewizji 2 (59,5 s), potem rewizji 1 (71,7 s); każda zwraca własny cytat i hashe.
- 83,8–92,1 s: uszkodzona kopia z pinem rewizji 1 → odmowa COLLECTION_INTEGRITY; find/proof nadal odpowiadają (87,6 s, 92,1 s).
- 96,1–102,15 s: zamknięcie i podsumowanie nagrywarki, widoczne do ostatniej klatki.
- Dlaczego po zamianie tytuł to wciąż original.txt: replace zachowuje id 1 i tytuł, a zmienia treść i document_sha256 (7a9ba8cb…).
- Dlaczego po odmowie odpowiedzi są dalej: nic nowego nie wczytano, więc aktywna zostaje rewizja 1 z 71,7 s. Uszkodzony plik był kopią tej samej rewizji, dlatego odpowiedzi wyglądają tak samo; tylko linia REFUSED pokazuje, że kopię odrzucono.
Film nie pokazuje: rozumienia ani prawdziwości treści (sama linia QUOTE gubi „Do not” z poprzedniej linii; widać je w CONTEXT), ogólnej rozmowy AI, obliczeń na poziomie sprzętu pamięci, przepustowości ani percentyli, izolacji sieci (deklaracja konfiguracji, niewidoczna). Czasy ns to pojedyncze pomiary na jednym syntetycznym dokumencie o 106 bajtach, na obciążonym hoście (load average 6,3–7,3, 24 CPU).

