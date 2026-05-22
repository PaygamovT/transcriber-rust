# AGENTS.md

> Project map for AI agents. Keep this file up-to-date as the project evolves.

## Project Overview
A lightweight, zero-footprint desktop utility called **Transcriber** rewritten from Python (PyQt6/pynput) to pure Rust. It sits silently in the system tray, records microphone audio entirely in memory on hotkey trigger, transcribes/translates using OpenRouter/OpenAI/Groq APIs, and types the result at the active cursor or copies it to the clipboard.

## Tech Stack
- **Language:** Rust
- **GUI Framework:** `egui` + `eframe` (ultra-lightweight settings window)
- **Tray & Menu:** `tray-icon`
- **Global Hotkey:** `global-hotkey`
- **Audio Capture:** `cpal` + `hound`
- **HTTP Client:** `ureq` (synchronous, lightweight HTTP)
- **Output Handlers:** `enigo` (typewriter emulation) + `arboard` (clipboard)
- **Database:** None (configurations stored in `~/.transcriber/config.json`)

## Project Structure
```
.
├── .agent/                  # Custom workflows, prompt instructions, and skills configurations
├── .agents/                 # Installed custom and external developer skills
│   └── skills/              # Installed skills directory
│       ├── audio-transcriber # Reference audio capturing and Whisper APIs patterns
│       ├── egui-tray-application # Custom egui + tray-icon desktop lifecycle skill
│       ├── rust-best-practices # Apollo GraphQL Rust clean coding standards
│       └── subagent-driven-development # Multi-agent task execution and review orchestration
├── .ai-factory/             # AI Factory configuration and specs
│   ├── ARCHITECTURE.md      # Project architecture guidelines (to be generated)
│   └── DESCRIPTION.md       # High-level product and technical specification
├── .ai-factory.json         # AI Factory initialization manifest
├── .mcp.json                # Project-level MCP configuration (e.g. filesystem)
├── ai_prompt.md             # Original user prompt and technical specification
└── AGENTS.md                # This file - structural map for AI agents
```

## Key Entry Points
The project structure skeleton and persistence config layer are fully initialized:
- `Cargo.toml`: Package configuration optimized for minimal binary footprint.
- `src/main.rs`: Core bootstrap loop configuring debug log routines.
- `src/config.rs`: Platforms-agnostic configuration structures.

## Documentation
| Document | Path | Description |
|----------|------|-------------|
| README | [README.md](README.md) | Project landing page |
| Getting Started | [docs/getting-started.md](docs/getting-started.md) | Installation, setup, first steps |
| Architecture | [docs/architecture.md](docs/architecture.md) | Project structure and patterns |
| Configuration | [docs/configuration.md](docs/configuration.md) | Environment variables, config files |

## AI Context Files
| File | Purpose |
|----------|-------------|
| AGENTS.md | This file — project structure map |
| .ai-factory/DESCRIPTION.md | Project specification and tech stack |
| .ai-factory/ARCHITECTURE.md | Architecture decisions and guidelines |

