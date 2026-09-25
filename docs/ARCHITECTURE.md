# Architecture

## Design objective

Keep ClearLane small around a very large dependency: Chromium.

Rust owns the product shell, user-facing browser behavior, ClearLane state, Shields integration, and later agent interfaces. CEF owns the web engine: Blink, V8, rendering, networking primitives, media, GPU integration, and browser subprocess behavior.

## Target ownership model

```text
ClearLane
├── app/        windowing + native browser chrome + composition
├── core/       browser state and product behavior
├── chromium/   CEF adapter and Chromium callbacks
└── shields/    adblock-rust integration and filter-list state
```

These are ownership areas first. They should become separate crates only when build/dependency boundaries justify it.

## Dependency direction

```text
app
  │ commands / view state
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

Key rule: **CEF types do not escape the `chromium` boundary.**

`core` should be testable without launching Chromium.

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

Use typed operations for real boundaries, for example:

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

Use Chromium Embedded Framework through the maintained Rust bindings in `tauri-apps/cef-rs` unless Slice 1 finds a concrete blocker.

The engine boundary exists because CEF is a vendor/runtime boundary, not because multiple engines are currently planned. Keep it narrow and grow it only when a feature requires another operation.

Do not design a Servo backend now. A future renderer experiment must adapt to the proven ClearLane core, not force abstractions into V1.

## UI host decision

ClearLane is Windows-first for initial implementation and the browser chrome must be Rust/native rather than an Electron/React/web-app shell.

The exact Rust UI toolkit is intentionally deferred to an early Slice 1 integration spike. The selected approach must prove all of the following with CEF before it becomes architectural policy:

- reliable Chromium child/off-screen composition
- keyboard focus and shortcuts
- text input and IME
- DPI scaling and resize behavior
- accessibility path
- low idle/startup overhead
- maintainable custom styling

Prefer the smallest proven solution. Do not select a framework because its demo screenshots look good.

## Shields

Use Brave's `adblock-rust`; do not write a new filtering engine.

Flow:

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

CEF provides request callbacks before a resource is loaded, which is the intended integration point for network filtering. Cosmetic filtering and scriptlet/resource behavior should be added only through capabilities supported safely by the chosen engine and CEF integration.

Keep the normal user-facing Shields policy small: enabled globally, per-site override, block count. More detailed filter-list controls can remain an advanced settings concern.

## Persistence

Prefer one SQLite database for ClearLane-owned durable state such as:

- history
- bookmarks
- downloads metadata
- ClearLane settings
- permission decisions when they are not already correctly owned by Chromium

Do not mirror Chromium-owned cache, cookies, local/session storage, or other site data without a concrete product requirement.

## Tab lifecycle

Tab resource state should eventually be explicit:

```text
Active → Background → Throttled → Discarded
```

Do not invent a second renderer lifecycle if Chromium already exposes the behavior needed. ClearLane owns policy and user expectations; Chromium owns renderer mechanics.

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
- TLS validation
- permission prompts
- safe external-protocol handling
- controlled file/download access

DevTools and future agent endpoints are privileged surfaces. They must be opt-in/local by default, bind narrowly, authenticate if exposed beyond a local trusted process, and never silently bypass normal site/user permissions.

## Dependency policy

Before adding a dependency, answer:

1. Is this problem already handled by CEF/Chromium or the standard library?
2. Is the crate actively maintained and appropriate for Windows?
3. Does it materially reduce our code/complexity?
4. What is its runtime/startup/binary cost?
5. Is its license compatible with the project?

Large frameworks require a stronger justification than focused libraries.

## Explicit anti-patterns

Do not add these preemptively:

- microservices or internal network APIs
- dependency-injection frameworks
- plugin architecture
- generic message/event infrastructure
- one crate per feature
- repository/service/controller layers around simple local state
- duplicate browser state that CEF already owns
- a second frontend stack for settings/new-tab unless genuinely required

## Upstream references

- CEF Rust bindings: https://github.com/tauri-apps/cef-rs
- Brave ad blocker: https://github.com/brave/adblock-rust
- Chromium Embedded Framework: https://bitbucket.org/chromiumembedded/cef/
