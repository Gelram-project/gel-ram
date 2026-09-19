//! Deterministic synthetic source workload. Not a semantic or Q8 benchmark.
use gel_source::{collection::Collection, hex};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    hint::black_box,
    io::Write,
    path::Path,
    time::Instant,
};
fn create(path: &Path) -> Result<std::fs::File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())
}
fn line(file: &mut std::fs::File, s: &str) -> Result<(), String> {
    writeln!(file, "{s}").map_err(|e| e.to_string())
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err("collection_campaign NEW_OUTPUT_DIRECTORY".into());
    }
    let out = Path::new(&args[0]);
    fs::create_dir(out).map_err(|e| e.to_string())?;
    let mut raw = create(&out.join("raw.csv"))?;
    let mut summary = create(&out.join("summary.csv"))?;
    let mut manifests = create(&out.join("corpora.txt"))?;
    line(
        &mut raw,
        "rep,documents,operation,sample,latency_ns,correct",
    )?;
    line(
        &mut summary,
        "rep,documents,operation,n,min_ns,p50_ns,p95_ns,p99_ns,max_ns,total_ns",
    )?;
    let mut hardware = create(&out.join("environment.txt"))?;
    line(&mut hardware,&format!("os={} arch={} threads=1 available_parallelism={}\nRUNTIME=CPU_ONLY_OFFLINE\nSCOPE=generated UTF-8 phrase matching, warm process; NOT semantic recall, Q8 ORB/s or cold DRAM\nTIMERS=search includes returned quote allocation; mutations include root serialization/hash; save includes filesystem sync; load includes validation\nQUERIES=EXACT NEAR MISS UNICODE; no statistical p99.9 claims\nMEMORY=Linux VmRSS and VmHWM are process-wide snapshots, HWM cumulative, not per-bank peak",std::env::consts::OS,std::env::consts::ARCH,std::thread::available_parallelism().map_or(1,usize::from)))?;
    for rep in 0..3 {
        for n in if rep % 2 == 0 {
            [8, 64, 256]
        } else {
            [256, 64, 8]
        } {
            let mut c = Collection::new();
            let build = Instant::now();
            for id in 0..n {
                let text = format!(
                    "Document key{id}.\nZażółć gęślą jaźń record{id}.\n{}End of source key{id}.\n",
                    "A neutral filler sentence with no requested identifier.\n".repeat(128)
                );
                c.add(&format!("Source {id}"), &text)?;
            }
            let build_ns = build.elapsed().as_nanos();
            let encoded = c.to_bytes();
            line(&mut manifests,&format!("rep={rep} documents={n} sha256={} text_bytes={} snapshot_bytes={} build_ns={build_ns}",hex(&c.root()),c.text_bytes(),encoded.len()))?;
            #[cfg(target_os = "linux")]
            if let Ok(s) = fs::read_to_string("/proc/self/status") {
                for l in s
                    .lines()
                    .filter(|l| l.starts_with("VmRSS:") || l.starts_with("VmHWM:"))
                {
                    line(&mut hardware, &format!("rep={rep} documents={n} {l}"))?;
                }
            }
            let mut times: BTreeMap<&str, Vec<u128>> = BTreeMap::new();
            // Every corpus is warmed on one separately labelled query.
            black_box(c.search("neutral filler")?);
            for sample in 0..50 {
                let id = (sample * 37 + rep * 11) % n;
                for (op, q, expected) in [
                    ("exact", format!("document key{id}"), 1),
                    ("near", format!("document key{id}x"), 0),
                    ("miss", "absent identifier xyz987".into(), 0),
                    ("unicode", format!("zażółć gęślą jaźń record{id}"), 1),
                ] {
                    let t = Instant::now();
                    let found = black_box(c.search(black_box(&q))?);
                    let ns = t.elapsed().as_nanos();
                    if found.matching_lines != expected
                        || found.skipped_long_lines != 0
                        || found.documents_examined != n
                    {
                        return Err("SEARCH_ORACLE".into());
                    }
                    for h in &found.hits {
                        c.validate(h)?;
                        if h.document_id() != id as u64 + 1 {
                            return Err("ID_ORACLE".into());
                        }
                    }
                    times.entry(op).or_default().push(ns);
                    line(&mut raw, &format!("{rep},{n},{op},{sample},{ns},1"))?;
                }
            }
            for sample in 0..5 {
                for op in [
                    "serialize",
                    "deserialize",
                    "insert",
                    "update",
                    "delete",
                    "save",
                    "load",
                ] {
                    let path = out.join(format!("snapshot-{rep}-{n}-{sample}.gelset"));
                    let t = Instant::now();
                    let ns = match op {
                        "serialize" => {
                            let b = black_box(c.to_bytes());
                            let ns = t.elapsed().as_nanos();
                            if b != encoded {
                                return Err("SERIALIZE_ORACLE".into());
                            }
                            ns
                        }
                        "deserialize" => {
                            let d = Collection::from_bytes(&encoded, c.root())?;
                            let ns = t.elapsed().as_nanos();
                            if d.to_bytes() != encoded {
                                return Err("DECODE_ORACLE".into());
                            }
                            ns
                        }
                        "insert" => {
                            let mut d = c.clone();
                            let t = Instant::now();
                            let id = d.add("temporary", "test-only data")?;
                            let ns = t.elapsed().as_nanos();
                            if d.get(id).is_none() {
                                return Err("INSERT_ORACLE".into());
                            }
                            ns
                        }
                        "update" => {
                            let mut d = c.clone();
                            let t = Instant::now();
                            d.replace(1, "replacement text")?;
                            let ns = t.elapsed().as_nanos();
                            if d.get(1).unwrap().text() != "replacement text" {
                                return Err("UPDATE_ORACLE".into());
                            }
                            ns
                        }
                        "delete" => {
                            let mut d = c.clone();
                            let t = Instant::now();
                            d.remove(1)?;
                            let ns = t.elapsed().as_nanos();
                            if d.get(1).is_some() {
                                return Err("DELETE_ORACLE".into());
                            }
                            ns
                        }
                        "save" => {
                            let pin = c.save_new(&path)?;
                            let ns = t.elapsed().as_nanos();
                            if pin != c.root() {
                                return Err("SAVE_ORACLE".into());
                            }
                            ns
                        }
                        "load" => {
                            let d = Collection::load(&path, c.root())?;
                            let ns = t.elapsed().as_nanos();
                            if d.to_bytes() != encoded {
                                return Err("REOPEN_ORACLE".into());
                            }
                            ns
                        }
                        _ => unreachable!(),
                    };
                    times.entry(op).or_default().push(ns);
                    line(&mut raw, &format!("{rep},{n},{op},{sample},{ns},1"))?;
                }
            }
            for (op, mut v) in times {
                v.sort_unstable();
                let len = v.len();
                let p = |q: usize| v[(len * q).div_ceil(100) - 1];
                let total: u128 = v.iter().sum();
                line(
                    &mut summary,
                    &format!(
                        "{rep},{n},{op},{len},{},{},{},{},{},{total}",
                        v[0],
                        p(50),
                        p(95),
                        p(99),
                        v[len - 1]
                    ),
                )?;
            }
        }
    }
    raw.sync_all().map_err(|e| e.to_string())?;
    summary.sync_all().map_err(|e| e.to_string())?;
    line(
        &mut create(&out.join("COMPLETE"))?,
        "COLLECTION_CAMPAIGN=PASS; oracle is generated exact-source matching, not semantic truth",
    )?;
    println!("COLLECTION_CAMPAIGN=PASS");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("COLLECTION_CAMPAIGN=FAIL {e}");
        std::process::exit(1);
    }
}
