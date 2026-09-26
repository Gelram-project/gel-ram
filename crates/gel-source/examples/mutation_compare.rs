//! Paired whole-mutation benchmark, historical Vec root versus current stream root.
//! The pinned historical modules are compiled unchanged; their tests are repetitions.
pub use gel_source::{
    digest, document, Address, Corpus, CorpusBuilder, EncodedCorpus, Error, Hash, MAX_CATALOG,
    MAX_PASSAGE, MAX_TEXT,
};
#[allow(dead_code)]
#[path = "../../../docs/evidence-collection/bundle.measured.rs.txt"]
mod bundle;
pub use bundle::read_regular;
#[allow(dead_code)]
#[path = "../../../docs/evidence-collection/collection.measured.rs.txt"]
mod historical;
use gel_source::{collection::Collection, hex};
use std::{fs, hint::black_box, io::Write, path::Path, time::Instant};

fn run(out: &Path) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err("use --release for this comparison".into());
    }
    let mut groups = std::collections::BTreeMap::<(usize, &str, usize), Vec<u128>>::new();
    fs::create_dir(out).map_err(|e| e.to_string())?;
    let mut raw = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("raw.csv"))
        .map_err(|e| e.to_string())?;
    writeln!(
        raw,
        "documents,sample,order,operation,variant,input_bytes,latency_ns,result_root,correct"
    )
    .map_err(|e| e.to_string())?;
    let mut identity = String::from("SCOPE=paired warm whole mutations; cloning and oracle outside timers\nTHREADS=1\nSAMPLES_PER_OPERATION_AND_SIZE=30\nALLOCATION_COUNTS=NOT_MEASURED\nRSS=NOT_MEASURED\nCOPY_COST=historical root constructs full serialized Vec; current root streams, not incremental\n");
    identity.push_str(&format!(
        "os={} arch={} profile=release available_parallelism={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(1, usize::from)
    ));
    for (name, bytes) in [
        (
            "historical_collection",
            include_bytes!("../../../docs/evidence-collection/collection.measured.rs.txt")
                .as_slice(),
        ),
        (
            "current_collection",
            include_bytes!("../src/collection.rs").as_slice(),
        ),
        ("harness", include_bytes!("mutation_compare.rs").as_slice()),
    ] {
        identity.push_str(&format!("{name}_sha256={}\n", hex(&digest(bytes))));
    }
    for n in [8, 64, 256] {
        let mut base = Collection::new();
        let text = "Synthetic source data. Zażółć gęślą jaźń.\n".repeat(400);
        for i in 0..n {
            base.add(&format!("Document {i}"), &text)?;
        }
        let bytes = base.to_bytes();
        let old = historical::Collection::from_bytes(&bytes, base.root())?;
        identity.push_str(&format!(
            "documents={n} input_bytes={} input_sha256={}\n",
            bytes.len(),
            hex(&base.root())
        ));
        let replacement = format!("Revised synthetic source.\n{text}");
        for sample in 0..31 {
            for op in ["add", "replace", "remove"] {
                // Two independent equivalent states. Alternating order limits systematic
                // first/second-run bias but does not isolate cache or allocator effects.
                let mut current = base.clone();
                let mut reference = old.clone();
                let mut timing = [0_u128; 2];
                let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
                for variant in order {
                    let t = Instant::now();
                    if variant == 0 {
                        match op {
                            "add" => {
                                black_box(current.add("Extra", black_box(&replacement))?);
                            }
                            "replace" => current.replace(1, black_box(&replacement))?,
                            _ => current.remove(1)?,
                        }
                        black_box(current.root());
                    } else {
                        match op {
                            "add" => {
                                black_box(reference.add("Extra", black_box(&replacement))?);
                            }
                            "replace" => reference.replace(1, black_box(&replacement))?,
                            _ => reference.remove(1)?,
                        }
                        black_box(reference.root());
                    }
                    timing[variant] = t.elapsed().as_nanos();
                }
                let result = current.to_bytes();
                if result != reference.to_bytes()
                    || current.root() != reference.root()
                    || digest(&result) != current.root()
                {
                    return Err("MUTATION_ORACLE_MISMATCH".into());
                }
                if sample == 0 {
                    continue;
                } // separate unrecorded warm-up per size/operation
                for (position, variant) in order.into_iter().enumerate() {
                    groups
                        .entry((n, op, variant))
                        .or_default()
                        .push(timing[variant]);
                    writeln!(
                        raw,
                        "{n},{sample},{position},{op},{},{},{},{},1",
                        if variant == 0 {
                            "stream"
                        } else {
                            "historical_vec"
                        },
                        bytes.len(),
                        timing[variant],
                        hex(&current.root())
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    raw.sync_all().map_err(|e| e.to_string())?;
    let mut summary =
        String::from("documents,operation,variant,n,min_ns,p50_ns,p95_ns,p99_ns,max_ns,total_ns\n");
    for ((n, op, variant), mut times) in groups {
        times.sort_unstable();
        if times.len() != 30 {
            return Err("incomplete timing group".into());
        }
        summary.push_str(&format!(
            "{n},{op},{},30,{},{},{},{},{},{}\n",
            if variant == 0 {
                "stream"
            } else {
                "historical_vec"
            },
            times[0],
            times[14],
            times[28],
            times[29],
            times[29],
            times.iter().sum::<u128>()
        ));
    }
    fs::write(out.join("summary.csv"), summary).map_err(|e| e.to_string())?;
    fs::write(out.join("inputs.txt"), identity).map_err(|e| e.to_string())?;
    fs::write(out.join("COMPLETE"), "MUTATION_COMPARE=PASS\n").map_err(|e| e.to_string())?;
    println!("MUTATION_COMPARE=PASS");
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        eprintln!("mutation_compare NEW_DIRECTORY");
        std::process::exit(2);
    }
    if let Err(e) = run(Path::new(&args[0])) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
