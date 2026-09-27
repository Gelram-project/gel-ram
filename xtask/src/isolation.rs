//! Network-isolation evidence for local reproduction runs.
//!
//! Cargo's `--offline` and `--locked` flags do not stop a spawned process from
//! opening a socket. This check reads the calling process's own network
//! namespace, and only when that already shows loopback alone does it confirm
//! that non-loopback connections fail as unreachable. On a networked host the
//! active probe is never reached, so the check sends no packets there.
use std::{io, net::TcpStream};

const PROBES: [&str; 2] = ["203.0.113.1:9", "[2001:db8::1]:9"];

/// Interface names from a `/proc/net/dev` listing (two header lines).
fn interfaces(dev: &str) -> Vec<&str> {
    dev.lines()
        .skip(2)
        .filter_map(|line| line.split_once(':'))
        .map(|(name, _)| name.trim())
        .collect()
}

/// Output devices of the IPv4 routing table (one header line, device first).
fn ipv4_route_devices(route: &str) -> Vec<&str> {
    route
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .collect()
}

/// Output devices of the IPv6 routing table (no header, device last).
fn ipv6_route_devices(route: &str) -> Vec<&str> {
    route
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .collect()
}

fn passive(dev: &str, route4: &str, route6: &str) -> Result<(), String> {
    let names = interfaces(dev);
    if names != ["lo"] {
        return Err(format!(
            "interfaces other than loopback: {}",
            names.join(",")
        ));
    }
    let routed: Vec<&str> = ipv4_route_devices(route4)
        .into_iter()
        .chain(ipv6_route_devices(route6))
        .filter(|device| *device != "lo")
        .collect();
    if !routed.is_empty() {
        return Err(format!("routes through: {}", routed.join(",")));
    }
    Ok(())
}

/// Only an immediate "network unreachable" proves the probe could not leave;
/// a refusal or a timeout means a route existed.
fn unreachable(target: &str, attempt: io::Result<()>) -> Result<(), String> {
    match attempt {
        Ok(()) => Err(format!("{target}: connection succeeded")),
        Err(e) if e.kind() == io::ErrorKind::NetworkUnreachable => Ok(()),
        Err(e) => Err(format!("{target}: {e} (expected network unreachable)")),
    }
}

#[cfg(target_os = "linux")]
fn evidence() -> Result<(), String> {
    use std::{fs, net::SocketAddr, time::Duration};
    let read = |path: &str| fs::read_to_string(path).map_err(|e| format!("{path}: {e}"));
    let dev = read("/proc/self/net/dev")?;
    let route4 = read("/proc/self/net/route")?;
    // Absent when IPv6 is disabled; then there is no IPv6 route to check.
    let route6 = fs::read_to_string("/proc/self/net/ipv6_route").unwrap_or_default();
    passive(&dev, &route4, &route6)?;
    let ipv6 = !fs::read_to_string("/proc/self/net/if_inet6")
        .unwrap_or_default()
        .trim()
        .is_empty();
    for target in PROBES.iter().take(if ipv6 { 2 } else { 1 }) {
        let addr: SocketAddr = target.parse().map_err(|e| format!("{target}: {e}"))?;
        unreachable(
            target,
            TcpStream::connect_timeout(&addr, Duration::from_secs(1)).map(drop),
        )?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn evidence() -> Result<(), String> {
    Err(format!(
        "no network-namespace evidence is implemented for {}",
        std::env::consts::OS
    ))
}

/// One line for reports that people share: VERIFIED or NOT_VERIFIED with a
/// reason that counts interfaces and routes instead of naming them, since host
/// interface names reveal the local network setup.
pub fn status() -> String {
    match evidence() {
        Ok(()) => "network_isolation=VERIFIED".into(),
        Err(reason) => format!("network_isolation=NOT_VERIFIED ({})", redact(&reason)),
    }
}

fn redact(reason: &str) -> String {
    for (prefix, noun) in [
        (
            "interfaces other than loopback: ",
            "network interfaces including loopback",
        ),
        ("routes through: ", "routes through other devices"),
    ] {
        if let Some(names) = reason.strip_prefix(prefix) {
            return format!("{} {noun}", names.split(',').count());
        }
    }
    reason.to_string()
}

pub fn check(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: cargo run -p xtask -- isolation-check".into());
    }
    evidence().map_err(|reason| format!("NETWORK_ISOLATION=NOT_VERIFIED {reason}"))?;
    println!("NETWORK_ISOLATION=VERIFIED loopback only, no routes, probes unreachable");
    // Dependencies must already be in the local cache; nothing can be fetched here.
    super::run("cargo", &["fetch", "--locked", "--offline"])
        .map_err(|e| format!("DEPENDENCIES_OFFLINE=MISSING {e}"))?;
    println!("DEPENDENCIES_OFFLINE=READY");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "Inter-|   Receive\n face |bytes\n";

    fn dev(names: &[&str]) -> String {
        let mut listing = String::from(HEADER);
        for name in names {
            listing.push_str("  ");
            listing.push_str(name);
            listing.push_str(": 0 0\n");
        }
        listing
    }

    #[test]
    fn loopback_alone_without_routes_passes() {
        assert_eq!(passive(&dev(&["lo"]), "Iface\tDestination\n", ""), Ok(()));
    }

    #[test]
    fn a_host_with_network_interfaces_is_not_isolated() {
        let host = dev(&["lo", "eth0", "docker0"]);
        let error = passive(&host, "Iface\tDestination\n", "").unwrap_err();
        assert!(error.contains("eth0"), "{error}");
    }

    #[test]
    fn a_route_through_another_device_fails_even_with_loopback_listed_alone() {
        let v4 = "Iface\tDestination\neth0\t00000000\n";
        assert!(passive(&dev(&["lo"]), v4, "").is_err());
        let v6 = "00000000000000000000000000000000 00 0 0 0 0 0 0 0 wlan0\n";
        assert!(passive(&dev(&["lo"]), "Iface\n", v6).is_err());
        let lo6 = "00000000000000000000000000000001 80 0 0 0 0 0 0 0 lo\n";
        assert_eq!(passive(&dev(&["lo"]), "Iface\n", lo6), Ok(()));
    }

    #[test]
    fn shared_reports_count_interfaces_instead_of_naming_them() {
        let named = passive(&dev(&["lo", "eth0", "vpn0"]), "Iface\n", "").unwrap_err();
        let shared = redact(&named);
        assert_eq!(shared, "3 network interfaces including loopback");
        assert!(!shared.contains("eth0") && !shared.contains("vpn0"));
        let routed = passive(&dev(&["lo"]), "Iface\tDestination\nwlan0\t0\n", "").unwrap_err();
        assert_eq!(redact(&routed), "1 routes through other devices");
    }

    #[test]
    fn only_network_unreachable_counts_as_blocked() {
        let unreachable_error = io::Error::from(io::ErrorKind::NetworkUnreachable);
        assert_eq!(unreachable("t", Err(unreachable_error)), Ok(()));
        for kind in [
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::TimedOut,
            io::ErrorKind::PermissionDenied,
        ] {
            assert!(unreachable("t", Err(io::Error::from(kind))).is_err());
        }
        // A successful connection is the counterexample. It is given as an
        // outcome, not opened: inside a fresh network namespace even loopback
        // is down, and the test must pass there too.
        assert!(unreachable("t", Ok(())).is_err());
    }
}
