//! `ucd-tables UCD_DIR NEW_FILE`: writes the canonical-normalization tables of
//! gel-source (`crates/gel-source/src/nfc_tables.rs`) from two files of the
//! Unicode Character Database: UnicodeData.txt (canonical combining classes and
//! canonical decomposition mappings) and DerivedNormalizationProps.txt
//! (Full_Composition_Exclusion). The output names the UCD version, the
//! copyright year and the SHA-256 of both inputs, and nothing about the machine
//! it was made on.
use std::{collections::BTreeSet, fmt::Write as _, fs, path::Path};

fn code_point(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s.trim(), 16).map_err(|e| format!("{s}: {e}"))
}

fn range(s: &str) -> Result<(u32, u32), String> {
    match s.trim().split_once("..") {
        Some((a, b)) => Ok((code_point(a)?, code_point(b)?)),
        None => code_point(s).map(|c| (c, c)),
    }
}

/// What the tables are made of, as read from the two files.
struct Ucd {
    version: (u8, u8, u8),
    /// The year of the copyright line in the header of DerivedNormalizationProps.txt.
    year: u16,
    /// (first, last, class) for nonzero classes, sorted.
    ccc: Vec<(u32, u32, u8)>,
    /// (code point, first, second) canonical decompositions, second 0 for one code point, sorted.
    decomposition: Vec<(u32, u32, u32)>,
    excluded: BTreeSet<u32>,
    /// (first, last, maybe) for NFC_QC No or Maybe, sorted and merged.
    quick_check: Vec<(u32, u32, bool)>,
    /// (first, second, composite), sorted.
    composition: Vec<(u32, u32, u32)>,
}

impl Ucd {
    fn counts(&self) -> [usize; 5] {
        [
            self.ccc.len(),
            self.decomposition.len(),
            self.excluded.len(),
            self.composition.len(),
            self.quick_check.len(),
        ]
    }
}

/// The header of DerivedNormalizationProps.txt: its version and the year of
/// its copyright line (`# © 2024 Unicode®, Inc.`).
fn header(normalization_props: &str) -> Result<((u8, u8, u8), u16), String> {
    let version = normalization_props
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("# DerivedNormalizationProps-"))
        .and_then(|l| l.strip_suffix(".txt"))
        .ok_or("DerivedNormalizationProps.txt has no version header")?;
    let parts: Vec<u8> = version
        .split('.')
        .map(|p| p.parse().map_err(|e| format!("version {version}: {e}")))
        .collect::<Result<_, String>>()?;
    let [major, minor, update] = parts[..] else {
        return Err(format!("version {version}"));
    };
    let year = normalization_props
        .lines()
        .take_while(|l| l.starts_with('#'))
        .find_map(|l| {
            l.strip_prefix("# © ")
                .and_then(|l| l.strip_suffix(" Unicode®, Inc."))
        })
        .ok_or("DerivedNormalizationProps.txt has no copyright line in its header")?;
    let year: u16 = year
        .parse()
        .map_err(|e| format!("copyright year {year}: {e}"))?;
    Ok(((major, minor, update), year))
}

fn read(unicode_data: &str, normalization_props: &str) -> Result<Ucd, String> {
    let (version, year) = header(normalization_props)?;
    let mut ccc: Vec<(u32, u32, u8)> = Vec::new();
    let mut decomposition: Vec<(u32, u32, u32)> = Vec::new();
    let mut first: Option<u32> = None;
    for line in unicode_data.lines() {
        let f: Vec<&str> = line.split(';').collect();
        if f.len() < 6 {
            return Err(format!("UnicodeData.txt line: {line}"));
        }
        let cp = code_point(f[0])?;
        let class: u8 = f[3].parse().map_err(|e| format!("{line}: {e}"))?;
        let (lo, hi) = if f[1].ends_with(", First>") {
            first = Some(cp);
            continue;
        } else if f[1].ends_with(", Last>") {
            (first.take().ok_or("Last without First")?, cp)
        } else {
            (cp, cp)
        };
        if class != 0 {
            match ccc.last_mut() {
                Some(last) if last.2 == class && last.1 + 1 == lo => last.1 = hi,
                _ => ccc.push((lo, hi, class)),
            }
        }
        if !f[5].is_empty() && !f[5].starts_with('<') {
            let to: Vec<u32> = f[5].split(' ').map(code_point).collect::<Result<_, _>>()?;
            match to[..] {
                [a] => decomposition.push((cp, a, 0)),
                [a, b] => decomposition.push((cp, a, b)),
                _ => return Err(format!("canonical decomposition longer than two: {line}")),
            }
        }
    }
    let mut excluded = BTreeSet::new();
    let mut quick_check: Vec<(u32, u32, bool)> = Vec::new();
    for line in normalization_props.lines() {
        let data = line.split('#').next().unwrap_or("");
        let f: Vec<&str> = data.split(';').map(str::trim).collect();
        if f.len() >= 2 && f[1] == "Full_Composition_Exclusion" {
            let (a, b) = range(f[0])?;
            excluded.extend(a..=b);
        }
        if f.len() >= 3 && f[1] == "NFC_QC" {
            let (a, b) = range(f[0])?;
            match f[2] {
                "M" => quick_check.push((a, b, true)),
                "N" => quick_check.push((a, b, false)),
                _ => return Err(format!("NFC_QC value: {line}")),
            }
        }
    }
    quick_check.sort_unstable();
    let mut merged: Vec<(u32, u32, bool)> = Vec::new();
    for (a, b, maybe) in quick_check {
        match merged.last_mut() {
            Some(last) if last.2 == maybe && last.1 + 1 == a => last.1 = b,
            Some(last) if last.1 >= a => return Err(format!("NFC_QC ranges overlap at {a:04X}")),
            _ => merged.push((a, b, maybe)),
        }
    }
    let mut composition: Vec<(u32, u32, u32)> = decomposition
        .iter()
        .filter(|&&(cp, _, b)| b != 0 && !excluded.contains(&cp))
        .map(|&(cp, a, b)| (a, b, cp))
        .collect();
    composition.sort_unstable();
    Ok(Ucd {
        version,
        year,
        ccc,
        decomposition,
        excluded,
        quick_check: merged,
        composition,
    })
}

/// The tables as Rust source.
fn generate(unicode_data: &str, normalization_props: &str) -> Result<(String, [usize; 5]), String> {
    let ucd = read(unicode_data, normalization_props)?;
    let counts = ucd.counts();
    let (major, minor, update) = ucd.version;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Generated by `cargo run -p xtask -- ucd-tables UCD_DIR NEW_FILE` from the Unicode\n\
         // Character Database {major}.{minor}.{update}; do not edit. Inputs (SHA-256):\n\
         //   UnicodeData.txt                {}\n\
         //   DerivedNormalizationProps.txt  {}\n\
         // Copyright (c) 1991-{} Unicode, Inc. These are Unicode data files, modified:\n\
         // transformed into Rust tables. Used under the Unicode License v3, whose full\n\
         // text is crates/gel-source/LICENSE-UNICODE.txt; see THIRD-PARTY-NOTICES.md.\n\
         // Counts: {} combining-class ranges, {} canonical decompositions,\n\
         // {} Full_Composition_Exclusion code points, {} primary composites,\n\
         // {} NFC_QC ranges (No or Maybe).\n",
        gel_source::hex(&gel_source::digest(unicode_data.as_bytes())),
        gel_source::hex(&gel_source::digest(normalization_props.as_bytes())),
        ucd.year,
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
    );
    let _ = writeln!(
        out,
        "/// The Unicode version of these tables, checked by the tests.\n#[cfg(test)]\npub(crate) const UNICODE_VERSION: (u8, u8, u8) = ({major}, {minor}, {update});\n"
    );
    let mut table = |name: &str, doc: &str, ty: &str, rows: Vec<String>| {
        let _ = writeln!(
            out,
            "/// {doc}\n#[rustfmt::skip]\npub(crate) const {name}: &[{ty}] = &["
        );
        for chunk in rows.chunks(4) {
            let _ = writeln!(out, "    {}", chunk.join(" "));
        }
        let _ = writeln!(out, "];\n");
    };
    table(
        "CANONICAL_COMBINING_CLASS",
        "(first, last, class) for every range of code points with a nonzero canonical combining class.",
        "(u32, u32, u8)",
        ucd.ccc
            .iter()
            .map(|(a, b, c)| format!("(0x{a:04X}, 0x{b:04X}, {c}),"))
            .collect(),
    );
    table(
        "CANONICAL_DECOMPOSITION",
        "(code point, first, second) canonical decomposition mappings, sorted; second is 0 for a single code point.",
        "(u32, u32, u32)",
        ucd.decomposition
            .iter()
            .map(|(cp, a, b)| format!("(0x{cp:04X}, 0x{a:04X}, 0x{b:04X}),"))
            .collect(),
    );
    table(
        "PRIMARY_COMPOSITE",
        "(first, second, composite) for every primary composite, sorted by (first, second).",
        "(u32, u32, u32)",
        ucd.composition
            .iter()
            .map(|(a, b, cp)| format!("(0x{a:04X}, 0x{b:04X}, 0x{cp:04X}),"))
            .collect(),
    );
    table(
        "NFC_QUICK_CHECK",
        "(first, last, maybe) for every range of code points whose NFC_Quick_Check is not Yes, sorted; maybe is false for No.",
        "(u32, u32, bool)",
        ucd.quick_check
            .iter()
            .map(|(a, b, maybe)| format!("(0x{a:04X}, 0x{b:04X}, {maybe}),"))
            .collect(),
    );
    while out.ends_with("\n\n") {
        out.pop();
    }
    Ok((out, counts))
}

pub fn run(args: &[String]) -> Result<(), String> {
    let [dir, new_file] = args else {
        return Err("usage: cargo run -p xtask -- ucd-tables UCD_DIR NEW_FILE".into());
    };
    let read = |name: &str| {
        fs::read_to_string(Path::new(dir).join(name)).map_err(|e| format!("{dir}/{name}: {e}"))
    };
    let (out, counts) = generate(
        &read("UnicodeData.txt")?,
        &read("DerivedNormalizationProps.txt")?,
    )?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(new_file)
        .and_then(|mut f| std::io::Write::write_all(&mut f, out.as_bytes()))
        .map_err(|e| format!("{new_file}: {e}"))?;
    println!(
        "UCD_TABLES=WRITTEN ccc_ranges={} decompositions={} full_composition_exclusions={} primary_composites={} nfc_qc_ranges={} file={new_file}",
        counts[0], counts[1], counts[2], counts[3], counts[4]
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const UNICODE_DATA_15_1: &str =
        include_str!("../../crates/gel-source/tests/data/ucd-15.1.0/UnicodeData.txt");
    const PROPS_15_1: &str =
        include_str!("../../crates/gel-source/tests/data/ucd-15.1.0/DerivedNormalizationProps.txt");
    const UNICODE_DATA_16_0: &str =
        include_str!("../../crates/gel-source/tests/data/ucd-16.0.0/UnicodeData.txt");
    const PROPS_16_0: &str =
        include_str!("../../crates/gel-source/tests/data/ucd-16.0.0/DerivedNormalizationProps.txt");

    #[test]
    fn a_small_database_gives_ranges_decompositions_and_composites() {
        let data = "0041;LATIN CAPITAL LETTER A;Lu;0;L;;;;;N;;;;0061;\n\
                    00C0;LATIN CAPITAL LETTER A WITH GRAVE;Lu;0;L;0041 0300;;;;N;;;;00E0;\n\
                    0300;COMBINING GRAVE ACCENT;Mn;230;NSM;;;;;N;;;;;\n\
                    0301;COMBINING ACUTE ACCENT;Mn;230;NSM;;;;;N;;;;;\n\
                    0340;COMBINING GRAVE TONE MARK;Mn;230;NSM;0300;;;;N;;;;;\n\
                    0958;DEVANAGARI LETTER QA;Lo;0;L;0915 093C;;;;N;;;;;\n\
                    AC00;<Hangul Syllable, First>;Lo;0;L;;;;;N;;;;;\n\
                    D7A3;<Hangul Syllable, Last>;Lo;0;L;;;;;N;;;;;\n";
        let props = "# DerivedNormalizationProps-16.0.0.txt\n# Date: 2024-04-30\n# © 2024 Unicode®, Inc.\n\n\
                     0340          ; Full_Composition_Exclusion # Mn\n0958          ; Full_Composition_Exclusion # Lo\n\
                     0300..0301    ; NFC_QC; M # Mn\n0340          ; NFC_QC; N # Mn\n0341          ; NFC_QC; N # Mn\n0958          ; NFC_QC; N # Lo\n";
        let (out, counts) = generate(data, props).unwrap();
        // 0300 and 0301 are adjacent and share a class; 0340 is not adjacent.
        // NFC_QC: 0340 and 0341 merge; 0300..0301 (Maybe) and 0958 stay apart.
        assert_eq!(counts, [2, 3, 2, 1, 3]);
        assert!(out.contains("UNICODE_VERSION: (u8, u8, u8) = (16, 0, 0);"));
        assert!(out.contains("Character Database 16.0.0; do not edit."));
        assert!(out.contains("// Copyright (c) 1991-2024 Unicode, Inc."));
        assert!(out.contains("(0x0300, 0x0301, 230), (0x0340, 0x0340, 230),"));
        assert!(out.contains("(0x0340, 0x0300, 0x0000),"));
        assert!(out.contains("(0x0041, 0x0300, 0x00C0),"));
        assert!(!out.contains("0x0958)"));
        assert!(out
            .contains("(0x0300, 0x0301, true), (0x0340, 0x0341, false), (0x0958, 0x0958, false),"));
        assert!(generate(data, "# no header\n").is_err());
        // The copyright line must be in the header, before the first data line.
        let no_year = "# DerivedNormalizationProps-16.0.0.txt\n\n# © 2024 Unicode®, Inc.\n";
        assert!(generate(data, no_year).is_err());
        let overlap = format!("{props}0301          ; NFC_QC; N # Mn\n");
        assert!(generate(data, &overlap).is_err());
    }

    /// The tables in the tree are exactly what this generator makes from the
    /// UCD 16.0.0 files kept beside them.
    #[test]
    fn the_tables_in_the_tree_are_regenerated_byte_for_byte() {
        let (out, counts) = generate(UNICODE_DATA_16_0, PROPS_16_0).unwrap();
        assert_eq!(counts, [393, 2081, 1120, 961, 122]);
        assert!(out == include_str!("../../crates/gel-source/src/nfc_tables.rs"));
    }

    /// Ranges of the code points assigned in a UnicodeData.txt, sorted.
    fn assigned(unicode_data: &str) -> Vec<(u32, u32)> {
        let mut ranges = Vec::new();
        let mut first = None;
        for line in unicode_data.lines() {
            let f: Vec<&str> = line.split(';').collect();
            let cp = code_point(f[0]).unwrap();
            if f[1].ends_with(", First>") {
                first = Some(cp);
            } else if f[1].ends_with(", Last>") {
                ranges.push((first.take().unwrap(), cp));
            } else {
                ranges.push((cp, cp));
            }
        }
        assert!(ranges.windows(2).all(|w| w[0].1 < w[1].0));
        ranges
    }

    fn in_ranges<T: Copy>(table: &[(u32, u32, T)], c: u32) -> Option<T> {
        let at = table.partition_point(|r| r.1 < c);
        table.get(at).filter(|r| r.0 <= c).map(|r| r.2)
    }

    /// Unicode stability, read from the files: for every code point assigned
    /// in 15.1.0 the 16.0.0 tables give the same combining class, canonical
    /// decomposition, Full_Composition_Exclusion and NFC_QC, every 15.1.0
    /// primary composite is kept, and every entry 16.0.0 adds is about a code
    /// point assigned after 15.1.0.
    #[test]
    fn the_16_0_tables_keep_every_15_1_property() {
        let old = read(UNICODE_DATA_15_1, PROPS_15_1).unwrap();
        let new = read(UNICODE_DATA_16_0, PROPS_16_0).unwrap();
        assert_eq!((old.version, old.year), ((15, 1, 0), 2023));
        assert_eq!((new.version, new.year), ((16, 0, 0), 2024));
        assert_eq!(old.counts(), [388, 2061, 1120, 941, 115]);
        let old_assigned = assigned(UNICODE_DATA_15_1);
        let was_assigned = |c: u32| {
            let at = old_assigned.partition_point(|r| r.1 < c);
            old_assigned.get(at).is_some_and(|r| r.0 <= c)
        };
        let decomposition = |u: &Ucd| -> BTreeMap<u32, (u32, u32)> {
            u.decomposition
                .iter()
                .map(|&(cp, a, b)| (cp, (a, b)))
                .collect()
        };
        let (old_decomposition, new_decomposition) = (decomposition(&old), decomposition(&new));
        let mut checked = 0u32;
        for &(first, last) in &old_assigned {
            for c in first..=last {
                assert_eq!(
                    in_ranges(&old.ccc, c),
                    in_ranges(&new.ccc, c),
                    "class {c:04X}"
                );
                assert_eq!(
                    old_decomposition.get(&c),
                    new_decomposition.get(&c),
                    "decomposition {c:04X}"
                );
                assert_eq!(
                    old.excluded.contains(&c),
                    new.excluded.contains(&c),
                    "exclusion {c:04X}"
                );
                assert_eq!(
                    in_ranges(&old.quick_check, c),
                    in_ranges(&new.quick_check, c),
                    "NFC_QC {c:04X}"
                );
                checked += 1;
            }
        }
        // Every code point UnicodeData.txt 15.1.0 lists, ranges included.
        assert_eq!(checked, 289_394);
        let composite = |u: &Ucd| -> BTreeMap<(u32, u32), u32> {
            u.composition
                .iter()
                .map(|&(a, b, cp)| ((a, b), cp))
                .collect()
        };
        let (old_composite, new_composite) = (composite(&old), composite(&new));
        for (pair, cp) in &old_composite {
            assert_eq!(new_composite.get(pair), Some(cp), "composite {cp:04X}");
        }
        // What 16.0.0 adds, each entry about a code point assigned later.
        let added_classes: Vec<u32> = new
            .ccc
            .iter()
            .flat_map(|&(a, b, _)| a..=b)
            .filter(|&c| in_ranges(&old.ccc, c).is_none())
            .collect();
        assert!(added_classes.iter().all(|&c| !was_assigned(c)));
        let added_decompositions: Vec<u32> = new_decomposition
            .keys()
            .copied()
            .filter(|c| !old_decomposition.contains_key(c))
            .collect();
        assert!(added_decompositions.iter().all(|&c| !was_assigned(c)));
        let added_composites: Vec<u32> = new_composite
            .iter()
            .filter(|(pair, _)| !old_composite.contains_key(pair))
            .map(|(_, &cp)| cp)
            .collect();
        assert!(added_composites.iter().all(|&c| !was_assigned(c)));
        let added_exclusions = new.excluded.difference(&old.excluded).count();
        let added_quick_check: Vec<u32> = new
            .quick_check
            .iter()
            .flat_map(|&(a, b, _)| a..=b)
            .filter(|&c| in_ranges(&old.quick_check, c).is_none())
            .collect();
        assert!(added_quick_check.iter().all(|&c| !was_assigned(c)));
        assert_eq!(
            (
                added_classes.len(),
                added_decompositions.len(),
                added_composites.len(),
                added_exclusions,
                added_quick_check.len()
            ),
            (12, 20, 20, 0, 21)
        );
    }
}
