# Security Policy

## Scope

Ghost CLI is a **local-only** tool. It communicates exclusively with `localhost` (127.0.0.1) via the Ollama API. No data is transmitted to external servers.

## What Ghost Does NOT Do

- Ghost does **not** execute commands. It only prints them to stdout.
- Ghost does **not** store or log your queries.
- Ghost does **not** make any network requests other than to your local Ollama instance.
- Ghost does **not** require root/sudo privileges to install or run.

## Built-in Safety

Ghost includes a regex-based dangerous command filter that flags:
- Recursive deletion (`rm -rf`)
- Disk formatting (`mkfs`, `dd`, `fdisk`)
- Fork bombs
- Remote code execution (`curl | bash`)
- Critical file destruction (`/etc/passwd`, `/boot/`)

**This filter is a safety net, not a guarantee.** AI models can hallucinate novel destructive commands that bypass regex patterns. **Always review commands before executing them.**

## Reporting Vulnerabilities

If you find a security issue (e.g., a bypass in the safety filter, or a way to make Ghost execute code), please:

1. **Do NOT open a public issue.**
2. Email: [open a private security advisory on GitHub](https://github.com/xmrah/ghost/security/advisories/new)
3. Include steps to reproduce.

I will respond within 48 hours.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x     | ✅ Current |
