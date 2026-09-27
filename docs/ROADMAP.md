# Roadmap

## Current stage

**Slice 1.5 — Browser Alpha Reset**

- [x] Product direction recorded
- [x] Minimal system boundaries recorded
- [x] Performance discipline recorded
- [x] Agent instructions recorded
- [x] Slice 1 produced a runnable Windows experiment
- [x] Slice 1 manual gate failed on UI quality, Shields effectiveness, and memory efficiency
- [ ] Slice 1.5 corrective reset implemented
- [ ] Slice 1.5 manual Windows gate passed

ClearLane still uses a small number of substantial implementation slices. **Slice 1.5 is a corrective reset, not permission to fragment the project into many mini-slices.**

---

## Slice 1 — Browser Alpha first attempt

### Result

PR #2 produced a working CEF/Chromium browser experiment, but it failed the user-facing manual gate and must not be merged or treated as architecture to preserve blindly.

### What was learned

- CEF/Alloy can be embedded and bundled on Windows.
- The pure Rust browser state/core and some setup, persistence, CI, and Chromium-boundary work are salvageable concepts.
- Raw/owner-drawn Win32 controls did not meet the browser-chrome quality bar.
- Using `adblock-rust` for network checks is not equivalent to implementing Brave-grade Shields; renderer-side cosmetic/scriptlet/procedural behavior matters.
- Restored background tabs must not eagerly create full CEF browser instances if ClearLane intends to be lightweight.
- A ClearLane-only working-set script is an internal regression harness, not proof of superiority over Chrome/Brave/Firefox.
- Green CI is necessary but says nothing about visual quality, real-world ad blocking, or competitive efficiency.

PR #2 is therefore an **experiment/reference branch**, not the base for incremental patching.

---

## Slice 1.5 — Browser Alpha Reset

### Outcome

A Windows ClearLane alpha that proves the three product claims the first attempt failed to prove:

1. the browser chrome is visually and ergonomically credible,
2. Shields blocks representative real-world ads/trackers through a complete enough integration,
3. the architecture is measurably efficient in realistic browser workloads.

The implementation should start from `main` and selectively salvage useful work from PR #2 rather than inheriting its architecture wholesale.

### Required internal gates

These are gates inside one slice, not separate roadmap slices.

#### Gate A — Shell quality

Build the browser chrome independently of Chromium first using the best researched native-Rust UI approach for this project.

Prove:

- coherent layout, spacing, typography and states
- sidebar/tabs/omnibox/navigation/Shields chrome
- resize and DPI behavior
- keyboard focus and text input/IME path
- accessibility path
- no developer-tool/raw-control appearance

The user should visually approve the shell before expensive browser integration continues.

#### Gate B — Chromium integration

Embed CEF/Chromium inside the approved shell without letting CEF dictate the entire app/UI architecture.

Prove:

- one ClearLane window
- stable child/off-screen composition as chosen by research
- focus, resizing, navigation and tab switching
- sandbox/site-isolation/security remain intact
- app UI runtime remains maintainable and independent from CEF-specific types

#### Gate C — Shields

Use `adblock-rust` as the filtering engine but implement the browser/renderer integration needed for real behavior rather than equating network cancellation with Brave parity.

Research current Brave/adblock-rust integration and prove:

- correct resource typing and network filtering
- filter lists are ready before normal first navigation, preferably from serialized/precompiled state where supported
- cosmetic filtering for initial and dynamically changing DOM
- required scriptlet/resource handling
- per-site enable/disable
- representative ad-heavy pages and several monetized YouTube videos are manually tested

A blocked counter is not proof of success; visible ads/trackers are the manual gate.

#### Gate D — efficiency

Implement real lazy/restored-tab behavior and a fair comparison harness.

Prove:

- saved background tabs do not all instantiate live CEF browsers on startup
- tab lifecycle is explicit enough to support live/background/discarded states
- active media tabs are not discarded incorrectly
- competitor measurements use comparable Chrome, Brave and Firefox workloads on the same machine
- memory metrics are labeled correctly; do not use summed working set as if it were unique/private memory
- no security feature is disabled for benchmark wins

### Research-first rule

Before selecting or replacing a difficult framework/integration, search current upstream documentation, maintained examples, issue trackers, release notes, and comparable production projects. Record the reason for the chosen approach in the PR.

If an implementation becomes stuck or repeatedly crashes:

1. stop patching symptoms,
2. reduce it to the smallest reproducible case,
3. inspect upstream docs/examples/issues and version compatibility,
4. verify assumptions with a tiny spike,
5. compare at least one viable alternative when the current approach remains uncertain,
6. then either fix the root cause or deliberately change approach.

Do not spend long runs brute-forcing the same architecture because it already has sunk cost.

### Acceptance gate

The user can clone the Slice 1.5 branch on Windows and verify:

- the browser chrome looks and behaves like a credible consumer browser,
- normal modern sites render in one ClearLane window,
- navigation and multiple tabs work reliably,
- Shields passes agreed real-world ad/tracker checks including monetized YouTube samples,
- restart/session behavior is sane,
- lazy/restored tabs do not eagerly create all browser surfaces,
- measured resource results are accompanied by fair Chrome/Brave/Firefox comparisons,
- no known crash is being hidden behind retries or a green CI badge.

The PR must include a short copy-paste Windows verification guide. Slice 1.5 is complete only after the user passes this gate.

---

## Slice 2 — Everyday Browser

### Outcome

ClearLane becomes a browser worth using daily without any agent features.

### Scope

- polished sidebar plus vertical/horizontal tab layouts
- pinned tabs
- Spaces/tab groups
- split view
- history UI
- bookmarks UI
- download management
- profiles
- private browsing
- site permissions
- robust session restore
- mature tab throttling/discarding policy
- command palette / keyboard workflow
- meaningful browser-chrome customization
- settings kept compact and task-oriented
- Shields polish and per-site controls
- accessibility/IME/DPI polish
- performance hardening against Chrome, Brave and Firefox
- packaging/update path sufficient for regular use

### Acceptance gate

The user can run ClearLane as their normal browser for representative browsing tasks without repeatedly falling back to another browser for basic browser functionality.

No Superagent work starts before this gate is met.

---

## Slice 3 — Agent Runtime

### Outcome

ClearLane becomes a preferred browser runtime for coding/automation agents while remaining the same human browser.

### Scope

- stable opt-in automation mode
- CDP compatibility needed by existing tooling
- Playwright connection path
- structured page/accessibility snapshots
- stable element references for deterministic actions
- incremental page-state deltas where practical
- network/console inspection with explicit permission boundaries
- MCP/native browser control interface
- agent-session isolation and lifecycle
- benchmark harness comparing task latency, payload size/token use, screenshots, and reliability against stock Chromium/Playwright workflows

### Acceptance gate

Representative Codex/Claude Code/Playwright-style tasks complete reliably, and ClearLane demonstrates measured reductions in automation payload/token cost or task overhead without weakening browser security or human UX.

---

## After the roadmap — Superagent

Only after the browser roadmap is healthy do we design the ClearLane Superagent.

It should not default to the Atlas/Comet pattern of attaching a generic chat sidebar to a browser. Its product model will be designed separately from the browser foundation.

## Status update rule

A slice moves to complete only when:

1. implementation is merged,
2. required automated checks pass,
3. security-sensitive changes are reviewed,
4. the user completes the manual Windows verification gate,
5. known failures are documented rather than hidden.
