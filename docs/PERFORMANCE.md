# Performance discipline

"Fast" is a measured property, not a design adjective.

ClearLane may aim to beat Chrome, Brave, and Firefox on meaningful local workloads, but the project must not claim a win without a reproducible benchmark on the same machine and workload.

## What ClearLane can control

Chromium still owns Blink, V8, page rendering, GPU work, and much of the web runtime. ClearLane should win by keeping its own shell and policies lean:

- no Electron/Node runtime
- no React/web-app browser chrome
- no unnecessary background services
- native request blocking before unwanted resources load
- efficient and genuinely lazy tab lifecycle policy
- minimal telemetry/cloud/account work
- low-cost persistence and UI updates

Do not disable browser security features to improve a number.

## Two different benchmark purposes

Do not mix these.

### 1. Internal regression harness

Used to answer:

> Did this ClearLane change make ClearLane better or worse?

It may measure a ClearLane process tree only and can be lightweight enough for frequent development.

### 2. Competitive browser harness

Used to answer:

> How does ClearLane compare with Chrome, Brave, and Firefox under equivalent conditions?

This requires equivalent workloads, browser versions, profile/cache conditions, settle times, multiple runs, and correctly labeled memory metrics.

A ClearLane-only script is **not** evidence for a competitive claim.

## Baseline browsers

For competitive work, compare against current stable builds of:

- Google Chrome
- Brave
- Firefox

Record exact versions and machine details with results.

## Human-browser metrics

At minimum measure:

- cold-ish startup to usable window, clearly labeled if OS caches are not purged
- warm startup
- idle CPU after settling
- process count by role where useful
- working set and a memory metric suitable for cross-process comparison such as private/unique committed memory where available
- representative 1 / 10 / 30+ tab sets
- memory after background tabs settle or are discarded
- tab-switch responsiveness
- representative page-load time
- a browser benchmark such as current Speedometer when useful
- background-tab behavior and memory recovery after discard

Do not sum process working sets and describe the result as unique application memory without qualification; shared pages may be counted more than once.

## Representative workloads

Slice 1.5 should include at least:

- one blank/new tab state
- one ordinary content page
- one heavy/media page such as YouTube
- a 10-tab mixed set
- a 30-tab mixed set
- restored-session behavior where only the active tab should become live initially

Use the same URLs and comparable browser states where practical.

## Method

- Automate measurement where possible.
- Prefer multiple runs and report the median rather than a single lucky run.
- Keep raw results or machine-readable summaries under an ignored results directory; commit scripts and useful aggregate baselines only.
- Compare against the previous ClearLane baseline as well as competitors.
- A >5% ClearLane regression in an established metric should be investigated or explicitly justified before merge.
- Distinguish measurement noise/direction from statistically meaningful claims.

## Performance investigations

If memory or CPU is unexpectedly high:

1. attribute cost by process role and workload,
2. verify whether it belongs to ClearLane shell/state or Chromium renderer/GPU/utility work,
3. check whether background tabs were actually instantiated/live,
4. inspect upstream Chromium/CEF behavior before adding switches,
5. prefer lifecycle/policy improvements over unsafe process-model hacks.

Do not use `--single-process`, disable sandboxing/site isolation, or disable important security features to make a graph look better.

## Slice 1.5 expected opportunity

Do not promise dramatic savings for one active heavy Chromium page. Blink, V8, GPU compositing and media decoding still exist.

ClearLane's credible performance opportunities are primarily:

- lower shell/application overhead,
- less unnecessary browser-service work,
- native blocking that avoids unwanted resource work,
- lazy session restore,
- stronger background/discard policy,
- fewer unnecessary live browser surfaces,
- later, lower agent automation payload/token cost.

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
- different measurement definitions presented as the same metric

Every performance PR should state what was measured, how, and whether the result is an internal regression measurement or a fair competitive comparison.
