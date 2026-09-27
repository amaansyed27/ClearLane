# Slice 1.5 — Browser Alpha Reset

This document exists because the first Slice 1 implementation reached a runnable Windows browser but failed the actual product gate. It should prevent future work from repeating the same implementation loop.

## User remarks that triggered the reset

The manual Windows tests reported three failures repeatedly:

1. **UI quality:** the browser chrome looked like a skinned developer tool/raw Win32 interface rather than a credible consumer browser.
2. **Shields:** monetized YouTube ads still played, even when the blocker counter increased.
3. **Efficiency:** a single YouTube workload still showed a high Chromium/CEF process-tree footprint, missing ClearLane's lightweight goal.

The user also objected to a process that kept producing green CI/rebuild cycles without proving the product claims that mattered.

These are not deferred polish items. They are Slice 1.5 design inputs.

## What went wrong technically

### UI

The experiment used owner-drawn Win32 controls (`BUTTON`, `EDIT`, `LISTBOX`) plus GDI painting and manual layout. This was a reasonable integration spike but not a viable path to the intended browser chrome.

**Lesson:** do not confuse native controls with good product UI. Validate the shell independently before coupling it to Chromium.

### CEF ownership

The easiest embedding path influenced too much of the application architecture and encouraged the shell to become a thin layer around CEF/Win32 mechanics.

**Lesson:** ClearLane owns the product shell/runtime. CEF is the web engine surface behind a narrow adapter.

### Shields

The implementation initially treated all requests too generically and later improved network typing and some cosmetic/scriptlet behavior, but it still did not implement the full browser/renderer workflow needed by representative dynamic pages.

**Lesson:** `adblock-rust` is the filtering engine, not a complete Brave integration. Network cancellation, cosmetic filtering, dynamic DOM updates, scriptlets/resources and procedural behavior must be treated as one system and validated against real pages.

### Startup blocker lifecycle

The experiment started with a fallback blocker, compiled the full engine after launch and reloaded the active page.

**Lesson:** prefer full-strength blocker state ready before normal first navigation, using upstream-supported serialization/precompilation when appropriate.

### Tabs and memory

Restored URLs were looped through `open_tab`, which created a CEF browser for each restored tab. That is not genuinely lazy restoration.

**Lesson:** tab metadata and live browser surfaces are different things. Restore metadata first and instantiate only what is needed.

### Benchmarks

The first performance script measured ClearLane itself and split Chromium process roles, which is useful for regression investigation, but it did not run equivalent Chrome/Brave/Firefox workloads. Summed working set was also easy to overinterpret because shared memory can be counted across processes.

**Lesson:** maintain separate internal-regression and competitive benchmark harnesses. Never convert an internal number into a marketing claim.

## What is worth salvaging from PR #2

Inspect, test and selectively transplant rather than cherry-picking everything.

Likely salvageable concepts/code include:

- small pure Rust `core` browser state
- omnibox normalization and tests
- persistence concepts/schema where sound
- CEF setup/bundling knowledge
- the Alloy embedding discovery that eliminated the extra Chrome-style top-level window
- sandbox-compatible build/packaging knowledge
- CI setup and useful core tests
- error-page/crash handling concepts
- filter-list fetching/update work where it matches the new Shields design

Likely replacement areas:

- raw Win32 browser chrome
- app/event-loop ownership that exists primarily to accommodate CEF
- partial renderer-side Shields integration
- eager restored-tab browser creation
- benchmark positioning that implies competitor results without running competitors

## Slice 1.5 implementation strategy

This remains one implementation slice, with proof gates inside it.

### Gate A — research + shell prototype

Before committing to the UI framework:

- search current native Rust UI/framework options,
- inspect maintained Windows support and accessibility/IME/DPI behavior,
- inspect CEF/native-child embedding examples and open issues,
- compare the leading candidate against at least one viable alternative,
- prefer maintained production evidence over popularity or screenshots.

A framework such as GPUI is worth evaluating because it is GPU/native and relevant to browser-style chrome, but it is not pre-approved merely because it was suggested previously.

Build the shell with fake/local tabs and state first. Ask for visual/manual review before coupling the replacement UI deeply to Chromium.

### Gate B — CEF integration

Research current CEF guidance rather than assuming the previous event-loop model is optimal.

Prove:

- one app window,
- correct browser-surface parenting/composition,
- reliable focus/IME/DPI/resize,
- no unsafe sandbox/site-isolation compromises,
- clean command/event boundary between product core and CEF.

Investigate host-owned message-loop options and dedicated subprocess-executable patterns if they materially simplify the architecture or footprint. Measure rather than assume their benefit.

### Gate C — Shields proof

Study current upstream `adblock-rust`, Brave integration code, docs, release notes and issues before implementing.

The proof must include:

- network filtering with correct request context/types,
- complete-enough cosmetic path for initial and dynamic DOM changes,
- scriptlet/resource handling required by chosen lists,
- procedural behavior required by representative pages,
- per-site policy,
- filter state ready before normal navigation where practical,
- real manual tests on ad-heavy sites and multiple monetized YouTube samples.

A rising blocked counter is diagnostic data, not a pass condition.

### Gate D — lifecycle + competitive measurement

Implement truly lazy restored tabs:

```text
persisted tab metadata
      ↓
restore tab model
      ↓
active tab creates browser surface
      ↓
background tab creates one only when policy requires it
```

Then build/extend a fair comparison procedure for ClearLane, Chrome, Brave and Firefox on the same Windows machine.

## Smart debugging rule

The implementation agent should not equate persistence with intelligence.

When stuck on a crash or integration bug:

1. reproduce it with the smallest useful case,
2. inspect logs/crash output and identify the failing boundary,
3. search exact error/version/framework combinations in upstream docs/issues/source,
4. read maintained examples using the same API/version,
5. write down the current hypothesis,
6. run one targeted experiment that can prove or disprove it,
7. if two similar attempts fail without new evidence, stop and compare another approach.

Do not keep changing flags/imports/callbacks randomly until CI turns green. Do not hide uncertainty behind long implementation runs.

## Manual gate philosophy

The user should not be the first person asked to discover an obvious architectural problem.

Use automated tests/CI for correctness that automation can prove. Use targeted local/prototype evidence for visual and integration decisions. Ask the user to manually validate the things only their real Windows environment or product judgment can validate.

For Slice 1.5 the three decisive manual questions are:

1. **Does this finally look like a browser worth continuing?**
2. **Do the representative ads actually disappear?**
3. **Do the resource measurements and tab behavior materially support the lightweight direction?**

If any answer is no, do not mark the slice complete and do not rationalize the result with CI output.
