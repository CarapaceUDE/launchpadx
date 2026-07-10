# Security Policy

## Supported Versions

Security fixes are provided for the latest release on the default branch.

| Version | Supported |
| ------- | --------- |
| 0.2.x   | Yes       |
| < 0.2   | No        |

## Reporting a Vulnerability

Please report security vulnerabilities privately to **[carapaceai@gmail.com](mailto:carapaceai@gmail.com)**.

Include:

- A description of the issue and its potential impact
- Steps to reproduce the behavior
- Affected versions or commit hashes, if known
- Any proof-of-concept code or logs that help us investigate

Do **not** open a public GitHub issue for security reports.

We aim to acknowledge reports within 3 business days and will work with you on remediation and coordinated disclosure when appropriate.

## Local Deployment Notes

This launcher stores API keys in plaintext in local config files (`config.json` and `~/.codex/config.toml`). Restrict file permissions on shared or multi-user systems, and prefer `envKey` mode or an external secret manager when handling sensitive credentials.
