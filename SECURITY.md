# Security Policy

## Supported versions

dosui is pre-1.0 software. Security fixes are made against the latest release
and the `main` branch only.

| Version | Supported |
| ------- | --------- |
| latest  | ✅        |
| older   | ❌        |

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Instead, report them privately via GitHub's
[private vulnerability reporting](https://github.com/lexandro2000/dosui/security/advisories/new)
or by email to **lexandro2000@gmail.com**.

Include as much detail as you can: affected version, reproduction steps, and the
potential impact. You can expect an acknowledgement within a few days. Once a
fix is available we will coordinate disclosure with you.

## Scope notes

dosui launches an external `dosbox` binary and reads/writes game profiles under
your XDG data directory. It does not run a network service. The most relevant
trust boundaries are imported artifacts (`dosbox.conf` files and zipped games)
and the configured DOSBox binary path — reports about those are especially
welcome.
