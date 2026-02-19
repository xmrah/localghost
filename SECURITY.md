# Security Policy

## Scope

LocalGhost is a **local-only** tool. It communicates exclusively with `localhost` (127.0.0.1) via the Ollama API. No data is transmitted to external servers.

## What LocalGhost Does NOT Do

- LocalGhost does **not** execute commands. It only prints them to stdout.
- LocalGhost does **not** store or log your queries externally. Local command history is stored in `~/.local/share/localghost/` and auto-expires after 7 days. You can clear it anytime with `localghost --clear-history`.
- LocalGhost does **not** make any network requests other than to your local Ollama instance.
- LocalGhost does **not** require root/sudo privileges to install or run.

## Built-in Safety

LocalGhost includes a regex-based dangerous command filter that flags:
- Recursive deletion (`rm -rf`)
- Disk formatting (`mkfs`, `dd`, `fdisk`)
- Fork bombs
- Remote code execution (`curl | bash`)
- Critical file destruction (`/etc/passwd`, `/boot/`)

**This filter is a safety net, not a guarantee.** AI models can hallucinate novel destructive commands that bypass regex patterns. **Always review commands before executing them.**

## Reporting Vulnerabilities

If you find a security issue (e.g., a bypass in the safety filter, or a way to make LocalGhost execute code), please:

1. **Do NOT open a public issue.**
2. Email: [open a private security advisory on GitHub](https://github.com/xmrah/localghost/security/advisories/new)
3. Include steps to reproduce.

I will respond within 48 hours.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x     | ✅ Current |
