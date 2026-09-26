//! New-bank construction only, not searches or live mutations. CSV to stdout.
use gel_source::{
    collection::{Collection, CollectionBuilder},
    digest, hex,
};
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    eprintln!("SCOPE=new bank construction; one thread; input allocation/oracle outside timer; paired alternating order; sample0 warmup omitted");
    eprintln!(
        "COLLECTION_SHA256={}",
        hex(&digest(include_bytes!("../src/collection.rs")))
    );
    eprintln!(
        "HARNESS_SHA256={}",
        hex(&digest(include_bytes!("collection_build_compare.rs")))
    );
    println!("documents,sample,order,variant,latency_ns,result_bytes,result_sha256,correct");
    for n in [8, 64, 256] {
        let inputs: Vec<_> = (0..n)
            .map(|i| {
                (
                    format!("Document {i}"),
                    format!(
                        "Synthetic document {i}.\n{}",
                        "Zażółć gęślą jaźń. Source bytes, not claims.\n".repeat(400)
                    ),
                )
            })
            .collect();
        for sample in 0..31 {
            let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
            let mut outputs = Vec::new();
            for variant in order {
                let start = Instant::now();
                let bank = if variant == 0 {
                    let mut c = Collection::new();
                    for (title, text) in &inputs {
                        c.add(black_box(title), black_box(text))?;
                    }
                    c
                } else {
                    let mut b = CollectionBuilder::new();
                    for (title, text) in &inputs {
                        b.add(black_box(title), black_box(text))?;
                    }
                    b.build()
                };
                black_box(bank.root());
                let elapsed = start.elapsed().as_nanos();
                outputs.push((variant, elapsed, bank));
            }
            let bytes = outputs[0].2.to_bytes();
            let pin = digest(&bytes);
            if bytes != outputs[1].2.to_bytes() || outputs.iter().any(|(_, _, b)| b.root() != pin) {
                return Err("BUILD_EQUIVALENCE_FAILED".into());
            }
            let loaded = Collection::from_bytes(&bytes, pin)?;
            if loaded.to_bytes() != bytes {
                return Err("REOPEN_FAILED".into());
            }
            if sample != 0 {
                for (position, (variant, elapsed, _)) in outputs.iter().enumerate() {
                    println!(
                        "{n},{sample},{position},{},{elapsed},{},{},1",
                        if *variant == 0 {
                            "sequential"
                        } else {
                            "builder"
                        },
                        bytes.len(),
                        hex(&pin)
                    );
                }
            }
        }
    }
    eprintln!("BUILD_COMPARE=PASS");
    Ok(())
}
