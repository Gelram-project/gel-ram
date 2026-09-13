// Compile with rustc --edition=2021. No external dependencies.
// Args: example-binary directory, NEW output directory, optional caller-owned Q8DEMO01 bank.
use std::{fs, io::Write, path::Path, process::Command};
fn run(binary: &Path, args: &[String], output: &Path, marker: &str) {
    let result = Command::new(binary).args(args).output().expect("launch benchmark");
    let mut log = fs::OpenOptions::new().create_new(true).write(true).open(output).unwrap();
    log.write_all(&result.stdout).unwrap();
    log.write_all(&result.stderr).unwrap();
    assert!(result.status.success(), "benchmark failed");
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains(marker), "missing successful validation");
    assert!(!text.contains("DEGRADED"), "worker fallback: do not treat as full-worker timing");
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() == 3 || args.len() == 4, "expected BIN_DIR NEW_OUTPUT [PRIVATE_INPUT]");
    let output = Path::new(&args[2]);
    fs::create_dir(output).expect("output must not exist");
    let compare = Path::new(&args[1]).join("quad_compare");
    let evidence = Path::new(&args[1]).join("quad_evidence");
    for n in [32, 512, 8192] {
        run(&evidence, &["--generate".into(), output.join(format!("synthetic-{n}.q8demo")).to_str().unwrap().into(), n.to_string()], &output.join(format!("generate-{n}.txt")), "CREATED");
    }
    let cells: Vec<_> = [512,8192].into_iter().flat_map(|n| [0,1].into_iter().flat_map(move |s| ["archive","active"].into_iter().flat_map(move |p| [1,24].into_iter().map(move |w| (n,s,p,w))))).collect();
    for rep in 0..3 {
        for j in 0..cells.len() {
            let (n,s,p,w) = cells[(j * 5 + rep * 3) % cells.len()];
            run(&compare, &["--orbs".into(),n.to_string(),"--rounds".into(),"9".into(),"--workers".into(),w.to_string(),"--sparse".into(),s.to_string(),"--policy".into(),p.into()], &output.join(format!("v2-{rep}-{n}-{s}-{p}-{w}.txt")), "Q8_QUAD_EXACT=PASS");
        }
        let mut banks: Vec<_> = [32,512,8192].into_iter().map(|n| (format!("synthetic{n}"), output.join(format!("synthetic-{n}.q8demo")))).collect();
        if let Some(path) = args.get(3) { banks.push(("external".into(), Path::new(path).to_path_buf())); }
        for (label,path) in banks {
            for p in if rep % 2 == 0 { ["active","archive"] } else { ["archive","active"] } {
                run(&evidence, &[path.to_str().unwrap().into(),"9".into(),p.into()], &output.join(format!("evidence-{rep}-{label}-{p}.txt")), "Q8_EVIDENCE=PASS");
            }
        }
        println!("repetition {} complete", rep + 1);
    }
}
