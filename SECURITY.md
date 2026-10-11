# Security Policy

## Scope

This application runs **elevated** and writes to:

`HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\...`

It changes (or removes) the `Attributes` value for power settings so they appear or disappear in **Advanced power settings**. It does **not** edit AC/DC power values.

## Reporting a vulnerability

Please open a [GitHub Security Advisory](https://docs.github.com/en/code-security/security-advisories) or a private report against this repository. Do not file public issues for exploitable privilege-escalation or remote-code paths.

Include:

- Windows version and build
- Steps to reproduce
- Expected vs actual behavior
- Whether a non-admin process can trigger the issue

## Safe use

- Prefer a restore point or registry export before bulk changes.
- Close and reopen the Windows Advanced power settings dialog after changing visibility.
- Only download release binaries from this repository’s Releases page.
