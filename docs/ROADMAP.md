# Roadmap

## Current stage

**Slice 1 — Browser Alpha: implementation ready, manual Windows verification pending**

- [x] Product direction recorded
- [x] Minimal system boundaries recorded
- [x] Performance discipline recorded
- [x] Agent instructions recorded
- [x] Slice 1 implementation started
- [x] Slice 1 implementation ready for manual Windows verification
- [ ] Slice 1 manual Windows verification complete
- [ ] Slice 1 implementation PR merged

The project deliberately uses only **three implementation slices**. Do not split them into many artificial milestones just to make agent work look smaller.

---

## Slice 1 — Browser Alpha

### Outcome

A real Windows ClearLane build that can replace a basic browsing session.

### Scope

- Rust workspace and Windows build path
- Proven native-Rust UI host + CEF integration
- Browser window and Chromium rendering
- Omnibox/search and URL normalization
- Back / forward / reload / stop
- Multiple tabs
- Basic collapsible sidebar/tab surface
- Native Shields request blocking using `adblock-rust`
- Minimal persistence needed for restart/session sanity
- Basic error/crash surfaces
- Windows CI: format, clippy, tests, build
- Repeatable performance-baseline script/harness

### Acceptance gate

The user can clone on Windows, run a short setup flow, launch ClearLane, browse normal modern sites, open/close/switch tabs, navigate reliably, observe ads/trackers being blocked, restart without obvious state corruption, and compare baseline startup/memory numbers with installed browsers.

The implementation PR must include a **copy-paste manual verification guide**. Do not mark Slice 1 complete from CI alone.

---

## Slice 2 — Everyday Browser

### Outcome

ClearLane becomes a browser worth using daily without any agent features.

### Scope

- Polished sidebar plus vertical/horizontal tab layouts
- Pinned tabs
- Spaces/tab groups
- Split view
- History UI
- Bookmarks UI
- Download management
- Profiles
- Private browsing
- Site permissions
- Session restore
- Tab throttling/discarding policy
- Command palette / keyboard workflow
- Meaningful browser-chrome customization
- Settings kept compact and task-oriented
- Shields polish and per-site controls
- Accessibility/IME/DPI polish
- Performance profiling and hardening against Chrome, Brave, Firefox
- Packaging/update path sufficient for regular use

### Acceptance gate

The user can run ClearLane as their normal browser for representative browsing tasks without repeatedly falling back to another browser for basic browser functionality. The manual test covers navigation, media-heavy sites, downloads, permissions, private browsing, restart/restore, Shields, tab lifecycle, and chosen customization paths.

No Superagent work starts before this gate is met.

---

## Slice 3 — Agent Runtime

### Outcome

ClearLane becomes a preferred browser runtime for coding/automation agents while remaining the same human browser.

### Scope

- Stable opt-in automation mode
- CDP compatibility needed by existing tooling
- Playwright connection path
- Structured page/accessibility snapshots
- Stable element references for deterministic actions
- Incremental page-state deltas where practical
- Network/console inspection with explicit permission boundaries
- MCP/native browser control interface
- Agent-session isolation and lifecycle
- Benchmark harness comparing task latency, payload size/token use, screenshots, and reliability against stock Chromium/Playwright workflows

### Acceptance gate

Representative Codex/Claude Code/Playwright-style tasks complete reliably, and ClearLane demonstrates measured reductions in automation payload/token cost or task overhead without weakening browser security or human UX.

---

## After the roadmap — Superagent

Only after all three slices are healthy do we design the ClearLane Superagent.

It should not default to the Atlas/Comet pattern of attaching a generic chat sidebar to a browser. Its product model will be designed separately from the browser foundation.

## Status update rule

A slice moves to complete only when:

1. implementation is merged,
2. required automated checks pass,
3. security-sensitive changes are reviewed,
4. the user completes the manual Windows verification gate,
5. known failures are documented rather than hidden.
