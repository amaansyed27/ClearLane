# ClearLane

A fast, lightweight, user-first Chromium browser written around a Rust-native core, with native ad/tracker blocking and an agent-ready foundation.

> **Current stage:** Stage 0 — project foundation. Architecture and product constraints are documented; implementation has not started.

## Goals

- Be an excellent everyday browser before becoming an AI product.
- Keep the common browsing path simple while retaining deep, optional features.
- Use Brave-style native ad/tracker blocking.
- Treat speed, memory use, startup time, and idle cost as product features.
- Later become a low-token browser runtime for Codex, Claude Code, MCP, and Playwright-style agents.

## Build plan

ClearLane is intentionally planned in only three implementation slices.

| Slice | Outcome | Status |
| --- | --- | --- |
| 1. Browser Alpha | Rust shell + Chromium, navigation, tabs/sidebar, Shields, persistence, CI and baseline performance harness | Next |
| 2. Everyday Browser | Full human browser: customization, Spaces/groups, split view, history, bookmarks, downloads, profiles/incognito, permissions, tab lifecycle and performance hardening | Planned |
| 3. Agent Runtime | CDP/Playwright compatibility, structured page state/deltas, MCP interface and token/latency benchmarks | Planned |

**Superagent is intentionally out of scope until the browser itself is good enough to use daily.**

## Engineering rules

- Simple modules before services or frameworks.
- Reuse proven libraries instead of rebuilding solved systems.
- Chromium/CEF types stay behind one integration boundary.
- No speculative abstractions, microservices, generic event buses, or unnecessary crates.
- No performance claims without reproducible comparisons against current Chrome, Brave, and Firefox.
- UI decisions come from explicit product requirements, usability evidence, and reference study — not generic SaaS/AI styling.

## Docs

- [Product and UX](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap and acceptance gates](docs/ROADMAP.md)
- [Performance discipline](docs/PERFORMANCE.md)
- [Agent instructions](AGENTS.md)

## License

ClearLane is licensed under the [Mozilla Public License 2.0](LICENSE).
