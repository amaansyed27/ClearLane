# Product and UX

## North star

ClearLane is a browser people should want to use even if they never use an AI agent.

It should feel direct, fast, calm, capable, and user-controlled. "Minimal" means low friction and low cognitive load, not deleting useful features or hiding familiar actions.

## Core product goals

1. **User first** — simple everyday browsing with a strong feature set.
2. **Native Shields** — ads and trackers blocked in the browser path by default.
3. **Performance first** — startup, memory, idle work, tab lifecycle, and responsiveness are continuously measured.
4. **Agent-ready** — browser internals are structured so tools can later control the web without screenshot-heavy loops.
5. **Customizable** — strong defaults, with meaningful layout and workflow choices.

## Quality bar learned from Slice 1

The first runnable alpha experiment failed the manual gate. That failure establishes explicit product rules:

- A functional shell that looks like a developer tool is not an acceptable browser UI.
- Raw/default/owner-drawn Win32 controls are not the ClearLane visual language merely because they are native and lightweight.
- CI success does not compensate for poor interaction or visual quality.
- A Shields block counter does not compensate for visible ads still playing.
- A memory script does not establish a competitive performance claim without comparable competitor measurements.
- Do not ask the user to repeatedly rebuild a full Chromium integration just to review elementary layout/spacing changes. Prototype and approve the browser chrome first.

## Human-browser feature set

### Always-important

- Omnibox/search
- Back, forward, reload/stop
- Tabs
- Optional sidebar
- Vertical or horizontal tab layout
- Pinned tabs
- Shields status and per-site control
- Downloads
- History
- Bookmarks
- Profiles and private browsing
- Site permissions

### Power without permanent clutter

- Spaces/tab groups
- Split view
- Command palette / keyboard actions
- Tab sleeping/discarding
- Restore/session continuity
- Configurable sidebar and key browser controls

Advanced controls should appear when relevant or through settings/commands, not as a wall of permanent toolbar icons.

## Reference study

ClearLane may learn from existing browsers without copying their appearance:

- **Arc:** sidebar organization, Spaces, split browsing, command-driven access to features.
- **Firefox:** user control, vertical/horizontal tab flexibility, configurable browser chrome.
- **Safari:** restrained browser chrome and clear visual priority for the page itself.

For each UI feature, study the interaction pattern before implementing it. Also inspect current production browser screenshots, platform conventions, accessibility expectations, and maintained component/framework examples relevant to the chosen implementation stack.

Do not combine all reference features into one screen by default.

## Visual and interaction constraints

- The webpage is the main surface; browser chrome supports it.
- Use a clear information hierarchy and familiar platform behavior.
- Prefer direct controls over explanatory UI.
- Common actions stay obvious; uncommon actions may be progressively disclosed.
- Motion communicates state or spatial change; it is not decoration.
- Keyboard and accessibility behavior are first-class, not cleanup work.
- Avoid generic SaaS dashboards, card grids, excessive rounded containers, glassmorphism, ornamental gradients, AI sparkles, and decorative side panels.
- Do not introduce a distinctive aesthetic before the underlying interaction has been validated.
- Do not use the assistant's taste as evidence. Use references, product requirements, and user review.

## Slice 1.5 shell gate

Before Chromium is deeply integrated into the replacement shell, the user should be able to judge a lightweight prototype containing representative fake/local state for:

- sidebar and tabs
- active/inactive/hover states
- omnibox
- navigation controls
- Shields indicator
- resizing
- typography and spacing

The implementation should iterate on this shell until the user considers it credible enough to become the browser foundation.

This is not a request for visual novelty. It is a request to avoid locking poor browser chrome behind expensive engine integration.

## Shields UX

Default state: protection on.

The normal per-site surface should answer only:

- Is protection on for this site?
- How many ads/trackers were blocked?
- Can I disable/enable it for this site?

Expert filter controls may exist later in settings, but should not dominate ordinary browsing.

For correctness, the internal implementation must be judged by representative real pages, not merely the number shown in the UI.

## Performance UX

Efficiency should be felt as well as benchmarked:

- startup should not feel delayed by browser-owned setup work that can be cached/prepared,
- restoring many tabs should not eagerly wake every renderer,
- inactive tabs should not compete needlessly with the active page,
- tab reactivation should remain predictable,
- ClearLane should never trade away browser security for a lower Task Manager number.

## Agentic direction

Agentic capability is infrastructure before it is a consumer feature.

ClearLane should eventually expose structured browser state and deterministic actions so Codex, Claude Code, MCP clients, and Playwright-style tools can do web work with less screenshot/DOM repetition and fewer tokens.

The eventual agent layer should support ideas such as:

- CDP/Playwright compatibility
- stable element references
- structured accessibility/page snapshots
- incremental page-state deltas
- network/console access where explicitly permitted
- MCP/native control surfaces

There is **no Superagent UI in the initial browser roadmap**. It begins only after the human browser meets its quality bar.

## Non-goals for the initial browser

- AI chat sidebar
- news/feed new-tab page
- rewards/crypto/VPN bundles
- cloud account requirement
- theme marketplace
- novel interaction for novelty's sake
- reimplementing Chromium web-platform features in Rust
