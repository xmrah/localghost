# Contributing to Ghost

Thanks for your interest! Ghost is a small, focused project. Here's how to contribute.

## Philosophy

- **Zero external dependencies.** Ghost uses only Python 3 standard library. Do not add `pip` packages.
- **Safety first.** Every change must preserve or improve the safety filter.
- **Distro-agnostic.** Never hardcode a single distribution's tools as the default.

## How to Contribute

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/your-idea`)
3. Make your changes
4. Test on your local system with Ollama running
5. Submit a Pull Request

## What We Need

- **Safety filter patterns:** Know a destructive command we don't catch? Add a regex.
- **Distro support:** Use a distro not in the `pkg_map`? Add it.
- **Prompt engineering:** Found a prompt that works better across models? Share it.
- **Bug reports:** If Ghost suggests a wrong command, open an issue with:
  - Your distro (`cat /etc/os-release`)
  - Model used (`ghost --models`)
  - Query and output

## Code Style

- Python: Follow PEP 8. Use f-strings. Type hints welcome but not required.
- Shell: Use `shellcheck` on `install.sh` before submitting.
- Comments: English.
