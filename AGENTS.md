# ClearLane agent instructions

Keep this file short. Read only the project docs relevant to the task instead of loading every document for every change.

## Sources of truth

- Current stage and acceptance gates: `docs/ROADMAP.md`
- Slice 1.5 failure/reset details: `docs/SLICE_1_5.md`
- System boundaries and dependency rules: `docs/ARCHITECTURE.md`
- Features and UX constraints: `docs/PRODUCT.md`
- Performance work and claims: `docs/PERFORMANCE.md`

## Product invariants

1. ClearLane is a human browser first. Do not add an AI sidebar, assistant, chat surface, or Superagent unless the roadmap explicitly reaches that phase.
2. Default browsing must be simple; advanced capability should be discoverable without permanently crowding the chrome.
3. Brave-style ad/tracker blocking is a core browser feature, not an extension.
4. Performance, memory, idle CPU, startup, tab lifecycle and later agent token cost are measurable requirements.
5. Web compatibility and security must not be weakened to win a benchmark.
6. Visual quality, real-world blocking and competitive efficiency are manual/product gates; green CI alone is not product progress.

## Architecture rules

- Prefer a small Rust workspace with four ownership areas: `app`, `core`, `chromium`, `shields`.
- `core` must not depend on CEF/Chromium-specific types or a specific UI toolkit.
- Keep the CEF integration behind one narrow engine boundary.
- The application/UI runtime should own the product shell; CEF should be treated as the web surface/runtime rather than dictating the entire UI architecture.
- Use typed commands/events at real boundaries; do not add a generic event bus.
- Add a module before adding a crate. Add a crate only when an independent responsibility or build boundary justifies it.
- Do not create `Manager`, `Service`, `Repository`, `Controller`, `Factory`, or similar layers unless they solve an actual problem visible in the current code.
- Do not build plugin systems, dependency-injection frameworks, internal RPC, or microservices without an explicit requirement.
- Reuse CEF for browser-engine responsibilities and `adblock-rust` for filtering rather than recreating them.
- Keep persistence simple. Prefer one SQLite database for ClearLane-owned durable state; let Chromium own browser-engine state such as cache/cookies/site storage unless a feature requires otherwise.
- Avoid large files and god objects. Split by responsibility when a file becomes hard to reason about, not by arbitrary line counts.

## Frontend / UX rules

- Do not invent ClearLane's UI from generic design trends or from the assistant's personal taste.
- Study the specific interaction being implemented in strong browsers/products before changing it; use `docs/PRODUCT.md` as the constraint set.
- Arc is a reference for sidebar/Spaces/split-view ideas, Firefox for meaningful customization and user control, Safari for restrained browser chrome. Extract behavior; do not clone visual styling.
- The webpage should remain the visual focus.
- Do not use generic SaaS cards, excessive pills, glassmorphism, gradients, AI sparkles, decorative dashboards, or animation without functional purpose.
- Do not hide common browser controls merely to look minimal.
- Keep familiar browser interactions familiar unless a measured usability improvement justifies changing them.
- Browser chrome must be Rust/native; do not introduce Electron, React, or a web-rendered app shell.
- Raw stock/owner-drawn Win32 controls from the failed Slice 1 experiment are not an acceptable production UI direction.
- For Slice 1.5, build and visually validate the shell before coupling it deeply to Chromium. Do not make the user repeatedly rebuild a full browser merely to judge spacing, typography or layout.

## Research-first problem solving

For difficult integrations, framework choices, crashes, unexplained behavior, or repeated failures:

1. Search current upstream docs, maintained examples, release notes and issue trackers.
2. Check exact versions and known incompatibilities before patching around symptoms.
3. Reduce the failure to the smallest useful reproduction/spike.
4. State the hypothesis being tested and what result would falsify it.
5. If two materially similar attempts fail, stop repeating the same tactic. Reassess the architecture or compare an alternative.
6. Prefer a small proven change over accumulating compatibility hacks.
7. Keep useful experimental findings, but do not preserve a failed architecture because of sunk cost.

Do not brute-force a crash or integration problem for a long run without new evidence. If the root cause remains unknown, say so and change the investigation method.

## Slice 1.5 specific rules

- Start implementation from current `main`; treat PR #2 as an experiment/reference, not a branch to keep patching.
- Selectively salvage pure core logic, setup/bundle knowledge, persistence/tests, CEF embedding discoveries and other proven pieces.
- Research the native Rust UI options that best satisfy the actual Windows + CEF requirements before locking one. A previously suggested framework is a candidate, not a command.
- Shields must implement enough browser-process and renderer-side integration to pass real pages; a network blocked counter alone is not success.
- Restored background tabs must be lazy: persisted tab metadata must not automatically imply a live CEF browser for every tab.
- Benchmark scripts must distinguish internal regression measurements from fair competitor comparisons.

## Work method

- Work on the current roadmap slice; avoid scope drift into later slices.
- Prefer one substantial PR per roadmap slice rather than manufacturing many tiny slices. Internal proof gates are allowed and encouraged without turning them into separate roadmap slices.
- Each implementation PR must include: research/decision notes, what changed, architecture impact, automated checks, known limitations, and a copy-paste Windows manual verification guide.
- Never claim manual verification that was not actually performed by the user or an available Windows environment.
- Never describe a feature as fixed merely because CI passes; verify the actual failure mode that triggered the change.
- Update roadmap status only when its acceptance gate is actually met.

## Required checks once Rust code exists

Run the applicable checks before considering work ready for manual verification:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

CI should build the Windows target used by the project. Performance-sensitive changes also follow `docs/PERFORMANCE.md`.

## Security

- Keep Chromium/CEF sandboxing enabled unless an explicit, documented technical requirement proves otherwise.
- Treat external protocol execution, downloads, file access, permissions, JavaScript bridges, DevTools exposure, and future agent-control endpoints as trust boundaries.
- Do not disable site isolation or force unsafe single-process modes to improve memory numbers.
- Agent endpoints must not silently expand user permissions.
