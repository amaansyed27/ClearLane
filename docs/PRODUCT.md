# Product and UX

## North star

ClearLane is a browser people should want to use even if they never use an AI agent.

It should feel direct, fast, calm, capable, and user-controlled. "Minimal" means low friction and low cognitive load, not deleting useful features or hiding familiar actions.

## Core product goals

1. **User first** — simple everyday browsing with a strong feature set.
2. **Native Shields** — ads and trackers blocked in the request path by default.
3. **Performance first** — startup, memory, idle work, tab lifecycle, and responsiveness are continuously measured.
4. **Agent-ready** — browser internals are structured so tools can later control the web without screenshot-heavy loops.
5. **Customizable** — strong defaults, with meaningful layout and workflow choices.

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

For each UI feature, study the interaction pattern before implementing it. Do not combine all reference features into one screen by default.

## Visual and interaction constraints

- The webpage is the main surface; browser chrome supports it.
- Use a clear information hierarchy and familiar platform behavior.
- Prefer direct controls over explanatory UI.
- Common actions stay obvious; uncommon actions may be progressively disclosed.
- Motion communicates state or spatial change; it is not decoration.
- Keyboard and accessibility behavior are first-class, not cleanup work.
- Avoid generic SaaS dashboards, card grids, excessive rounded containers, glassmorphism, ornamental gradients, AI sparkles, and decorative side panels.
- Do not introduce a distinctive aesthetic before the underlying interaction has been validated.

## Shields UX

Default state: protection on.

The normal per-site surface should answer only:

- Is protection on for this site?
- How many ads/trackers were blocked?
- Can I disable/enable it for this site?

Expert filter controls may exist later in settings, but should not dominate ordinary browsing.

## Agentic direction

Agentic capability is infrastructure before it is a consumer feature.

ClearLane should eventually expose structured browser state and deterministic actions so Codex, Claude Code, MCP clients, and Playwright-style tools can do web work with less screenshot/DOM repetition and fewer tokens.

The eventual agent layer should support ideas such as:

- CDP/Playwright compatibility
- Stable element references
- Structured accessibility/page snapshots
- Incremental page-state deltas
- Network/console access where explicitly permitted
- MCP/native control surfaces

There is **no Superagent UI in the initial browser roadmap**. It begins only after the human browser meets its quality bar.

## Non-goals for the initial browser

- AI chat sidebar
- News/feed new-tab page
- Rewards/crypto/VPN bundles
- Cloud account requirement
- Theme marketplace
- Novel interaction for novelty's sake
- Reimplementing Chromium web-platform features in Rust
