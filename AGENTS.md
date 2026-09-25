# ClearLane agent instructions

Keep this file short. Read only the project docs relevant to the task instead of loading every document for every change.

## Sources of truth

- Current stage and acceptance gates: `docs/ROADMAP.md`
- System boundaries and dependency rules: `docs/ARCHITECTURE.md`
- Features and UX constraints: `docs/PRODUCT.md`
- Performance work and claims: `docs/PERFORMANCE.md`

## Product invariants

1. ClearLane is a human browser first. Do not add an AI sidebar, assistant, chat surface, or Superagent unless the roadmap explicitly reaches that phase.
2. Default browsing must be simple; advanced capability should be discoverable without permanently crowding the chrome.
3. Brave-style ad/tracker blocking is a core browser feature, not an extension.
4. Performance, memory, idle CPU, startup, and agent token cost are measurable requirements.
5. Web compatibility and security must not be weakened to win a benchmark.

## Architecture rules

- Prefer a small Rust workspace with four ownership areas: `app`, `core`, `chromium`, `shields`.
- `core` must not depend on CEF/Chromium-specific types.
- Keep the CEF integration behind one narrow engine boundary.
- Use typed commands/events at real boundaries; do not add a generic event bus.
- Add a module before adding a crate. Add a crate only when an independent responsibility or build boundary justifies it.
- Do not create `Manager`, `Service`, `Repository`, `Controller`, `Factory`, or similar layers unless they solve an actual problem visible in the current code.
- Do not build plugin systems, dependency-injection frameworks, internal RPC, or microservices without an explicit requirement.
- Reuse CEF for browser-engine responsibilities and `adblock-rust` for filtering rather than recreating them.
- Keep persistence simple. Prefer one SQLite database for ClearLane-owned durable state; let Chromium own browser-engine state such as cache/cookies/site storage unless a feature requires otherwise.
- Avoid large files and god objects. Split by responsibility when a file becomes hard to reason about, not by arbitrary line counts.

## Frontend / UX rules

- Do not invent ClearLane's UI from generic design trends.
- Study the specific interaction being implemented in strong browsers/products before changing it; use `docs/PRODUCT.md` as the constraint set.
- Arc is a reference for sidebar/Spaces/split-view ideas, Firefox for meaningful customization and user control, Safari for restrained browser chrome. Extract behavior; do not clone visual styling.
- The webpage should remain the visual focus.
- Do not use generic SaaS cards, excessive pills, glassmorphism, gradients, AI sparkles, decorative dashboards, or animation without functional purpose.
- Do not hide common browser controls merely to look minimal.
- Keep familiar browser interactions familiar unless a measured usability improvement justifies changing them.
- Browser chrome must be Rust/native; do not introduce Electron, React, or a web-rendered app shell.
- The concrete Rust UI toolkit is intentionally not locked until Slice 1 proves Windows + CEF composition, DPI, IME, keyboard focus, accessibility, resizing, and overhead.

## Work method

- Work on the current roadmap slice; avoid scope drift into later slices.
- Prefer one substantial PR per roadmap slice rather than manufacturing many tiny slices. Use additional PRs only when risk or an external blocker genuinely requires it.
- Each implementation PR must include: what changed, architecture impact, automated checks, known limitations, and a copy-paste Windows manual verification guide.
- Never claim manual verification that was not actually performed by the user or an available Windows environment.
- Update roadmap status only when its acceptance gate is actually met.

## Required checks once Rust code exists

Run the applicable checks before considering work complete:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

CI should build the Windows target used by the project. Performance-sensitive changes also follow `docs/PERFORMANCE.md`.

## Security

- Keep Chromium/CEF sandboxing enabled unless an explicit, documented technical requirement proves otherwise.
- Treat external protocol execution, downloads, file access, permissions, JavaScript bridges, DevTools exposure, and future agent-control endpoints as trust boundaries.
- Agent endpoints must not silently expand user permissions.
