//! answer_or_abstain_v1 to v6 (docs/answer-or-abstain*): score any
//! system's answers with the published rules or the stricter ones, and check
//! that every published number is reproduced from the recorded answers.
//!
//! `check` re-scores the recorded answers of every system of every set,
//! renders the result tables and fails unless the README of the set contains
//! exactly those tables and the side-by-side document contains the rows it
//! publishes. It also verifies every source-passage hash and the set identity.
//! It re-scores recorded text; it does not re-run any system.
use std::{collections::HashMap, fmt::Write as _, fs, path::Path};

/// A recorded system: file name and the name printed in the tables.
type System = (&'static str, &'static str);

/// One published set: its folder, identity name, parts and questions per part.
struct SetDef {
    dir: &'static str,
    name: &'static str,
    parts: &'static [Set],
    count: usize,
    /// Whether docs/GEL-BESIDE-GROQ.md publishes rows re-scored from this set.
    side_by_side: bool,
    /// The systems whose answers the set records, in table order.
    systems: &'static [System],
    /// Whether the README also shows precision and language splits with Wilson intervals.
    precision: bool,
    /// Pairs of systems compared question by question (paired exact test).
    paired: &'static [(&'static str, &'static str)],
}

const V1: SetDef = SetDef {
    dir: "docs/answer-or-abstain",
    name: "answer_or_abstain_v1",
    parts: &[Set::WithAnswer, Set::NoAnswer],
    count: 80,
    side_by_side: true,
    systems: &SYSTEMS,
    precision: false,
    paired: &[],
};
const V2: SetDef = SetDef {
    dir: "docs/answer-or-abstain-v2",
    name: "answer_or_abstain_v2",
    parts: &[Set::WithAnswer],
    count: 394,
    side_by_side: false,
    systems: &SYSTEMS,
    precision: false,
    paired: &[],
};
const V3: SetDef = SetDef {
    dir: "docs/answer-or-abstain-v3",
    name: "answer_or_abstain_v3",
    parts: &[Set::WithAnswer],
    count: 979,
    side_by_side: false,
    systems: &[
        ("gel-ram", "GEL RAM"),
        ("gel-ram-first-run", "GEL RAM, first run"),
        ("gel-ram-v2-build", "GEL RAM, v1/v2 build"),
        ("gel-ram-prototype", "GEL RAM, prototype"),
        ("tantivy-bm25", "Tantivy BM25, threshold"),
        ("tantivy-bm25-top1", "Tantivy BM25, always top 1"),
        ("sqlite-fts5", "SQLite FTS5, threshold"),
        ("sqlite-fts5-top1", "SQLite FTS5, always top 1"),
    ],
    precision: true,
    paired: &[
        ("gel-ram", "gel-ram-first-run"),
        ("gel-ram", "tantivy-bm25"),
        ("gel-ram", "sqlite-fts5"),
    ],
};
const V4: SetDef = SetDef {
    dir: "docs/answer-or-abstain-v4",
    name: "answer_or_abstain_v4",
    parts: &[Set::WithAnswer],
    count: 985,
    side_by_side: false,
    systems: &[
        ("gel-ram", "GEL RAM"),
        ("gel-ram-precise", "GEL RAM, precise setting"),
        ("gel-ram-v3-build", "GEL RAM, v3 build"),
        ("tantivy-bm25", "Tantivy BM25, threshold"),
        ("tantivy-bm25-strict", "Tantivy BM25, strict threshold"),
        ("tantivy-bm25-top1", "Tantivy BM25, always top 1"),
        ("sqlite-fts5", "SQLite FTS5, threshold"),
        ("sqlite-fts5-strict", "SQLite FTS5, strict threshold"),
        ("sqlite-fts5-top1", "SQLite FTS5, always top 1"),
        ("tantivy-bm25-v3-bank", "Tantivy BM25, v3 bank"),
        ("sqlite-fts5-v3-bank", "SQLite FTS5, v3 bank"),
    ],
    precision: true,
    paired: &[
        ("gel-ram", "gel-ram-v3-build"),
        ("gel-ram", "tantivy-bm25"),
        ("gel-ram", "sqlite-fts5"),
        ("gel-ram-precise", "tantivy-bm25-strict"),
        ("gel-ram-precise", "sqlite-fts5-strict"),
    ],
};
const V5: SetDef = SetDef {
    dir: "docs/answer-or-abstain-v5",
    name: "answer_or_abstain_v5",
    parts: &[Set::WithAnswer],
    count: 987,
    side_by_side: false,
    systems: &[
        ("gel-ram", "GEL RAM"),
        ("gel-ram-precise", "GEL RAM, precise setting"),
        ("gel-ram-candidate", "GEL RAM, candidate change"),
        (
            "gel-ram-candidate-precise",
            "GEL RAM, candidate change, precise setting",
        ),
        ("tantivy-bm25", "Tantivy BM25, threshold"),
        ("tantivy-bm25-strict", "Tantivy BM25, strict threshold"),
        ("tantivy-bm25-top1", "Tantivy BM25, always top 1"),
        ("sqlite-fts5", "SQLite FTS5, threshold"),
        ("sqlite-fts5-strict", "SQLite FTS5, strict threshold"),
        ("sqlite-fts5-top1", "SQLite FTS5, always top 1"),
    ],
    precision: true,
    paired: &[
        ("gel-ram-candidate", "gel-ram"),
        ("gel-ram-candidate-precise", "gel-ram-precise"),
        ("gel-ram", "tantivy-bm25"),
        ("gel-ram", "sqlite-fts5"),
        ("gel-ram-precise", "tantivy-bm25-strict"),
        ("gel-ram-precise", "sqlite-fts5-strict"),
    ],
};
const V6: SetDef = SetDef {
    dir: "docs/answer-or-abstain-v6",
    name: "answer_or_abstain_v6",
    parts: &[Set::WithAnswer],
    count: 990,
    side_by_side: false,
    systems: &[
        ("gel-ram", "GEL RAM"),
        ("gel-ram-precise", "GEL RAM, precise setting"),
        ("tantivy-bm25", "Tantivy BM25, threshold"),
        ("tantivy-bm25-strict", "Tantivy BM25, strict threshold"),
        ("tantivy-bm25-top1", "Tantivy BM25, always top 1"),
        ("sqlite-fts5", "SQLite FTS5, threshold"),
        ("sqlite-fts5-strict", "SQLite FTS5, strict threshold"),
        ("sqlite-fts5-top1", "SQLite FTS5, always top 1"),
    ],
    precision: true,
    paired: &[
        ("gel-ram", "tantivy-bm25"),
        ("gel-ram", "sqlite-fts5"),
        ("gel-ram-precise", "tantivy-bm25-strict"),
        ("gel-ram-precise", "sqlite-fts5-strict"),
    ],
};
const SETS: [&SetDef; 6] = [&V1, &V2, &V3, &V4, &V5, &V6];
const SYSTEMS: [System; 4] = [
    ("gel-ram", "GEL RAM"),
    ("gpt-oss-120b", "GPT-OSS-120B"),
    ("gpt-oss-20b", "GPT-OSS-20B"),
    ("qwen3.8-27b", "Qwen3.8-27B"),
];
const BEGIN: &str = "<!-- ANSWER-BENCH-RESULTS-BEGIN -->\n";
const END: &str = "<!-- ANSWER-BENCH-RESULTS-END -->\n";
/// Whole words that reject a premise (after normalization "didn't" is "didn t").
const NEGATIONS: [&str; 23] = [
    "nie", "nigdy", "zaden", "brak", "nikt", "nic", "niczego", "no", "not", "never", "none",
    "nobody", "nothing", "didn", "wasn", "weren", "isn", "doesn", "hasn", "hadn", "cannot",
    "neither", "nor",
];
/// Word beginnings that reject a premise or call the subject fictional.
const NEGATION_PREFIXES: [&str; 7] = [
    "zadn",
    "nieistn",
    "fikcyj",
    "fiction",
    "fictitious",
    "nonexist",
    "falszyw",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Set {
    WithAnswer,
    NoAnswer,
}

/// Published: the rules frozen before the recorded runs. Strict: recommended for new runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rules {
    Published,
    Strict,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Correct,
    Wrong,
    Unknown,
    Rejected,
    Answered,
    Review,
    Error,
}

impl Verdict {
    fn name(self) -> &'static str {
        match self {
            Verdict::Correct => "CORRECT",
            Verdict::Wrong => "WRONG",
            Verdict::Unknown => "UNKNOWN",
            Verdict::Rejected => "REJECTED",
            Verdict::Answered => "ANSWERED",
            Verdict::Review => "REVIEW",
            Verdict::Error => "ERROR",
        }
    }
}

struct Question {
    /// Groups of (spelling, is_stem); a stem is marked with a trailing `*` in the file.
    accepted: Vec<Vec<(String, bool)>>,
    invented: bool,
    /// Language of the question: `pl` or `en`.
    lang: String,
}

struct Answer {
    text: String,
    passage: bool,
}

/// correct, wrong, unknown, rejected, answered (invented), answered (false premise), review, errors.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Totals([usize; 8]);

impl Totals {
    fn add(&mut self, v: Verdict, invented: bool) {
        let i = match v {
            Verdict::Correct => 0,
            Verdict::Wrong => 1,
            Verdict::Unknown => 2,
            Verdict::Rejected => 3,
            Verdict::Answered if invented => 4,
            Verdict::Answered => 5,
            Verdict::Review => 6,
            Verdict::Error => 7,
        };
        self.0[i] += 1;
    }
    fn describe(self, set: Set) -> String {
        let t = self.0;
        match set {
            Set::WithAnswer => format!(
                "answered={} correct={} wrong={} unknown={} errors={}",
                t[0] + t[1],
                t[0],
                t[1],
                t[2],
                t[7]
            ),
            Set::NoAnswer => format!(
                "answered_anyway={} (invented={} false_premise={}) unknown={} rejected={} review={} errors={}",
                t[4] + t[5],
                t[4],
                t[5],
                t[2],
                t[3],
                t[6],
                t[7]
            ),
        }
    }
}

fn fold(c: char) -> &'static str {
    match c {
        'ą' | 'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ă' => "a",
        'ć' | 'ç' | 'č' => "c",
        'ę' | 'é' | 'è' | 'ê' | 'ë' | 'ě' => "e",
        'í' | 'ì' | 'î' | 'ï' => "i",
        'ł' => "l",
        'ń' | 'ñ' => "n",
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ő' => "o",
        'ś' | 'š' | 'ș' | 'ş' => "s",
        'ź' | 'ż' | 'ž' => "z",
        'ú' | 'ù' | 'û' | 'ü' | 'ů' | 'ű' => "u",
        'ð' => "d",
        'þ' => "th",
        'ß' => "ss",
        'ț' | 'ţ' => "t",
        'ý' => "y",
        'ř' => "r",
        _ => "",
    }
}

/// Lower case, common diacritics to ASCII, everything but letters and digits to one space.
fn normalize(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        let folded = fold(c);
        if !folded.is_empty() {
            out.push_str(folded)
        } else if c.is_alphanumeric() {
            out.push(c)
        } else {
            out.push(' ')
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A spelling always starts a word. It must also end the word when it ends in a
/// digit, and under the strict rules unless it is marked as a stem.
fn alias_hit(normalized: &str, alias: &str, stem: bool, rules: Rules) -> bool {
    let padded = format!(" {normalized} ");
    let needle = format!(" {alias}");
    let digit = alias.chars().last().is_some_and(|c| c.is_ascii_digit());
    let whole = digit || (rules == Rules::Strict && !stem);
    padded
        .match_indices(&needle)
        .any(|(i, _)| !whole || padded[i + needle.len()..].starts_with(' '))
}

fn abstains(text: &str, rules: Rules) -> bool {
    let n = normalize(text);
    match rules {
        Rules::Published => n.split(' ').next() == Some("unknown"),
        Rules::Strict => n == "unknown",
    }
}

fn negates(text: &str) -> bool {
    normalize(text)
        .split(' ')
        .any(|w| NEGATIONS.contains(&w) || NEGATION_PREFIXES.iter().any(|p| w.starts_with(p)))
}

fn verdict(set: Set, rules: Rules, q: &Question, answer: Option<&Answer>) -> Verdict {
    let Some(a) = answer.filter(|a| !a.text.trim().is_empty()) else {
        return Verdict::Error;
    };
    if abstains(&a.text, rules) {
        return Verdict::Unknown;
    }
    match set {
        Set::WithAnswer => {
            let n = normalize(&a.text);
            let hit = |g: &Vec<(String, bool)>| {
                g.iter().any(|(al, stem)| alias_hit(&n, al, *stem, rules))
            };
            if q.accepted.iter().all(hit) {
                Verdict::Correct
            } else {
                Verdict::Wrong
            }
        }
        // A returned source passage is an answer: its words are not read as a rejection.
        Set::NoAnswer if a.passage || !negates(&a.text) => Verdict::Answered,
        // A negation may reject the premise or sit next to an invented fact: strict rules leave it to review.
        Set::NoAnswer => match rules {
            Rules::Published => Verdict::Rejected,
            Rules::Strict => Verdict::Review,
        },
    }
}

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split('\t').collect())
}

fn question_number(field: &str, seen: &mut [bool]) -> Result<usize, String> {
    let nr: usize = field
        .parse()
        .map_err(|_| format!("bad question number {field}"))?;
    if !(1..=seen.len()).contains(&nr) || std::mem::replace(&mut seen[nr - 1], true) {
        return Err(format!("question number {nr} out of range or repeated"));
    }
    Ok(nr)
}

fn sha256(text: &str) -> String {
    gel_source::hex(&gel_source::digest(text.as_bytes()))
}

/// Source fields: title, url, dump, entry, sha256, passage — the hash must match the passage.
fn check_source(nr: usize, f: &[&str]) -> Result<(), String> {
    if f.iter().any(|x| x.is_empty()) || sha256(f[5]) != f[4] {
        return Err(format!(
            "question {nr}: missing source field or passage hash mismatch"
        ));
    }
    Ok(())
}

fn questions(set: Set, text: &str, count: usize) -> Result<Vec<Question>, String> {
    let mut out: Vec<Option<Question>> = (0..count).map(|_| None).collect();
    let mut seen = vec![false; count];
    for f in rows(text) {
        let nr = question_number(f[0], &mut seen)?;
        let lang = match f.get(1) {
            Some(l @ (&"pl" | &"en")) => l.to_string(),
            _ => return Err(format!("question {nr}: language must be pl or en")),
        };
        let q = match set {
            Set::WithAnswer if f.len() == 11 && !f[4].is_empty() => {
                check_source(nr, &f[5..11])?;
                let accepted = f[4]
                    .split(';')
                    .map(|g| {
                        g.split('|')
                            .map(|a| match a.strip_suffix('*') {
                                Some(stem) => (stem.to_owned(), true),
                                None => (a.to_owned(), false),
                            })
                            .collect()
                    })
                    .collect();
                Question {
                    accepted,
                    invented: false,
                    lang,
                }
            }
            Set::NoAnswer if f.len() == 12 && f[2] == "invented" => {
                if f[5].is_empty() || f[6..].iter().any(|x| !x.is_empty()) {
                    return Err(format!(
                        "question {nr}: invented subject needs checked names only"
                    ));
                }
                Question {
                    accepted: Vec::new(),
                    invented: true,
                    lang,
                }
            }
            Set::NoAnswer if f.len() == 12 && f[2] == "false-premise" => {
                check_source(nr, &f[6..12])?;
                Question {
                    accepted: Vec::new(),
                    invented: false,
                    lang,
                }
            }
            _ => return Err(format!("question {nr}: unexpected fields")),
        };
        out[nr - 1] = Some(q);
    }
    out.into_iter()
        .enumerate()
        .map(|(i, q)| q.ok_or(format!("question {} missing", i + 1)))
        .collect()
}

/// Exactly `nr<TAB>answer`, or `nr<TAB>answer<TAB>passage`.
fn answers(text: &str, count: usize) -> Result<HashMap<usize, Answer>, String> {
    let mut out = HashMap::new();
    let mut seen = vec![false; count];
    for f in rows(text) {
        let nr = question_number(f[0], &mut seen)?;
        let passage = match f.len() {
            2 => false,
            3 if f[2] == "passage" => true,
            _ => {
                return Err(format!(
                    "answer {nr}: expected nr, answer and optionally \"passage\""
                ))
            }
        };
        out.insert(
            nr,
            Answer {
                text: f[1].to_owned(),
                passage,
            },
        );
    }
    Ok(out)
}

/// Exactly `system<TAB>nr<TAB>verdict<TAB>reason`; known systems, verdicts allowed for the set, no repeats.
fn reviews(
    set: Set,
    systems: &[System],
    text: &str,
    count: usize,
) -> Result<HashMap<(String, usize), Verdict>, String> {
    let mut out = HashMap::new();
    for f in rows(text) {
        let [system, nr, verdict, reason] = f.as_slice() else {
            return Err("review: expected system, nr, verdict and reason".into());
        };
        if !systems.iter().any(|(id, _)| id == system) || reason.trim().is_empty() {
            return Err(format!("review: unknown system {system} or empty reason"));
        }
        let nr: usize = nr.parse().map_err(|_| format!("review: bad number {nr}"))?;
        if !(1..=count).contains(&nr) {
            return Err(format!("review: number {nr} out of range"));
        }
        let v = match (set, *verdict) {
            (Set::WithAnswer, "CORRECT") => Verdict::Correct,
            (Set::WithAnswer, "WRONG") => Verdict::Wrong,
            (Set::NoAnswer, "REJECTED") => Verdict::Rejected,
            (Set::NoAnswer, "ANSWERED") => Verdict::Answered,
            _ => {
                return Err(format!(
                    "review: verdict {verdict} not allowed for this set"
                ))
            }
        };
        if out.insert((system.to_string(), nr), v).is_some() {
            return Err(format!("review: {system} {nr} repeated"));
        }
    }
    Ok(out)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn set_name(set: Set) -> &'static str {
    match set {
        Set::WithAnswer => "with-answer",
        Set::NoAnswer => "no-answer",
    }
}

fn question_file(set: Set) -> String {
    format!("{}-questions.txt", set_name(set))
}

fn score(set: Set, rules: Rules, qs: &[Question], given: &HashMap<usize, Answer>) -> Vec<Verdict> {
    qs.iter()
        .enumerate()
        .map(|(i, q)| verdict(set, rules, q, given.get(&(i + 1))))
        .collect()
}

fn totals(qs: &[Question], verdicts: &[Verdict]) -> Totals {
    let mut t = Totals::default();
    for (q, v) in qs.iter().zip(verdicts) {
        t.add(*v, q.invented);
    }
    t
}

/// One system scored on one part of a set under one rule set.
struct Scored {
    set: Set,
    rules: Rules,
    system: &'static str,
    auto: Totals,
    reviewed: Totals,
    /// Verdicts after the manual review, one per question.
    verdicts: Vec<Verdict>,
    /// Totals after the review for the Polish and for the English questions.
    by_lang: [Totals; 2],
}

type Results = Vec<Scored>;

fn results(root: &Path, def: &SetDef) -> Result<Results, String> {
    let mut out = Vec::new();
    for &set in def.parts {
        let dir = root.join(def.dir);
        let qs = questions(set, &read(&dir.join(question_file(set)))?, def.count)?;
        let rec = dir.join("recorded").join(set_name(set));
        let review = reviews(set, def.systems, &read(&rec.join("review.txt"))?, def.count)?;
        for &(system, _) in def.systems {
            let given = answers(&read(&rec.join(format!("{system}.txt")))?, def.count)?;
            for rules in [Rules::Published, Rules::Strict] {
                let auto = score(set, rules, &qs, &given);
                let mut reviewed = auto.clone();
                for (i, v) in reviewed.iter_mut().enumerate() {
                    if let Some(r) = review.get(&(system.to_owned(), i + 1)) {
                        *v = *r;
                    }
                }
                let mut by_lang = [Totals::default(); 2];
                for (q, v) in qs.iter().zip(&reviewed) {
                    by_lang[usize::from(q.lang == "en")].add(*v, q.invented);
                }
                out.push(Scored {
                    set,
                    rules,
                    system,
                    auto: totals(&qs, &auto),
                    reviewed: totals(&qs, &reviewed),
                    verdicts: reviewed,
                    by_lang,
                });
            }
        }
    }
    Ok(out)
}

fn cell(auto: usize, reviewed: usize) -> String {
    if auto == reviewed {
        reviewed.to_string()
    } else {
        format!("{reviewed} ({auto})")
    }
}

fn display(def: &SetDef, system: &str) -> &'static str {
    def.systems
        .iter()
        .find(|(id, _)| *id == system)
        .map(|(_, name)| *name)
        .unwrap_or("?")
}

fn rules_name(rules: Rules) -> &'static str {
    match rules {
        Rules::Published => "published",
        Rules::Strict => "strict",
    }
}

/// The README block: every part, both rule sets, reviewed totals with automatic ones in
/// brackets; for the sets that ask for them also precision and the paired comparison.
fn render(def: &SetDef, results: &Results) -> String {
    let mut s = String::from(BEGIN);
    s.push_str("| With an answer | Rules | Answered | Correct | Wrong | UNKNOWN | Errors |\n|---|---|---:|---:|---:|---:|---:|\n");
    for r in results.iter().filter(|r| r.set == Set::WithAnswer) {
        let (a, v) = (r.auto.0, r.reviewed.0);
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} |",
            display(def, r.system),
            rules_name(r.rules),
            cell(a[0] + a[1], v[0] + v[1]),
            cell(a[0], v[0]),
            cell(a[1], v[1]),
            cell(a[2], v[2]),
            cell(a[7], v[7])
        );
    }
    if results.iter().any(|r| r.set == Set::NoAnswer) {
        s.push_str("\n| Without an answer | Rules | Answered anyway: invented / false premise | UNKNOWN | Rejected the premise | Needs review | Errors |\n|---|---|---:|---:|---:|---:|---:|\n");
        for r in results.iter().filter(|r| r.set == Set::NoAnswer) {
            let (a, v) = (r.auto.0, r.reviewed.0);
            let _ = writeln!(
                s,
                "| {} | {} | {} / {} | {} | {} | {} | {} |",
                display(def, r.system),
                rules_name(r.rules),
                cell(a[4], v[4]),
                cell(a[5], v[5]),
                cell(a[2], v[2]),
                cell(a[3], v[3]),
                cell(a[6], v[6]),
                cell(a[7], v[7])
            );
        }
    }
    if def.precision {
        s.push_str(&precision_table(def, results));
    }
    if !def.paired.is_empty() {
        s.push_str(&paired_table(def, results));
    }
    s.push_str(END);
    s
}

/// Published rules after the review: precision and wrong answers among all questions, each
/// with its 95% Wilson interval, and correct / answered for each language.
fn precision_table(def: &SetDef, results: &Results) -> String {
    let mut s = String::from("\n| Published rules, after the review | Precision (95% Wilson) | Wrong among all questions (95% Wilson) | Polish: correct / answered | English: correct / answered |\n|---|---|---|---:|---:|\n");
    for r in results
        .iter()
        .filter(|r| r.set == Set::WithAnswer && r.rules == Rules::Published)
    {
        let t = r.reviewed.0;
        let [pl, en] = r
            .by_lang
            .map(|l| format!("{} / {}", l.0[0], l.0[0] + l.0[1]));
        let _ = writeln!(
            s,
            "| {} | {} | {} | {pl} | {en} |",
            display(def, r.system),
            share(t[0], t[0] + t[1]),
            share(t[1], t.iter().sum())
        );
    }
    s
}

/// `k` of `n` as a percentage with its 95% Wilson interval.
fn share(k: usize, n: usize) -> String {
    if n == 0 {
        return "—".into();
    }
    let (lo, hi) = wilson(k, n);
    format!(
        "{:.1}% ({:.1}–{:.1}%)",
        100.0 * k as f64 / n as f64,
        100.0 * lo,
        100.0 * hi
    )
}

/// Each listed pair of systems, question by question (published rules, after the review): how
/// many questions only one of the two answered correctly, and how many only one answered
/// wrongly, with the two-sided exact sign test on those questions.
fn paired_table(def: &SetDef, results: &Results) -> String {
    let published = |system: &str| {
        results
            .iter()
            .find(|r| r.set == Set::WithAnswer && r.rules == Rules::Published && r.system == system)
    };
    let mut s = String::from("\n| The same questions: first beside second | Correct only in the first / only in the second | Wrong only in the first / only in the second |\n|---|---:|---:|\n");
    for (first, second) in def.paired {
        let (Some(a), Some(b)) = (published(first), published(second)) else {
            continue;
        };
        let only = |v: Verdict| {
            let pairs = a.verdicts.iter().zip(&b.verdicts);
            let mine = pairs.clone().filter(|(x, y)| **x == v && **y != v).count();
            let theirs = pairs.filter(|(x, y)| **x != v && **y == v).count();
            format!("{mine} / {theirs} ({})", p_text(sign_test(mine, theirs)))
        };
        let _ = writeln!(
            s,
            "| {} beside {} | {} | {} |",
            display(def, first),
            display(def, second),
            only(Verdict::Correct),
            only(Verdict::Wrong)
        );
    }
    s
}

/// Two-sided exact sign test (McNemar's exact test) of `a` against `b` discordant questions.
fn sign_test(a: usize, b: usize) -> f64 {
    let n = a + b;
    let mut ln_choose = 0.0f64;
    let mut tail = 0.0f64;
    for i in 0..=a.min(b) {
        if i > 0 {
            ln_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (ln_choose - n as f64 * std::f64::consts::LN_2).exp();
    }
    (2.0 * tail).min(1.0)
}

fn p_text(p: f64) -> String {
    if p < 0.001 {
        "p < 0.001".into()
    } else {
        format!("p = {p:.3}")
    }
}

fn wilson(k: usize, n: usize) -> (f64, f64) {
    let (z, n, p) = (1.96f64, n as f64, k as f64 / n as f64);
    let d = 1.0 + z * z / n;
    let c = p + z * z / (2.0 * n);
    let r = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    ((c - r) / d, (c + r) / d)
}

/// Rows of the summary table in docs/GEL-BESIDE-GROQ.md (published rules, after the review).
fn side_by_side_rows(def: &SetDef, results: &Results) -> Vec<String> {
    results
        .iter()
        .filter(|r| r.set == Set::WithAnswer && r.rules == Rules::Published)
        .map(|r| {
            let t = r.reviewed.0;
            let answered = t[0] + t[1];
            let (lo, hi) = wilson(t[0], answered);
            let conditions = if r.system == "gel-ram" {
                "local bank, answers with a source passage"
            } else {
                "Groq API, closed book"
            };
            format!(
                "| {} | {conditions} | {answered} | {} | {} | {} | {} | {}/{answered} ({:.0}–{:.0}%) |",
                display(def, r.system),
                t[0],
                t[1],
                t[2],
                t[7],
                t[0],
                100.0 * lo,
                100.0 * hi
            )
        })
        .collect()
}

/// SHA-256 over the names and hashes of every data file of the set, in a fixed order.
fn set_identity(root: &Path, def: &SetDef) -> Result<String, String> {
    let dir = root.join(def.dir);
    let mut names: Vec<String> = def.parts.iter().map(|&set| question_file(set)).collect();
    for &set in def.parts {
        for (system, _) in def.systems {
            names.push(format!("recorded/{}/{system}.txt", set_name(set)));
        }
        names.push(format!("recorded/{}/review.txt", set_name(set)));
    }
    let mut manifest = String::new();
    for name in names {
        let bytes = fs::read(dir.join(&name)).map_err(|e| format!("{name}: {e}"))?;
        let _ = writeln!(
            manifest,
            "{}  {name}",
            gel_source::hex(&gel_source::digest(&bytes))
        );
    }
    Ok(sha256(&manifest))
}

/// The label printed for a part: the plain part name for v1, `set:part` for later sets.
fn label(def: &SetDef, set: Set) -> String {
    if def.name == V1.name {
        set_name(set).to_owned()
    } else {
        format!("{}:{}", def.name, set_name(set))
    }
}

pub fn check(root: &Path) -> Result<(), String> {
    for def in SETS {
        let results = results(root, def)?;
        for r in &results {
            println!(
                "ANSWER_BENCH set={} rules={} system={} {} reviewed: {}",
                label(def, r.set),
                rules_name(r.rules),
                r.system,
                r.auto.describe(r.set),
                r.reviewed.describe(r.set)
            );
        }
        let readme = read(&root.join(def.dir).join("README.md"))?;
        let block = readme
            .split_once(BEGIN)
            .and_then(|(_, rest)| rest.split_once(END))
            .map(|(inner, _)| format!("{BEGIN}{inner}{END}"))
            .ok_or("README of the set has no results block")?;
        if block != render(def, &results) {
            return Err(format!(
                "{}: README results differ from the re-scored recorded answers; run `answer-bench tables`",
                def.name
            ));
        }
        if def.side_by_side {
            let side = read(&root.join("docs/GEL-BESIDE-GROQ.md"))?;
            for row in side_by_side_rows(def, &results) {
                if !side.contains(&row) {
                    return Err(format!(
                        "docs/GEL-BESIDE-GROQ.md lacks the re-scored row: {row}"
                    ));
                }
            }
        }
        let identity = set_identity(root, def)?;
        if !readme.contains(&format!("`{}` · SHA-256 `{identity}`", def.name)) {
            return Err(format!(
                "README does not state the set identity {} {identity}",
                def.name
            ));
        }
        println!("ANSWER_BENCH_SET={} sha256={identity}", def.name);
    }
    check_beside(root)?;
    println!("ANSWER_BENCH=PASS recorded answers reproduce the published tables");
    Ok(())
}

/// A set definition by name: the full identity (`answer_or_abstain_v2`) or its suffix (`v2`).
fn find_set(name: &str) -> Result<&'static SetDef, String> {
    SETS.iter()
        .copied()
        .find(|d| d.name == name || d.name.ends_with(&format!("_{name}")))
        .ok_or_else(|| format!("unknown set {name}"))
}

/// `with-answer` and `no-answer` name the parts of v1; `v2:with-answer` a part of another set.
fn find_part(spec: &str) -> Result<(&'static SetDef, Set), String> {
    let (def, part) = match spec.split_once(':') {
        Some((name, part)) => (find_set(name)?, part),
        None => (&V1, spec),
    };
    let set = match part {
        "with-answer" => Set::WithAnswer,
        "no-answer" => Set::NoAnswer,
        other => return Err(format!("unknown set part {other}")),
    };
    if !def.parts.contains(&set) {
        return Err(format!("{} has no {part} part", def.name));
    }
    Ok((def, set))
}

/// The drawn sample of v6 put to three language models: docs/GEL-BESIDE-ENGINES-AND-LLMS-V6.md.
const BESIDE_DIR: &str = "docs/answer-or-abstain-v6/beside-llm";
const BESIDE_PAGE: &str = "docs/GEL-BESIDE-ENGINES-AND-LLMS-V6.md";
const BESIDE_COUNT: usize = 400;
/// Systems of the sample in table order: id, name on the page, conditions. The first six are
/// re-scored v6 recordings; the last three are the models' answers in the sample folder.
const BESIDE_SYSTEMS: [(&str, &str, &str); 9] = [
    (
        "gel-ram",
        "GEL RAM, balanced",
        "local bank, one stored passage",
    ),
    (
        "tantivy-bm25",
        "Tantivy (BM25), threshold",
        "same bank, threshold",
    ),
    (
        "sqlite-fts5",
        "SQLite FTS5 (BM25), threshold",
        "same bank, threshold",
    ),
    (
        "gel-ram-precise",
        "GEL RAM, precise",
        "the same, stricter setting",
    ),
    (
        "tantivy-bm25-strict",
        "Tantivy (BM25), strict",
        "same bank, strict threshold",
    ),
    (
        "sqlite-fts5-strict",
        "SQLite FTS5 (BM25), strict",
        "same bank, strict threshold",
    ),
    ("gpt-oss-120b", "GPT-OSS-120B", "Groq API, closed book"),
    ("gpt-oss-20b", "GPT-OSS-20B", "Groq API, closed book"),
    ("qwen3.8-27b", "Qwen3.8-27B", "Groq API, closed book"),
];
/// Pairs compared question by question on the page: first, second, row label.
const BESIDE_PAIRS: [(&str, &str, &str); 7] = [
    (
        "gel-ram",
        "tantivy-bm25",
        "balanced / Tantivy (BM25), threshold",
    ),
    (
        "gel-ram",
        "sqlite-fts5",
        "balanced / SQLite FTS5 (BM25), threshold",
    ),
    (
        "gel-ram-precise",
        "tantivy-bm25-strict",
        "precise / Tantivy (BM25), strict",
    ),
    (
        "gel-ram-precise",
        "sqlite-fts5-strict",
        "precise / SQLite FTS5 (BM25), strict",
    ),
    ("gel-ram", "gpt-oss-120b", "balanced / GPT-OSS-120B"),
    ("gel-ram", "gpt-oss-20b", "balanced / GPT-OSS-20B"),
    ("gel-ram", "qwen3.8-27b", "balanced / Qwen3.8-27B"),
];

/// The sample's questions in file order with their language, and every system's verdict on
/// each (published rules, after the review: v6's for the recorded systems, the sample's own
/// for the models).
struct Beside {
    sample: Vec<(usize, String)>,
    verdicts: Vec<Vec<Verdict>>,
}

fn beside(root: &Path) -> Result<Beside, String> {
    let dir = root.join(BESIDE_DIR);
    let mut seen = vec![false; V6.count];
    let numbers = read(&dir.join("sample-400.txt"))?;
    let numbers: Vec<usize> = rows(&numbers)
        .map(|f| question_number(f[0], &mut seen))
        .collect::<Result<_, _>>()?;
    if numbers.len() != BESIDE_COUNT {
        return Err(format!(
            "{BESIDE_DIR}/sample-400.txt: expected {BESIDE_COUNT} numbers"
        ));
    }
    let qs = questions(
        Set::WithAnswer,
        &read(&root.join(V6.dir).join(question_file(Set::WithAnswer)))?,
        V6.count,
    )?;
    let v6 = results(root, &V6)?;
    let models = &SYSTEMS[1..];
    let review = reviews(
        Set::WithAnswer,
        models,
        &read(&dir.join("recorded/review.txt"))?,
        V6.count,
    )?;
    if let Some((system, nr)) = review.keys().find(|(_, nr)| !numbers.contains(nr)) {
        return Err(format!(
            "{BESIDE_DIR}: review of {system} {nr} outside the sample"
        ));
    }
    let mut verdicts = Vec::new();
    for (id, _, _) in BESIDE_SYSTEMS {
        let all = if models.iter().any(|(m, _)| *m == id) {
            let given = answers(&read(&dir.join(format!("recorded/{id}.txt")))?, V6.count)?;
            if given.len() != BESIDE_COUNT || numbers.iter().any(|nr| !given.contains_key(nr)) {
                return Err(format!(
                    "{BESIDE_DIR}/recorded/{id}.txt: expected exactly the sample's questions"
                ));
            }
            let mut v = score(Set::WithAnswer, Rules::Published, &qs, &given);
            for ((system, nr), r) in &review {
                if system == id {
                    v[nr - 1] = *r;
                }
            }
            v
        } else {
            v6.iter()
                .find(|r| r.set == Set::WithAnswer && r.rules == Rules::Published && r.system == id)
                .ok_or(format!("v6 has no recorded system {id}"))?
                .verdicts
                .clone()
        };
        verdicts.push(numbers.iter().map(|nr| all[nr - 1]).collect());
    }
    let sample = numbers
        .iter()
        .map(|&nr| (nr, qs[nr - 1].lang.clone()))
        .collect();
    Ok(Beside { sample, verdicts })
}

fn letter(v: Verdict) -> char {
    match v {
        Verdict::Correct => 'C',
        Verdict::Wrong => 'W',
        Verdict::Unknown => 'U',
        _ => 'E',
    }
}

/// The per-question verdict file the README grid is drawn from.
fn beside_verdicts(b: &Beside) -> String {
    let mut s = String::from("# Verdicts on the 400 questions of sample-400.txt (published rules, after the review: set v6's review for GEL RAM and the engines, recorded/review.txt for the models). C correct, W wrong, U UNKNOWN, E error. Written by `answer-bench beside`; `answer-bench check` fails unless this file matches the recorded answers.\n# nr\tlang");
    for (id, _, _) in BESIDE_SYSTEMS {
        s.push('\t');
        s.push_str(id);
    }
    s.push('\n');
    for (i, (nr, lang)) in b.sample.iter().enumerate() {
        let _ = write!(s, "{nr}\t{lang}");
        for v in &b.verdicts {
            s.push('\t');
            s.push(letter(v[i]));
        }
        s.push('\n');
    }
    s
}

/// The rows of the page's three tables: results, the language split and the pairs.
fn beside_rows(b: &Beside) -> Vec<String> {
    let mut out = Vec::new();
    let count = |v: &[Verdict], lang: Option<&str>, x: Verdict| {
        v.iter()
            .zip(&b.sample)
            .filter(|(y, (_, l))| **y == x && lang.is_none_or(|lang| l == lang))
            .count()
    };
    for ((_, name, conditions), v) in BESIDE_SYSTEMS.iter().zip(&b.verdicts) {
        let (c, w, u) = (
            count(v, None, Verdict::Correct),
            count(v, None, Verdict::Wrong),
            count(v, None, Verdict::Unknown),
        );
        out.push(format!(
            "| {name} | {conditions} | {} | {c} | {w} | {u} | {} |",
            c + w,
            share(c, c + w)
        ));
        let [pl, en] = ["pl", "en"].map(|l| {
            format!(
                "{} / {} / {}",
                count(v, Some(l), Verdict::Correct),
                count(v, Some(l), Verdict::Wrong),
                count(v, Some(l), Verdict::Unknown)
            )
        });
        out.push(format!("| {name} | {pl} | {en} |"));
    }
    let of = |id: &str| {
        BESIDE_SYSTEMS
            .iter()
            .position(|(s, _, _)| *s == id)
            .map(|i| &b.verdicts[i])
    };
    for (first, second, label) in BESIDE_PAIRS {
        let (Some(a), Some(c)) = (of(first), of(second)) else {
            continue;
        };
        let only = |v: Verdict| {
            let mine = a
                .iter()
                .zip(c)
                .filter(|(x, y)| **x == v && **y != v)
                .count();
            let theirs = a
                .iter()
                .zip(c)
                .filter(|(x, y)| **x != v && **y == v)
                .count();
            format!("{mine} / {theirs} ({})", p_text(sign_test(mine, theirs)))
        };
        out.push(format!(
            "| {label} | {} | {} |",
            only(Verdict::Correct),
            only(Verdict::Wrong)
        ));
    }
    out
}

/// Fails unless the verdict file and the page's table rows follow from the recorded answers.
fn check_beside(root: &Path) -> Result<(), String> {
    let b = beside(root)?;
    if b.verdicts.iter().flatten().any(|v| *v == Verdict::Error) {
        return Err(format!(
            "{BESIDE_DIR}: a sample question has no admitted answer"
        ));
    }
    if read(&root.join(BESIDE_DIR).join("verdicts-400.txt"))? != beside_verdicts(&b) {
        return Err(format!(
            "{BESIDE_DIR}/verdicts-400.txt differs from the re-scored answers; run `answer-bench beside`"
        ));
    }
    let page = read(&root.join(BESIDE_PAGE))?;
    for row in beside_rows(&b) {
        if !page.lines().any(|l| l == row) {
            return Err(format!("{BESIDE_PAGE} lacks the re-scored row: {row}"));
        }
    }
    println!("ANSWER_BENCH_BESIDE=PASS sample=400 systems=9 page rows and verdict file re-scored");
    Ok(())
}

/// `answer-bench check`, `answer-bench tables [SET]`, `answer-bench beside`, or
/// `answer-bench score <[SET:]with-answer|[SET:]no-answer> <answers.txt> [--rules strict|published]`.
pub fn run(args: &[String]) -> Result<(), String> {
    let root = crate::workspace_root()?;
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (rules, args) = match args.as_slice() {
        [head @ .., "--rules", "strict"] => (Rules::Strict, head.to_vec()),
        [head @ .., "--rules", "published"] => (Rules::Published, head.to_vec()),
        _ => (Rules::Strict, args),
    };
    match args.as_slice() {
        ["check"] => check(root),
        ["beside"] => {
            print!("{}", beside_verdicts(&beside(root)?));
            Ok(())
        }
        ["tables"] => {
            print!("{}", render(&V1, &results(root, &V1)?));
            Ok(())
        }
        ["tables", name] => {
            let def = find_set(name)?;
            print!("{}", render(def, &results(root, def)?));
            Ok(())
        }
        ["score", spec, file] => {
            let (def, set) = find_part(spec)?;
            let qs = questions(
                set,
                &read(&root.join(def.dir).join(question_file(set)))?,
                def.count,
            )?;
            let given = answers(&read(Path::new(file))?, def.count)?;
            let verdicts = score(set, rules, &qs, &given);
            for (i, v) in verdicts.iter().enumerate() {
                println!("{}\t{}", i + 1, v.name());
            }
            println!(
                "ANSWER_BENCH_SCORE set={} rules={} {}",
                label(def, set),
                rules_name(rules),
                totals(&qs, &verdicts).describe(set)
            );
            Ok(())
        }
        _ => Err("usage: answer-bench check | tables [SET] | beside | score <[SET:]with-answer|[SET:]no-answer> <answers.txt> [--rules strict|published]".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beside_rows_follow_the_verdicts() {
        let v = |s: &str| -> Vec<Verdict> {
            s.bytes()
                .map(|c| match c {
                    b'C' => Verdict::Correct,
                    b'W' => Verdict::Wrong,
                    _ => Verdict::Unknown,
                })
                .collect()
        };
        let b = Beside {
            sample: vec![(1, "pl".into()), (2, "en".into()), (3, "en".into())],
            verdicts: [
                "CWU", "CCU", "UUU", "CUU", "CUU", "CUU", "WWC", "WWW", "UUU",
            ]
            .map(v)
            .to_vec(),
        };
        let rows = beside_rows(&b);
        for row in [
            "| GEL RAM, balanced | local bank, one stored passage | 2 | 1 | 1 | 1 | 50.0% (9.5–90.5%) |",
            "| GEL RAM, balanced | 1 / 0 / 0 | 0 / 1 / 1 |",
            "| balanced / GPT-OSS-120B | 1 / 1 (p = 1.000) | 0 / 1 (p = 1.000) |",
            "| Qwen3.8-27B | Groq API, closed book | 0 | 0 | 0 | 3 | — |",
        ] {
            assert!(rows.iter().any(|r| r == row), "missing row {row}");
        }
        let file = beside_verdicts(&b);
        assert!(file.ends_with("3\ten\tU\tU\tU\tU\tU\tU\tC\tW\tU\n"));
        assert_eq!(file.lines().count(), 2 + 3);
    }

    fn q(groups: &[&[(&str, bool)]]) -> Question {
        Question {
            accepted: groups
                .iter()
                .map(|g| g.iter().map(|(a, s)| (a.to_string(), *s)).collect())
                .collect(),
            invented: false,
            lang: "pl".into(),
        }
    }
    fn a(text: &str) -> Answer {
        Answer {
            text: text.to_owned(),
            passage: false,
        }
    }

    #[test]
    fn normalizes_diacritics_and_punctuation() {
        assert_eq!(normalize("Łódź, Gałków Duży!"), "lodz galkow duzy");
        assert_eq!(normalize("21:28 (6:12)"), "21 28 6 12");
        assert_eq!(normalize("Sigurðsson – Münster"), "sigurdsson munster");
    }

    #[test]
    fn strict_spellings_end_a_word_unless_marked_as_stems() {
        let mannheim = normalize("Mannheim");
        assert!(alias_hit(&mannheim, "mann", false, Rules::Published));
        assert!(!alias_hit(&mannheim, "mann", false, Rules::Strict));
        assert!(alias_hit(
            &normalize("w Łodzi"),
            "lodz",
            true,
            Rules::Strict
        ));
        assert!(!alias_hit(
            &normalize("Wlodzimierz"),
            "lodz",
            true,
            Rules::Strict
        ));
        for rules in [Rules::Published, Rules::Strict] {
            assert!(alias_hit(&normalize("9 punktów"), "9", false, rules));
            assert!(!alias_hit(&normalize("19 punktów"), "9", false, rules));
            assert!(!alias_hit(&normalize("90 punktów"), "9", true, rules));
        }
    }

    #[test]
    fn every_group_must_match() {
        let question = q(&[
            &[("aktyn", true), ("actin", true)],
            &[("miozyn", true), ("myosin", true)],
        ]);
        for rules in [Rules::Published, Rules::Strict] {
            assert_eq!(
                verdict(
                    Set::WithAnswer,
                    rules,
                    &question,
                    Some(&a("aktyny i miozyny"))
                ),
                Verdict::Correct
            );
            assert_eq!(
                verdict(Set::WithAnswer, rules, &question, Some(&a("tylko aktyna"))),
                Verdict::Wrong
            );
            assert_eq!(
                verdict(Set::WithAnswer, rules, &question, None),
                Verdict::Error
            );
        }
    }

    #[test]
    fn strict_unknown_is_the_whole_answer() {
        assert!(abstains("UNKNOWN.", Rules::Strict));
        assert!(abstains("UNKNOWN, but the answer is X", Rules::Published));
        assert!(!abstains("UNKNOWN, but the answer is X", Rules::Strict));
        assert!(!abstains("The answer is unknown", Rules::Published));
    }

    #[test]
    fn a_negation_is_not_automatically_a_rejection_under_strict_rules() {
        let question = Question {
            accepted: Vec::new(),
            invented: true,
            lang: "pl".into(),
        };
        for t in [
            "Nie zdobyła Oscara.",
            "It is not X, it is Y.",
            "Żaden.",
            "No such film exists.",
        ] {
            assert_eq!(
                verdict(Set::NoAnswer, Rules::Published, &question, Some(&a(t))),
                Verdict::Rejected,
                "{t}"
            );
            assert_eq!(
                verdict(Set::NoAnswer, Rules::Strict, &question, Some(&a(t))),
                Verdict::Review,
                "{t}"
            );
        }
        for t in [
            "Andrzej Kowalski",
            "1987",
            "Niemcy",
            "Nieporęt",
            "Northumberland",
            "Notting Hill",
        ] {
            assert_eq!(
                verdict(Set::NoAnswer, Rules::Strict, &question, Some(&a(t))),
                Verdict::Answered,
                "{t}"
            );
        }
        let passage = Answer {
            text: "He did not refuse; he began his service.".into(),
            passage: true,
        };
        assert_eq!(
            verdict(Set::NoAnswer, Rules::Published, &question, Some(&passage)),
            Verdict::Answered
        );
    }

    #[test]
    fn answer_files_are_fail_closed() {
        assert!(answers("1\tA\n1\tB\n", 80).is_err());
        assert!(answers("81\tA\n", 80).is_err());
        assert!(answers("1\tA\tsummary\n", 80).is_err());
        assert!(answers("1\tA\tpassage\textra\n", 80).is_err());
        assert!(answers("1\n", 80).is_err());
        let ok = answers("# header\n1\tA\n2\tB\tpassage\n", 80).unwrap();
        assert!(ok[&2].passage && !ok[&1].passage);
    }

    #[test]
    fn review_files_are_fail_closed() {
        assert!(reviews(
            Set::WithAnswer,
            &SYSTEMS,
            "gel-ram\t1\tCORRECT\tsame fact\n",
            80
        )
        .is_ok());
        assert!(reviews(Set::WithAnswer, &SYSTEMS, "gel-ram\t1\tCORRECT\n", 80).is_err());
        assert!(reviews(
            Set::WithAnswer,
            &SYSTEMS,
            "gel-ram\t1\tCORRECT\tr\textra\n",
            80
        )
        .is_err());
        assert!(reviews(Set::WithAnswer, &SYSTEMS, "someone\t1\tCORRECT\tr\n", 80).is_err());
        assert!(reviews(Set::WithAnswer, &SYSTEMS, "gel-ram\t0\tCORRECT\tr\n", 80).is_err());
        assert!(reviews(Set::WithAnswer, &SYSTEMS, "gel-ram\t1\tREJECTED\tr\n", 80).is_err());
        assert!(reviews(
            Set::NoAnswer,
            &SYSTEMS,
            "gel-ram\t1\tREJECTED\tr\ngel-ram\t1\tANSWERED\tr\n",
            80
        )
        .is_err());
        assert!(reviews(Set::NoAnswer, &SYSTEMS, "gel-ram\t1\tREJECTED\t \n", 80).is_err());
        let v3 = reviews(
            Set::WithAnswer,
            V3.systems,
            "tantivy-bm25\t1\tWRONG\tr\n",
            979,
        );
        assert!(v3.is_ok());
        assert!(reviews(
            Set::WithAnswer,
            V3.systems,
            "gpt-oss-20b\t1\tWRONG\tr\n",
            979
        )
        .is_err());
    }

    #[test]
    fn questions_need_a_known_language() {
        let passage = "a b";
        let row = |lang: &str| {
            format!(
                "1\t{lang}\tq\te\ta\tt\tu\td\tz\t{}\t{passage}\n",
                sha256(passage)
            )
        };
        assert_eq!(
            questions(Set::WithAnswer, &row("en"), 1).unwrap()[0].lang,
            "en"
        );
        assert!(questions(Set::WithAnswer, &row("de"), 1).is_err());
    }

    #[test]
    fn exact_sign_test_and_wilson_shares() {
        assert!((sign_test(0, 5) - 0.0625).abs() < 1e-12);
        assert!((sign_test(8, 2) - 0.109375).abs() < 1e-12);
        assert_eq!(sign_test(1, 1), 1.0);
        assert_eq!(sign_test(0, 0), 1.0);
        assert_eq!(p_text(0.0004), "p < 0.001");
        assert_eq!(p_text(0.0625), "p = 0.062");
        assert_eq!(share(0, 0), "—");
        assert_eq!(share(59, 63), "93.7% (84.8–97.5%)");
    }

    #[test]
    fn every_paired_system_is_recorded() {
        for def in SETS {
            for (a, b) in def.paired {
                for id in [a, b] {
                    assert!(def.systems.iter().any(|(s, _)| s == id), "{id}");
                }
            }
        }
    }

    #[test]
    fn recorded_answers_reproduce_the_published_tables() {
        check(crate::workspace_root().unwrap()).unwrap();
    }
}
