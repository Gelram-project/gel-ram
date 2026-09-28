//! Answer-or-abstain set (docs/answer-or-abstain): score any system's answers
//! with the rules behind the published GEL-beside-Groq tables.
//!
//! `score` rates one answer file. `check` re-scores the recorded answers of
//! GEL RAM and three language models and fails if the totals differ from the
//! published tables, so the tables cannot drift from the data. It re-scores
//! recorded text; it does not re-run any system.
use std::{collections::HashMap, fs, path::Path};

const DIR: &str = "docs/answer-or-abstain";
const SYSTEMS: [&str; 4] = ["gel-ram", "gpt-oss-120b", "gpt-oss-20b", "qwen3.8-27b"];
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Correct,
    Wrong,
    Unknown,
    Rejected,
    Answered,
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
            Verdict::Error => "ERROR",
        }
    }
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "CORRECT" => Ok(Verdict::Correct),
            "WRONG" => Ok(Verdict::Wrong),
            "UNKNOWN" => Ok(Verdict::Unknown),
            "REJECTED" => Ok(Verdict::Rejected),
            "ANSWERED" => Ok(Verdict::Answered),
            other => Err(format!("unknown verdict {other}")),
        }
    }
}

/// One question: accepted spelling groups (with-answer) or whether the subject is invented (no-answer).
struct Question {
    accepted: Vec<Vec<String>>,
    invented: bool,
}

struct Answer {
    text: String,
    passage: bool,
}

/// Totals: correct, wrong, unknown, rejected, answered (invented), answered (false premise), errors.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Totals([usize; 7]);

impl Totals {
    fn add(&mut self, v: Verdict, invented: bool) {
        let i = match v {
            Verdict::Correct => 0,
            Verdict::Wrong => 1,
            Verdict::Unknown => 2,
            Verdict::Rejected => 3,
            Verdict::Answered if invented => 4,
            Verdict::Answered => 5,
            Verdict::Error => 6,
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
                t[6]
            ),
            Set::NoAnswer => format!(
                "answered_anyway={} (invented={} false_premise={}) unknown={} rejected={} errors={}",
                t[4] + t[5],
                t[4],
                t[5],
                t[2],
                t[3],
                t[6]
            ),
        }
    }
}

/// Published totals: (set, system, automatic, after the manual review).
const EXPECTED: [(Set, &str, [usize; 7], [usize; 7]); 8] = [
    (
        Set::WithAnswer,
        "gel-ram",
        [11, 0, 69, 0, 0, 0, 0],
        [11, 0, 69, 0, 0, 0, 0],
    ),
    (
        Set::WithAnswer,
        "gpt-oss-120b",
        [11, 20, 49, 0, 0, 0, 0],
        [10, 21, 49, 0, 0, 0, 0],
    ),
    (
        Set::WithAnswer,
        "gpt-oss-20b",
        [8, 28, 44, 0, 0, 0, 0],
        [8, 28, 44, 0, 0, 0, 0],
    ),
    (
        Set::WithAnswer,
        "qwen3.8-27b",
        [5, 12, 63, 0, 0, 0, 0],
        [6, 11, 63, 0, 0, 0, 0],
    ),
    (
        Set::NoAnswer,
        "gel-ram",
        [0, 0, 74, 0, 0, 6, 0],
        [0, 0, 74, 0, 0, 6, 0],
    ),
    (
        Set::NoAnswer,
        "gpt-oss-120b",
        [0, 0, 68, 0, 5, 7, 0],
        [0, 0, 68, 3, 4, 5, 0],
    ),
    (
        Set::NoAnswer,
        "gpt-oss-20b",
        [0, 0, 40, 0, 23, 17, 0],
        [0, 0, 40, 2, 22, 16, 0],
    ),
    (
        Set::NoAnswer,
        "qwen3.8-27b",
        [0, 0, 75, 0, 3, 2, 0],
        [0, 0, 75, 0, 3, 2, 0],
    ),
];

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

/// An accepted spelling must start a word; one ending in a digit must also end it ("9" is not "19" or "90").
fn alias_hit(normalized: &str, alias: &str) -> bool {
    let padded = format!(" {normalized} ");
    let needle = format!(" {alias}");
    let digit = alias.chars().last().is_some_and(|c| c.is_ascii_digit());
    padded
        .match_indices(&needle)
        .any(|(i, _)| !digit || padded[i + needle.len()..].starts_with(' '))
}

fn abstains(text: &str) -> bool {
    normalize(text).split(' ').next() == Some("unknown")
}

fn rejects(text: &str) -> bool {
    normalize(text)
        .split(' ')
        .any(|w| NEGATIONS.contains(&w) || NEGATION_PREFIXES.iter().any(|p| w.starts_with(p)))
}

fn verdict(set: Set, q: &Question, answer: Option<&Answer>) -> Verdict {
    let Some(a) = answer.filter(|a| !a.text.trim().is_empty()) else {
        return Verdict::Error;
    };
    if abstains(&a.text) {
        return Verdict::Unknown;
    }
    match set {
        Set::WithAnswer => {
            let n = normalize(&a.text);
            if q.accepted
                .iter()
                .all(|g| g.iter().any(|al| alias_hit(&n, al)))
            {
                Verdict::Correct
            } else {
                Verdict::Wrong
            }
        }
        // A returned source passage is an answer: its words are not read as a rejection.
        Set::NoAnswer if a.passage => Verdict::Answered,
        Set::NoAnswer if rejects(&a.text) => Verdict::Rejected,
        Set::NoAnswer => Verdict::Answered,
    }
}

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split('\t').collect())
}

fn question_number(field: &str, seen: &mut [bool; 80]) -> Result<usize, String> {
    let nr: usize = field
        .parse()
        .map_err(|_| format!("bad question number {field}"))?;
    if !(1..=80).contains(&nr) || std::mem::replace(&mut seen[nr - 1], true) {
        return Err(format!("question number {nr} out of range or repeated"));
    }
    Ok(nr)
}

fn questions(set: Set, text: &str) -> Result<Vec<Question>, String> {
    let mut out: Vec<Option<Question>> = (0..80).map(|_| None).collect();
    let mut seen = [false; 80];
    for f in rows(text) {
        let nr = question_number(f[0], &mut seen)?;
        let q = match set {
            Set::WithAnswer if f.len() == 6 => Question {
                accepted: f[4]
                    .split(';')
                    .map(|g| g.split('|').map(str::to_owned).collect())
                    .collect(),
                invented: false,
            },
            Set::NoAnswer if f.len() == 5 && ["invented", "false-premise"].contains(&f[2]) => {
                Question {
                    accepted: Vec::new(),
                    invented: f[2] == "invented",
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

fn answers(text: &str) -> Result<HashMap<usize, Answer>, String> {
    let mut out = HashMap::new();
    let mut seen = [false; 80];
    for f in rows(text) {
        let nr = question_number(f[0], &mut seen)?;
        let passage = match f.get(2) {
            None => false,
            Some(&"passage") => true,
            Some(other) => return Err(format!("answer {nr}: unknown kind {other}")),
        };
        let text = f.get(1).copied().unwrap_or_default().to_owned();
        out.insert(nr, Answer { text, passage });
    }
    Ok(out)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn load(root: &Path, set: Set) -> Result<Vec<Question>, String> {
    let name = match set {
        Set::WithAnswer => "with-answer-questions.txt",
        Set::NoAnswer => "no-answer-questions.txt",
    };
    questions(set, &read(&root.join(DIR).join(name))?)
}

fn score(set: Set, qs: &[Question], given: &HashMap<usize, Answer>) -> Vec<Verdict> {
    qs.iter()
        .enumerate()
        .map(|(i, q)| verdict(set, q, given.get(&(i + 1))))
        .collect()
}

fn totals(qs: &[Question], verdicts: &[Verdict]) -> Totals {
    let mut t = Totals::default();
    for (q, v) in qs.iter().zip(verdicts) {
        t.add(*v, q.invented);
    }
    t
}

fn set_name(set: Set) -> &'static str {
    match set {
        Set::WithAnswer => "with-answer",
        Set::NoAnswer => "no-answer",
    }
}

/// Re-score every recorded answer file and compare with the published totals.
pub fn check(root: &Path) -> Result<(), String> {
    for set in [Set::WithAnswer, Set::NoAnswer] {
        let qs = load(root, set)?;
        let dir = root.join(DIR).join("recorded").join(set_name(set));
        let mut review: HashMap<(String, usize), Verdict> = HashMap::new();
        for f in rows(&read(&dir.join("review.txt"))?) {
            let nr: usize = f[1].parse().map_err(|_| "bad review number".to_owned())?;
            review.insert((f[0].to_owned(), nr), Verdict::parse(f[2])?);
        }
        for system in SYSTEMS {
            let given = answers(&read(&dir.join(format!("{system}.txt")))?)?;
            let auto = score(set, &qs, &given);
            let mut reviewed = auto.clone();
            for (i, v) in reviewed.iter_mut().enumerate() {
                if let Some(r) = review.get(&(system.to_owned(), i + 1)) {
                    *v = *r;
                }
            }
            let (a, r) = (totals(&qs, &auto), totals(&qs, &reviewed));
            let expected = EXPECTED
                .iter()
                .find(|(s, n, _, _)| *s == set && *n == system)
                .ok_or("missing expected totals")?;
            if a.0 != expected.2 || r.0 != expected.3 {
                return Err(format!(
                    "{} {system}: recorded answers give {} / reviewed {}, published tables differ",
                    set_name(set),
                    a.describe(set),
                    r.describe(set)
                ));
            }
            println!(
                "ANSWER_BENCH set={} system={system} {} reviewed: {}",
                set_name(set),
                a.describe(set),
                r.describe(set)
            );
        }
    }
    println!("ANSWER_BENCH=PASS recorded answers reproduce the published totals");
    Ok(())
}

/// `answer-bench check` or `answer-bench score <with-answer|no-answer> <answers.txt>`.
pub fn run(args: &[String]) -> Result<(), String> {
    let root = crate::workspace_root()?;
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check"] => check(root),
        ["score", set, file] => {
            let set = match *set {
                "with-answer" => Set::WithAnswer,
                "no-answer" => Set::NoAnswer,
                other => return Err(format!("unknown set {other}")),
            };
            let qs = load(root, set)?;
            let given = answers(&read(Path::new(file))?)?;
            let verdicts = score(set, &qs, &given);
            for (i, v) in verdicts.iter().enumerate() {
                println!("{}\t{}", i + 1, v.name());
            }
            println!(
                "ANSWER_BENCH_SCORE set={} {}",
                set_name(set),
                totals(&qs, &verdicts).describe(set)
            );
            Ok(())
        }
        _ => Err(
            "usage: answer-bench check | answer-bench score <with-answer|no-answer> <answers.txt>"
                .into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(groups: &[&[&str]]) -> Question {
        Question {
            accepted: groups
                .iter()
                .map(|g| g.iter().map(|a| a.to_string()).collect())
                .collect(),
            invented: false,
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
    fn spelling_starts_a_word_and_numbers_end_one() {
        assert!(alias_hit(&normalize("w Łodzi"), "lodz"));
        assert!(!alias_hit(&normalize("Wlodzimierz"), "lodz"));
        assert!(alias_hit(&normalize("9 punktów"), "9"));
        assert!(!alias_hit(&normalize("19 punktów"), "9"));
        assert!(!alias_hit(&normalize("90 punktów"), "9"));
    }

    #[test]
    fn every_group_must_match() {
        let question = q(&[&["aktyn", "actin"], &["miozyn", "myosin"]]);
        assert_eq!(
            verdict(Set::WithAnswer, &question, Some(&a("aktyny i miozyny"))),
            Verdict::Correct
        );
        assert_eq!(
            verdict(Set::WithAnswer, &question, Some(&a("tylko aktyna"))),
            Verdict::Wrong
        );
        assert_eq!(
            verdict(Set::WithAnswer, &question, Some(&a("UNKNOWN."))),
            Verdict::Unknown
        );
        assert_eq!(verdict(Set::WithAnswer, &question, None), Verdict::Error);
    }

    #[test]
    fn unknown_only_as_the_first_word() {
        assert!(abstains(" unknown — not sure"));
        assert!(!abstains("The answer is unknown"));
    }

    #[test]
    fn rejecting_a_premise_is_right_and_inventing_is_wrong() {
        let question = Question {
            accepted: Vec::new(),
            invented: true,
        };
        for t in [
            "Nie zdobyła Oscara.",
            "She did not win an Oscar.",
            "Żaden.",
            "No such film exists.",
            "This is fictional.",
        ] {
            assert_eq!(
                verdict(Set::NoAnswer, &question, Some(&a(t))),
                Verdict::Rejected,
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
                verdict(Set::NoAnswer, &question, Some(&a(t))),
                Verdict::Answered,
                "{t}"
            );
        }
        let passage = Answer {
            text: "He did not refuse; he began his service.".into(),
            passage: true,
        };
        assert_eq!(
            verdict(Set::NoAnswer, &question, Some(&passage)),
            Verdict::Answered
        );
    }

    #[test]
    fn answer_files_reject_repeats_and_unknown_kinds() {
        assert!(answers("1\tA\n1\tB\n").is_err());
        assert!(answers("81\tA\n").is_err());
        assert!(answers("1\tA\tsummary\n").is_err());
        let ok = answers("# header\n1\tA\n2\tB\tpassage\n").unwrap();
        assert!(ok[&2].passage && !ok[&1].passage);
    }

    #[test]
    fn recorded_answers_reproduce_the_published_totals() {
        check(crate::workspace_root().unwrap()).unwrap();
    }
}
