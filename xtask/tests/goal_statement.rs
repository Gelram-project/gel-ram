//! The goal is stated once, in docs/GOAL.md, in a fixed shape that the project
//! website reads; the README and start-here pages repeat its sentence word for word.
const GOAL: &str = include_str!("../../docs/GOAL.md");
const README: &str = include_str!("../../README.md");
const START_HERE: &str = include_str!("../../docs/START-HERE.md");

fn words(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One commitment: (heading, text, status).
type Commitment<'a> = (&'a str, &'a str, &'a str);

/// The goal sentence and the four commitments.
fn parse(goal: &str) -> Result<(&str, Vec<Commitment<'_>>), &'static str> {
    let mut sections = goal.split("\n## ");
    let head = sections.next().ok_or("empty")?;
    let mut paragraphs = head.split("\n\n");
    if paragraphs.next() != Some("# Goal") {
        return Err("the file must start with '# Goal' and a blank line");
    }
    let sentence = paragraphs.next().ok_or("no goal paragraph")?;
    if !sentence.starts_with("The goal is ") || sentence.contains('\n') {
        return Err("the goal paragraph must be one line starting 'The goal is '");
    }
    let mut commitments = Vec::new();
    for (i, section) in sections.enumerate() {
        let parts: Vec<&str> = section.trim_end().split("\n\n").collect();
        let [heading, text, status] = parts[..] else {
            return Err("a commitment must be a heading, one paragraph and a status line");
        };
        let index = format!("{:02} · ", i + 1);
        if !heading.starts_with(&index) || text.contains('\n') || text.is_empty() {
            return Err("a commitment heading must be 'NN · Title' with one one-line paragraph");
        }
        let status = status
            .strip_prefix("Status: ")
            .filter(|s| !s.contains('\n'))
            .ok_or("a commitment must end with one 'Status: ' line")?;
        commitments.push((&heading[index.len()..], text, status));
    }
    if commitments.len() != 4 {
        return Err("there must be exactly four commitments");
    }
    Ok((sentence, commitments))
}

fn check(goal: &str, pages: &[&str]) -> Result<(), &'static str> {
    let (sentence, _) = parse(goal)?;
    let sentence = words(sentence);
    if pages.iter().all(|page| words(page).contains(&sentence)) {
        Ok(())
    } else {
        Err("a page does not repeat the goal sentence word for word")
    }
}

#[test]
fn the_goal_has_one_sentence_and_four_commitments_repeated_where_it_is_stated() {
    check(GOAL, &[README, START_HERE]).unwrap();
    let (_, commitments) = parse(GOAL).unwrap();
    assert_eq!(
        commitments[1].0,
        "In its own words, from a source, or “I don’t know”"
    );
}

#[test]
fn a_changed_page_or_a_broken_shape_is_rejected() {
    let changed = README.replacen("sentences of its own", "sentences", 1);
    assert_ne!(changed, README);
    assert!(check(GOAL, &[&changed, START_HERE]).is_err());
    let three = GOAL.rsplit_once("\n## 04").unwrap().0;
    assert!(check(three, &[README, START_HERE]).is_err());
    let no_status = GOAL.replacen("Status: goal: knowledge", "goal: knowledge", 1);
    assert!(check(&no_status, &[README, START_HERE]).is_err());
    let renumbered = GOAL.replacen("## 02 · ", "## 05 · ", 1);
    assert!(check(&renumbered, &[README, START_HERE]).is_err());
}
