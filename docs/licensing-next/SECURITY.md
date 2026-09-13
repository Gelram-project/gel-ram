# GEL RAM security policy after the next-license cutover

> STAGED FOR THE NEXT RELEASE. NOT CURRENTLY IN FORCE.

This policy applies only after the next licensing package is activated for a future GEL RAM release.

## Supported code

Security fixes, when made, target the currently maintained release line and the current development branch as determined by the project at the time of the report.

Historical releases may remain available for reproducibility, but availability does not imply ongoing security support.

No response time, remediation deadline, service level, warranty, or support commitment is promised by this public policy.

## Reporting vulnerabilities

Use GitHub Private Vulnerability Reporting when it is enabled for the repository. Otherwise contact `gelram.licensing@gmail.com` privately.

Do not disclose an unpatched vulnerability, exploit details, credentials, private datasets, personal data, or other sensitive information in a public issue, discussion, pull request, or benchmark report.

A useful report should include, where safely possible:

- the affected release or exact commit;
- affected component and platform;
- impact and realistic attack preconditions;
- minimal reproduction steps using non-sensitive synthetic data;
- whether the issue has already been disclosed elsewhere;
- suggested mitigation if known.

## Disclosure

The project may coordinate remediation and public disclosure with the reporter. Public disclosure timing depends on the specific issue and does not create a guaranteed embargo period or response schedule.

## Security and licensing are separate

Reporting or fixing a vulnerability does not change the applicable software license. A commercial user still requires the commercial rights applicable to its use.

A security contribution intended for incorporation remains subject to the operative contributor-rights policy and CLA unless the Project Licensor expressly agrees to another rights basis in writing.

## Trust boundaries

GEL RAM consumers must continue to treat external files, data, manifests, indexes, and other inputs as untrusted unless independently authenticated.

Checksums and CRCs detect classes of corruption but do not automatically provide authenticity. Cryptographic authentication, signatures, access controls, sandboxing, resource limits, and deployment-specific controls remain the responsibility of the system crossing the relevant trust boundary unless the release expressly provides them.

## No implied certification

A passing project test, verification gate, fuzzing campaign, source hash, or security review is evidence for its stated scope only. It is not a certification that GEL RAM is secure for every environment or threat model.
