# Performance discipline

"Fast" is a measured property, not a design adjective.

ClearLane may aim to beat Chrome, Brave, and Firefox on meaningful local workloads, but the project must not claim a win without a reproducible benchmark on the same machine and workload.

## What ClearLane can control

Chromium still owns Blink, V8, page rendering, GPU work, and much of the web runtime. ClearLane should win by keeping its own shell and policies lean:

- no Electron/Node runtime
- no React/web-app browser chrome
- no unnecessary background services
- native request blocking before unwanted resources load
- efficient tab lifecycle policy
- minimal telemetry/cloud/account work
- low-cost persistence and UI updates

Do not disable browser security features to improve a number.

## Baseline browsers

For release/performance work, compare against current stable builds of:

- Google Chrome
- Brave
- Firefox

Record exact versions and machine details with results.

## Human-browser metrics

At minimum measure:

- cold startup to usable window
- warm startup
- idle CPU after settling
- total process-tree memory/RSS
- memory with representative 1 / 10 / 30+ tab sets
- tab-switch responsiveness
- representative page-load time
- a browser benchmark such as current Speedometer when useful
- background-tab behavior and memory recovery after discard

Use the same URLs, profile state, extensions, cache conditions, and measurement procedure across browsers as far as practical.

## Method

- Automate measurement where possible.
- Prefer multiple runs and report the median rather than a single lucky run.
- Keep raw results or machine-readable summaries under an ignored results directory; commit only scripts and useful aggregate baselines.
- Compare against the previous ClearLane baseline as well as competitors.
- A >5% ClearLane regression in an established metric should be investigated or explicitly justified before merge.

## Agent metrics — Slice 3

Agent efficiency is measured separately from normal browsing speed:

- bytes/tokens returned to the agent
- number of full snapshots
- number of screenshots
- tool calls per task
- action latency
- end-to-end task time
- success/retry rate

The structured snapshot/delta design is successful only if it reduces real task cost or increases reliability compared with normal Chromium/Playwright workflows.

## Benchmark hygiene

Do not publish comparisons that mix:

- different machines
- materially different browser versions
- different page sets
- enabled extensions in only one browser
- warm cache vs cold cache without labeling it
- debug ClearLane builds vs optimized competitor builds

Every performance PR should state what was measured, how, and whether the result is statistically meaningful or merely directional.
