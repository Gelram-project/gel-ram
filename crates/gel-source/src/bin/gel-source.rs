//! Local exact-source CLI. No model, network client, private banks or semantic claims.
use gel_source::{hex, import_text, load_bundle, read_regular, write_bundle_new, Hash, MAX_TEXT};
use std::{path::Path, time::Instant};

fn pin(raw: &str) -> Result<Hash, String> {
    if raw.len() != 64 || !raw.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("expected 64 hex digits for trusted SHA256".into());
    }
    let mut out = [0; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&raw[2 * i..2 * i + 2], 16).map_err(|_| "invalid pin")?;
    }
    Ok(out)
}
fn run(args: &[String]) -> Result<(), String> {
    match args {
        [cmd, input, output, title] if cmd == "build" => {
            let start = Instant::now();
            let bytes =
                read_regular(Path::new(input), MAX_TEXT).map_err(|e| format!("input: {e:?}"))?;
            let encoded = import_text(&bytes, title).map_err(|e| format!("import: {e:?}"))?;
            let hash = write_bundle_new(Path::new(output), &encoded)
                .map_err(|e| format!("export: {e:?}"))?;
            println!("BUNDLE_SHA256={}\nTEXT_BYTES={}\nCATALOG_BYTES={}\nENVELOPE_BYTES=24\nBUILD_MS={:.3}\nBUILD=PASS", hex(&hash), bytes.len(), encoded.catalog_bytes().len(), start.elapsed().as_secs_f64()*1000.0);
            println!("Keep this pin separately. Plaintext source catalog; not encrypted ORBs or semantic understanding.");
            Ok(())
        }
        [cmd, file, hash, title] if cmd == "read" => {
            let start = Instant::now();
            let corpus =
                load_bundle(Path::new(file), pin(hash)?).map_err(|e| format!("load: {e:?}"))?;
            let parts = corpus
                .lead_parts(title, None)
                .map_err(|e| format!("title: {e:?}"))?;
            let elapsed = start.elapsed();
            for p in &parts {
                // Escape terminal controls from arbitrary untrusted user text.
                println!(
                    "PART={} QUOTE_ESCAPED={:?}",
                    p.record().address.role,
                    p.quote()
                );
            }
            println!(
                "PARTS={}\nLOAD_LOOKUP_MS={:.3}\nREAD=PASS",
                parts.len(),
                elapsed.as_secs_f64() * 1000.0
            );
            println!(
                "Timing excludes terminal output. Exact title lookup, not AI generation or ORB/s."
            );
            Ok(())
        }
        [arg] if arg == "--help" => {
            println!("gel-source build INPUT.txt NEW_OUTPUT.gelsrc TITLE\ngel-source read INPUT.gelsrc TRUSTED_SHA256 TITLE\nOffline, bounded UTF-8, no overwrite. Use a directory you control. Output is plaintext.");
            Ok(())
        }
        _ => Err("usage: gel-source --help".into()),
    }
}
fn main() {
    if let Err(e) = run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        eprintln!("GEL_SOURCE=FAIL {e}");
        std::process::exit(1);
    }
}
