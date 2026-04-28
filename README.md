# Helios-Terminal

> An agentic terminal powered by [helios-agi](https://github.com/helios-agi) — born from Warp's open-source codebase.

---

## What is Helios-Terminal?

Helios-Terminal is a fork of [warpdotdev/warp](https://github.com/warpdotdev/warp) that replaces Warp's built-in AI layer with **Pi** — the helios-agi runtime. Everything that makes Warp an exceptional terminal (block-based UI, inline editing, completions, voice input) is preserved. The AI brain is swapped: Pi is the sole agent, running locally with your own API keys.

### How it works

```
┌─────────────────────────────────┐
│       Helios-Terminal (Rust)    │
│  Warp terminal shell + UI      │
│  ─────────────────────────────  │
│       helios_bridge crate       │
│  (subprocess / PTY to Pi)       │
└───────────────┬─────────────────┘
                │
┌───────────────▼─────────────────┐
│       Pi Runtime (helios-agi)   │
│  Helios orchestrator · skills   │
│  providers · tools · governance │
└─────────────────────────────────┘
```

- **Warp** = the terminal shell (kept as-is)
- **Pi** = the brain (spawned as a subprocess)
- **helios_bridge** = the plumbing connecting them

## Getting Started

### Prerequisites

1. **Pi runtime** — Install from [helios-agi](https://github.com/helios-agi)
2. **Provider keys** — Configure at least one provider in `~/.pi/providers.json`
3. **Rust toolchain** — For building from source

### Building from source

```bash
./script/bootstrap   # platform-specific setup
./script/run         # build and run Helios-Terminal
./script/presubmit   # fmt, clippy, and tests
```

See [WARP.md](WARP.md) for the full engineering guide, including coding style, testing, and platform-specific notes.

## Project Structure

| Crate | Purpose |
|-------|---------|
| `crates/helios_bridge` | Bridge between terminal and Pi runtime (subprocess management, type mapping) |
| `crates/helios_auth` | Hybrid auth: Pi local providers + helios-agi cloud OAuth (optional) |
| `crates/warpui` / `warpui_core` | Warp's UI framework (MIT licensed) |
| `crates/warp_terminal` | Terminal emulator |
| `crates/editor` | Inline code editing |
| `app` | Main application |

## Licensing

- **Warp's UI framework** (`warpui_core`, `warpui`) — [MIT License](LICENSE-MIT)
- **Warp codebase** — [AGPL v3](LICENSE-AGPL)
- **Helios additions** (`helios_bridge`, `helios_auth`) — MIT License

## Acknowledgements

Helios-Terminal is built on the incredible work of the [Warp](https://www.warp.dev) team. We're grateful they open-sourced their terminal — it's the best shell foundation we've ever seen.

This project also depends on many excellent open-source libraries. See [WARP.md](WARP.md) and the Cargo dependency tree for the full list.

## Contributing

We welcome contributions! Please follow the same workflow described in [CONTRIBUTING.md](CONTRIBUTING.md).

## Code of Conduct

We follow the [Code of Conduct](CODE_OF_CONDUCT.md) inherited from the Warp project.
