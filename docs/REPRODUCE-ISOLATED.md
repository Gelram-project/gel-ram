# Reproduce in an isolated network namespace

Cargo's `--offline` and `--locked` stop Cargo itself from downloading, but they
do not stop a spawned test or example from opening a socket. A run whose report
claims network isolation therefore needs an operating-system block, and the
repository checks that block from inside before anything else runs.

## 1. Prepare the dependencies (network needed)

```text
git clone https://github.com/Gelram-project/gel-ram.git
cd gel-ram
rustup toolchain install 1.85.0 --profile minimal
cargo fetch --locked
cargo build --locked --offline -p xtask
```

## 2. Enter a network namespace (Linux)

Either of:

```text
unshare --user --net -- bash
bwrap --dev-bind / / --unshare-net -- bash
```

Do not add `--map-root-user`: root, also root mapped inside a user namespace,
bypasses file permissions, and the permission-denial tests then fail for that
reason alone. `xtask reproduce` refuses to run as root. Some distributions
restrict unprivileged user namespaces (on Ubuntu the AppArmor setting
kernel.apparmor_restrict_unprivileged_userns). Then an administrator has to
allow them for this run.

## 3. Prove the isolation

```text
cargo run --locked --offline -p xtask -- isolation-check
```

Expected:

```text
NETWORK_ISOLATION=VERIFIED loopback only, no routes, probes unreachable
DEPENDENCIES_OFFLINE=READY
```

The check reads the process's own network view, not the host's:

1. the network-device list shows loopback only;
2. neither the IPv4 nor the IPv6 routing table has a route through another device;
3. only then, TCP connections to the documentation addresses 203.0.113.1 and
   2001:db8::1 must fail immediately as "network unreachable". A refusal or a
   timeout would mean a route existed and fails the check. On a networked host
   step 3 is never reached, so the check sends no packets there;
4. every locked dependency must already be in the local Cargo cache.

The system's /sys/class/net view is deliberately not used: inside a new network
namespace it can still list the host's interfaces unless sysfs is remounted.

Negative control: the same command outside the namespace, on a networked host,
must print NETWORK_ISOLATION=NOT_VERIFIED with the reason and exit nonzero.
Linux CI runs both the negative and the positive control on every revision.

## 4. Run the report inside the same namespace

```text
cargo run --locked --offline -p xtask -- report ../gel-report-isolated
```

The report's hardware file records network_isolation=VERIFIED, or NOT_VERIFIED
with the reason, and the host load average at the start; a separate file records
it at the end. Timings from a busy host are still valid records, but they must
be read with that load.

## Limits

- Linux only. macOS and Windows report NOT_VERIFIED: no namespace evidence is
  implemented for them.
- The check proves the network state of this process tree at the moment it runs.
  It does not restrict file-system access and does not rule out other side
  channels.
- A report counts as isolated only if it was started inside the namespace that
  passed the check.
