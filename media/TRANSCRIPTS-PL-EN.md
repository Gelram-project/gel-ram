# Descriptive transcripts / Transkrypcje opisowe

All three recordings are silent. These descriptions include the meaningful
commands, answers and limitations; they are not verbatim transcripts of every
cursor movement. Times below are approximate navigation points, checked using
frames sampled every five seconds and the ending frames. Sampling is not a
complete frame-by-frame visual/privacy review. Original videos are unchanged.

Wszystkie trzy nagrania są nieme. Opisy obejmują istotne polecenia, odpowiedzi
i ograniczenia, nie każdy ruch kursora. Znaczniki czasu są orientacyjne:
sprawdzono klatki co pięć sekund oraz zakończenia, nie każdą klatkę filmu.
Polskie tłumaczenia są pomocą dla czytelnika, nie dodatkowymi odpowiedziami GEL.

## 1. Public Evidence Lab / Publiczne narzędzie — 70.35 s

[Video](GEL-EVIDENCE-LAB-EN.mp4) · [Reproduction and provenance](EVIDENCE-LAB-GUIDE.md)

| Around / Około | English description | Polski opis |
|---|---|---|
| 5 s | A centered Linux terminal labels scripted typing, live output, offline CPU execution and phrase retrieval, NOT semantic conversation. The operator starts gel-evidence. | Wyśrodkowany terminal oznacza automatyczne wpisywanie i rzeczywiste wyniki. Uruchamiane jest wyszukiwanie fraz, nie rozmowa semantyczna. |
| 10 s | add memory.txt and add unicode.txt import 169 and 139 bytes. | Dodanie dwóch dokumentów: łącznie 308 bajtów tekstu. |
| 15–20 s | find ram is volatile returns HIT: “RAM is volatile.” Document 1, bytes 0..16. | Trafienie: „RAM jest pamięcią ulotną”. Cytat pochodzi z dokumentu 1, bajty 0..16. |
| 25 s | proof 1 returns CITATION=PASS with document and collection hashes. | Sprawdzenie powiązania cytatu ze źródłem, nie dowód prawdziwości treści. |
| 30 s | find imaginary evidence returns UNKNOWN. | Nieobecna fraza nie powoduje wygenerowania odpowiedzi. |
| 35 s | save checkpoint.gelset saves revision 2 and prints its SHA256 pin. | Jawny zapis snapshotu i wypisanie sumy kontrolnej do niezależnego zachowania. |
| 40–45 s | exit ends the first process; a new process is started. | Pierwszy proces kończy się; uruchamiany jest nowy. |
| 50 s | load with the retained pin returns REOPEN=PASS, revision 2. | Odtworzenie zapisanej kolekcji z kontrolą przypiętej sumy. |
| 55–60 s | The same find and proof return the same quote, byte range and hashes. | Ponowny odczyt zwraca ten sam cytat i jego identyfikację. |
| 65–70 s | exit closes the second process. Only explicitly saved snapshots persist. | Zakończenie. Trwałe są wyłącznie jawnie zapisane snapshoty. |

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

At 0–10 s the terminal identifies a private preview and displays hardware,
memory and diagnostic logging status. Around 15–20 s the operator enters /chat.
Responses remain on screen as subsequent questions are entered, without clear.

Na początku widać oznaczenie prywatnego podglądu, sprzęt, pamięć i włączone
logowanie diagnostyczne. Około 15–20 s następuje wejście do /chat. Kolejne
pytania nie czyszczą wcześniejszej rozmowy.

### Around 25–30 s: What do we know about Erich Honecker?

The displayed source-bound summary gives birth date “25 August 1912”, birthplace
“Neunkirchen”, death date “29 May 1994” and place of death “Santiago”. It explicitly
says these are four biographical fields, not a complete biography; a missing
death date does not establish that a person is alive; source consistency is not
independent verification of truth. Visible latency: **5.107 ms**, source 30:0.

Podsumowanie źródłowe podaje cztery pola: urodzenie 25 sierpnia 1912 r.
w Neunkirchen oraz śmierć 29 maja 1994 r. w Santiago. Zastrzega, że to nie pełna
biografia, brak daty śmierci nie dowodzi życia osoby, a zgodność ze źródłem nie
stanowi niezależnego potwierdzenia prawdy.

### Around 35–40 s: What do we know about Rami Malek?

The summary gives birth date “12 May 1981” and birthplace “Torrance”. It says no
unambiguous date or place of death was extracted from this sample, and repeats
the three limitations above. Visible latency: **16.974 ms**, source 1:0.

Odpowiedź podaje urodzenie 12 maja 1981 r. w Torrance. Data i miejsce śmierci
pozostają nierozstrzygnięte w tej próbce; zachowane są powyższe ograniczenia.

### Around 45–50 s: What do we know about John Lennon?

“I do not have an unambiguous source-bound answer. Please clarify the subject
and relation.” Visible latency: **23.707 ms**, UNKNOWN, no source anchor.

„Nie mam jednoznacznej odpowiedzi opartej na źródle. Doprecyzuj podmiot i relację”.
John Lennon exists in the catalog; this route failed to resolve a supported
summary. UNKNOWN does not prove that the bank lacks information about him.
Podmiot jest w katalogu, lecz ta ścieżka nie potrafiła złożyć jednoznacznego
podsumowania. Nie jest to dowód braku wiedzy w banku.

Around 55 s /status shows three completed turns. At the end /exit closes the
session: no data published, test conversation volatile. Diagnostic logging was
enabled; volatile session context does **not** mean that no diagnostic logs exist.

Około 55 s status pokazuje trzy tury. Zakończenie informuje o ulotności rozmowy
i braku publikacji. Nie oznacza to usunięcia ani braku logów diagnostycznych.

## 3. Private hardware and short answers / Sprzęt i krótkie odpowiedzi — 90 s

[Video](GEL-RAM-HARDWARE-EN-CENTERED-90s.mp4) · [Historical scope](DEMO.md)

Around 5–20 s the application displays hardware and memory. Around 25 s /chat
opens. The following answers are visible at the sampled navigation points:

Około 5–20 s aplikacja pokazuje sprzęt i pamięć, następnie otwiera /chat.

| Around / Około | Question and English answer | Polski opis | Visible latency |
|---|---|---|---:|
| 35–40 s | Where was Rami Malek born? — The source gives the birthplace as “Torrance”. | Miejsce urodzenia według źródła: Torrance. | 1.336 ms |
| 45–50 s | And when was he born? — The source gives the date of birth as “12 May 1981”. | Dopowiedzenie utrzymuje temat; data urodzenia: 12 maja 1981 r. | 0.233 ms |
| 55–65 s | Where did Erich Honecker die? — The source gives the place of death as “Santiago”. | Zmiana osoby; miejsce śmierci według źródła: Santiago. | 0.436 ms |
| 70–75 s | When did his father die? — I cannot resolve that relationship with the supported source rules. I will not guess. | Odmowa zgadywania: reguły nie rozstrzygają relacji z ojcem. | 0.140 ms |

Around 80–85 s /status shows four completed turns; /exit ends the session.
The first two answers use source 1:0, the third 30:0, the refusal has no source.
Około 80–85 s status pokazuje cztery tury. Następnie /exit kończy sesję.

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

Neither preview establishes general understanding, persistent learning, P2P
operation, physical DRAM-refresh computation or chat ORB/s. Public checkout
does not provide the private /chat. For reproduction, use Evidence Lab above.

Podglądy nie dowodzą ogólnego rozumienia, trwałego uczenia, działania P2P,
obliczeń fizycznym odświeżaniem DRAM ani przepustowości rozmowy w ORB/s.
Publiczne repozytorium nie zawiera prywatnego /chat.
