//! Local interactive numeric experiment; no private data, network, or LLM.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "../src/reference.rs"]
mod reference;
use gel_phase_quad::{grid, Policy, Reader, Record};
use std::io::{BufRead, Read, Write};
use std::time::Instant;
const DIM: usize = grid::DIM;
#[derive(Clone, Copy, Default)]
struct Config {
    phase: u8,
    mask: usize,
    noise: usize,
}
fn command(c: &mut Config, line: &str) -> Result<bool, String> {
    let parts: Vec<_> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["quit"] => return Ok(false),
        ["show"] => {}
        ["phase", v] => c.phase = v.parse().map_err(|_| "phase must be 0..255")?,
        ["mask", v] => {
            let n = v.parse::<usize>().map_err(|_| "mask must be 0..1024")?;
            if n > DIM {
                return Err("mask must be 0..1024".into());
            }
            c.mask = n;
        }
        ["noise", v] => {
            let n = v.parse::<usize>().map_err(|_| "noise must be 0..1024")?;
            if n > DIM {
                return Err("noise must be 0..1024".into());
            }
            c.noise = n;
        }
        _ => {
            return Err(
                "commands: phase 0..255 | mask 0..1024 | noise 0..1024 | show | quit".into(),
            )
        }
    }
    Ok(true)
}
fn evaluate(c: Config) -> Option<f64> {
    let reader = Reader::new(510051);
    let g = grid::Grid::new(256, 510051).unwrap();
    let a = Record::new(std::array::from_fn(|j| j as u8), &[true; DIM]);
    let phase = std::array::from_fn(|j| {
        (j as u8)
            .wrapping_add(c.phase)
            .wrapping_add(if j < c.noise {
                (j as u8).wrapping_mul(73).wrapping_add(19)
            } else {
                0
            })
    });
    let b = Record::new(
        phase,
        &std::array::from_fn(|j| c.mask != 0 && j % c.mask == 0),
    );
    let query = reader.prepare(a.clone());
    let start = Instant::now();
    let score = reader.read(
        std::hint::black_box(&query),
        std::hint::black_box(&b),
        Policy::BodyActivity,
    );
    let elapsed = start.elapsed().as_nanos();
    let fa = reference::frames(a.phase(), &a.active_mask(), &g);
    let fb = reference::frames(b.phase(), &b.active_mask(), &g);
    let cos: [f64; 256] = std::array::from_fn(|j| (std::f64::consts::TAU * j as f64 / 256.).cos());
    println!(
        "phase={} mask_stride={} noise_dimensions={} policy=BodyActivity",
        c.phase, c.mask, c.noise
    );
    for pole in 0..4 {
        let reference = reference::score(
            &fa[pole],
            &fb[pole],
            &reference::indices(&fa[pole]),
            true,
            &cos,
        )
        .unwrap();
        let shared = score.unwrap().at_pole(pole as u8).unwrap();
        assert_eq!(reference.to_bits(), shared.to_bits());
        let restored = reader
            .restore_bound_view(&reader.bound_view(&b, pole as u8).unwrap())
            .unwrap();
        assert_eq!(restored.phase(), b.phase());
        assert_eq!(restored.active_mask(), b.active_mask());
        println!("P{pole}: reference={reference:.9} shared={shared:.9} inverse=PASS");
    }
    println!("single_shared_read_ns={elapsed} (one observation, NOT a benchmark)");
    println!(
        "canonical_pair_bytes={} materialized_pair_bytes={} (payload only, NOT total RAM)",
        2 * std::mem::size_of::<Record>(),
        2 * std::mem::size_of::<[reference::Frame; 4]>()
    );
    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        if let Some(rss) = status.lines().find(|l| l.starts_with("VmRSS:")) {
            println!("process_{rss} (includes runtime and demo buffers)");
        }
    }
    #[cfg(not(target_os = "linux"))]
    println!("process_RSS=NOT_MEASURED");
    println!("QUAD_PLAYGROUND=PASS; four equivalent views, not four independent facts");
    score.map(|x| x.value())
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut c = Config {
        mask: 1,
        ..Config::default()
    };
    if args == ["--demo"] {
        evaluate(c);
        return std::process::ExitCode::SUCCESS;
    }
    if args != ["--interactive"] {
        eprintln!("usage: quad_playground --demo | --interactive");
        return std::process::ExitCode::from(2);
    }
    println!("GEL Q8 numeric playground. phase N | mask N (0=empty) | noise N | show | quit");
    evaluate(c);
    let mut input = std::io::stdin().lock();
    loop {
        print!("> ");
        if std::io::stdout().flush().is_err() {
            return std::process::ExitCode::from(2);
        }
        let mut line = String::new();
        match (&mut input).take(1025).read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if line.len() <= 1024 => {}
            _ => {
                eprintln!("input error or line exceeds 1024 bytes");
                return std::process::ExitCode::from(2);
            }
        }
        match command(&mut c, &line) {
            Ok(false) => break,
            Ok(true) => {
                evaluate(c);
            }
            Err(e) => eprintln!("{e}"),
        }
    }
    std::process::ExitCode::SUCCESS
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phase_and_mask_behaviour() {
        assert_eq!(
            evaluate(Config {
                phase: 0,
                mask: 1,
                noise: 0
            }),
            Some(1.)
        );
        assert_eq!(
            evaluate(Config {
                phase: 128,
                mask: 1,
                noise: 0
            }),
            Some(-1.)
        );
        assert_eq!(evaluate(Config::default()), Some(0.));
        evaluate(Config {
            phase: 255,
            mask: 3,
            noise: 1024,
        });
    }
    #[test]
    fn invalid_commands_preserve_state() {
        let mut c = Config {
            mask: 1,
            ..Config::default()
        };
        for line in ["phase 256", "mask 1025", "noise -1", "show extra"] {
            assert!(command(&mut c, line).is_err());
        }
        assert_eq!((c.phase, c.mask, c.noise), (0, 1, 0));
        assert!(!command(&mut c, "quit").unwrap());
    }
}
