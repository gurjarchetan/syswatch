# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.4.x   | ✅ Yes     |
| 0.3.x   | ⚠️ Critical fixes only |
| < 0.3   | ❌ No      |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Report security issues privately by emailing **chetan04.2014@gmail.com** with the subject line `[SysWatch Security]`.

Include:
- A description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- The SysWatch version and OS affected
- Any suggested fix (optional but appreciated)

You will receive an acknowledgement within **72 hours**. We aim to release a patch within **14 days** of a confirmed vulnerability.

## Scope

SysWatch is a read-only system monitor. It:
- **Does not** accept network connections
- **Does not** run as root (no SUID/SGID bits)
- **Does read** `/proc` and system files as the invoking user
- **Does fetch** the public IP from `api.ipify.org` once at startup over plain HTTP (no secrets transmitted)

If you discover a way to escalate privileges, leak sensitive data, or crash the system through SysWatch, that qualifies as a security issue.
