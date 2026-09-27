# Architecture

## Design objective

Keep ClearLane small around a very large dependency: Chromium.

Rust owns the product shell, user-facing browser behavior, ClearLane state, Shields integration, and later agent interfaces. CEF owns the web engine: Blink, V8, rendering, networking primitives, media, GPU integration, and browser subprocess behavior.

The failed Slice 1 experiment adds one important rule: **CEF is a dependency of the browser product, not the architecture of the whole application.** The UI/runtime should not collapse into whichever host controls are easiest to bolt onto CEF.

## Target ownership model

```text
ClearLane
├── app/        native browser chrome + product composition
├── core/       browser state and product behavior
├── chromium/   CEF adapter and Chromium callbacks
└── shields/    adblock-rust engine + browser/renderer integration
```

These are ownership areas first. They should become separate crates only when build/dependency boundaries justify it.

## Dependency direction

```text
app
  │ typed commands / view state
  ▼
core ◄──────── shields policy/state
  │
  │ narrow engine interface
  ▼
chromium
  │
  ▼
CEF / Chromium
```

Key rules:

- CEF types do not escape the `chromium` boundary.
- UI toolkit types do not leak into `core`.
- `core` should be testable without launching Chromium or a native window.
- Human UI and later agent clients should call the same browser capabilities where practical.

## Core model

Start with the smallest useful domain model:

```text
Browser
├── windows
├── tabs
├── navigation
├── downloads
├── history/bookmarks
├── profiles/permissions
└── settings
```

Use typed operations at real boundaries, for example:

```text
OpenTab
CloseTab
ActivateTab
Navigate
Back
Forward
Reload
Stop
```

and typed browser events such as URL/title/loading changes. Do not introduce a generic string event bus.

## Chromium boundary

Use Chromium Embedded Framework through maintained Rust bindings unless current Slice 1.5 research finds a concrete blocker or a clearly better supported embedding route.

The engine boundary exists because CEF is a vendor/runtime boundary, not because multiple engines are currently planned. Keep it narrow and grow it only when a feature requires another operation.

Do not design a Servo backend now. A future renderer experiment must adapt to the proven ClearLane core, not force abstractions into V1.

## UI host decision — reset

The first Slice 1 attempt used raw/owner-drawn Win32 `BUTTON`, `EDIT`, `LISTBOX`, GDI painting and hard-coded layout. That was useful as an embedding experiment but failed the product-quality gate. **Do not continue that shell as the production UI.**

Slice 1.5 must research current native-Rust UI options and maintained Windows/CEF embedding patterns before committing. Candidate frameworks or approaches should be evaluated against the actual requirements rather than selected from demo aesthetics.

The chosen approach must prove:

- credible custom browser chrome without raw stock-control appearance
- reliable CEF child/off-screen composition
- ownership of the app/UI event loop that does not force CEF-specific code throughout the shell
- keyboard focus and shortcuts
- text input and IME
- DPI scaling and resize behavior
- accessibility path
- overlay/popover behavior around a native web surface
- low idle/startup overhead
- maintainable custom styling

A GPU/native Rust UI framework such as GPUI may be investigated because it is relevant to these requirements, but it is a candidate, not an architectural mandate. Compare it against at least one viable alternative or a simpler native composition approach before locking the dependency.

### Shell-first proof

Before full Chromium integration, implement enough fake/local state to exercise:

- sidebar/tabs
- omnibox
- navigation controls
- Shields indicator
- resizing and focus
- visual states

The user should visually approve this shell before expensive CEF integration continues. This is an internal proof gate inside Slice 1.5, not a new roadmap slice.

## CEF integration strategy

Once the shell is accepted, embed CEF as a web surface behind the `chromium` boundary.

Research current CEF recommendations for:

- host-owned Windows message loops vs CEF-owned loops
- child-window vs off-screen/windowless rendering tradeoffs
- subprocess executable separation
- sandbox-compatible packaging
- focus/IME/DPI handling

Prefer the route with the smallest amount of glue and the strongest upstream support. Do not force the application into a weak UI architecture solely because it makes one CEF callback easier.

## Shields — complete integration, not just the engine

Use Brave's `adblock-rust`; do not write a new filtering engine.

Network filtering remains:

```text
CEF request
   ↓
request interception
   ↓
ClearLane Shields
   ↓
adblock-rust
   ├── allow → Chromium continues
   └── block → request cancelled
```

But Slice 1 proved that network cancellation alone is not Brave-style Shields. Slice 1.5 must research the current `adblock-rust`/Brave integration and implement enough renderer-side behavior to make the engine effective on real pages.

Expected responsibilities include, where supported by the current upstream API:

- correct resource-type mapping
- filter-list lifecycle
- serialized/precompiled engine state for fast full-strength startup when appropriate
- initial cosmetic resources
- dynamic class/id cosmetic updates
- scriptlet/resource injection
- procedural cosmetic actions needed by representative pages
- safe browser↔renderer communication

Keep the normal user-facing Shields policy small: enabled globally, per-site override, block count. Internally, however, a block counter is not a correctness test.

## Persistence

Prefer one SQLite database for ClearLane-owned durable state such as:

- history
- bookmarks
- downloads metadata
- ClearLane settings
- permission decisions when they are not already correctly owned by Chromium

Do not mirror Chromium-owned cache, cookies, local/session storage, or other site data without a concrete product requirement.

## Tab lifecycle — required for Slice 1.5

A persisted tab is not automatically a live Chromium browser.

Use an explicit model such as:

```text
Tab metadata
   │
   ├── Live       active/recent browser surface exists
   ├── Background live but deprioritized where justified
   └── Discarded  metadata/session retained, no live browser surface
```

On restore, create the active browser surface first. Background restored tabs should remain metadata-only until activated unless a concrete reason requires otherwise.

Do not invent unsafe renderer tricks when Chromium already provides lifecycle primitives. ClearLane owns policy and user expectations; Chromium owns renderer mechanics.

Protect active media/download/critical state from naive discard policies.

## Subprocess footprint

CEF is intentionally multiprocess. Do not use `--single-process`, disable site isolation, or disable sandboxing to chase Task Manager numbers.

Research whether a minimal dedicated CEF subprocess executable is useful for ClearLane's packaging/runtime model. If adopted, keep application-only UI/state code out of subprocess paths where practical. Any claimed benefit must be measured rather than assumed.

## Agent seam

Do not build the agent runtime during the human-browser slices. Preserve the ability to expose browser operations later through:

- Chromium DevTools Protocol compatibility
- Playwright connection
- structured page snapshots
- incremental page deltas
- MCP/native APIs

Human UI and agent clients should ultimately call the same browser capabilities instead of maintaining two unrelated automation stacks.

## Security boundaries

Never trade these away for performance:

- Chromium/CEF sandboxing
- origin/security model
- site isolation
- TLS validation
- permission prompts
- safe external-protocol handling
- controlled file/download access

DevTools and future agent endpoints are privileged surfaces. They must be opt-in/local by default, bind narrowly, authenticate if exposed beyond a local trusted process, and never silently bypass normal site/user permissions.

## Dependency policy

Before adding a dependency, answer:

1. Is this problem already handled by CEF/Chromium or the standard library?
2. Is the crate actively maintained and appropriate for current Windows/Rust versions?
3. Does it materially reduce our code/complexity?
4. What are its runtime/startup/binary costs?
5. Is its license compatible with the project?
6. Are there maintained examples of the exact integration pattern we need?
7. What is the fallback if this dependency becomes the blocker?

Large frameworks require a stronger justification than focused libraries.

## Failure/recovery protocol

When a difficult integration repeatedly fails, do not accumulate patches indefinitely.

```text
failure
  ↓
reproduce minimally
  ↓
check upstream docs/issues/examples + exact versions
  ↓
test a specific hypothesis
  ↓
root cause known?
  ├── yes → fix smallest cause
  └── no  → compare alternative / change approach
```

After two materially similar failed attempts, continuing the same tactic requires new evidence. Sunk cost is not evidence.

## Explicit anti-patterns

Do not add these preemptively:

- microservices or internal network APIs
- dependency-injection frameworks
- plugin architecture
- generic message/event infrastructure
- one crate per feature
- repository/service/controller layers around simple local state
- duplicate browser state that CEF already owns
- a second web frontend stack for settings/new-tab unless genuinely required
- raw Win32 owner-draw browser chrome as the production design
- performance/security hacks that make ClearLane look good only in a benchmark

## Upstream references

These references must be re-checked for current APIs during implementation rather than treated as frozen documentation:

- CEF Rust bindings: https://github.com/tauri-apps/cef-rs
- Brave ad blocker: https://github.com/brave/adblock-rust
- Chromium Embedded Framework: https://bitbucket.org/chromiumembedded/cef/
- Brave browser integration source: https://github.com/brave/brave-core
