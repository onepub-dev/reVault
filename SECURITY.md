# Security Policy

## Supported Versions

reVault is currently in early development. Security fixes are provided for the
latest published release only. Users should upgrade to the newest release
before reporting an issue that affects an older version.

| Version | Supported |
| --- | --- |
| Latest published release | :white_check_mark: |
| Earlier releases | :x: |

Security fixes may include changes that are not backward compatible while the
project remains pre-1.0.

## Reporting a Vulnerability

Please do not report suspected vulnerabilities in a public issue, discussion,
pull request, or other public channel.

Use GitHub's private vulnerability reporting for this repository:

1. Open the repository's **Security** tab.
2. Select **Advisories** and then **Report a vulnerability**.
3. Include the affected version and platform, the impact, reproduction steps
   or a proof of concept, and any suggested mitigation. Please omit real
   credentials, private keys, lockboxes, or other sensitive user data.

You can expect:

- acknowledgement within three business days;
- an initial assessment or status update within seven calendar days; and
- a status update at least every 14 days until the report is resolved or
  declined.

If the report is accepted, the maintainers will work with you on validation,
severity, remediation, release timing, and coordinated disclosure. Credit will
be offered unless you prefer to remain anonymous. If the report is declined,
the maintainers will explain why, where doing so does not create additional
security risk.

Please allow a reasonable remediation period before public disclosure. Good
faith research and reporting that avoids privacy violations, data destruction,
service disruption, and access beyond what is necessary to demonstrate the
issue will not be pursued by the project.

## Mirror safety

`lbx LOCKBOX mirror NAME update` is one-way. A wrong, incomplete, or
compromised host source can replace or remove files in the project's managed
lockbox directory. Inspect `mirror NAME status` before a destructive update.

reVault stores the canonical source path and available filesystem identity in
an encrypted mirror-project record, refuses filesystem roots, and requires
explicit overrides for empty selected sources and unusually large deletion
plans. A project exclusively owns one logical subtree, projects cannot overlap,
and core mutation APIs reject ordinary writes into those subtrees. These
controls reduce accidents; they cannot establish that the host source itself is
benign or uncompromised.

Mirror records use dot-prefixed normal variables under `/.revault/mirrors/`.
Ordinary variable listings hide them and exports always omit them;
`variable list --all` and exact `variable get` provide explicit inspection.
