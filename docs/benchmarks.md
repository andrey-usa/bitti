# Benchmarks

Numbers: [bench run 45478008828](https://github.com/andrey-usa/bitti/actions/runs/45478008828), 2026-11-24.

Playwright's own TodoMVC suite (`demo-todo-app.spec.ts`, 24 tests) and its one-to-one bitti port, on the same local app and the same browser binary, headless, every run passing all 24 tests. Run on GitHub's `ubuntu-24.04`: AMD EPYC 7763 64-Core Processor, 4 cores, 15 GB, Linux-7.0.0-1017-azure-x86_64-with-glibc2.39.

Time is the whole command, the median of 5 runs taken in turns; CPU is user and system time of every process. Runner memory is bitti's own process, or Playwright's node or bun processes; browser memory is every browser process; each is its peak RSS, median of the runs.
PSS, in brackets, splits each shared page between the processes that share it: the memory a tool really takes. It comes from 2 more runs whose times are not used.

## Chrome for Testing 159.0.8154.6

| tool | workers | time, median (range) | CPU | runner memory | browser memory |
|---|---|---|---|---|---|
| **each tool's default workers** | | | | | |
| bitti 0.4.0 | 2 | **5.47 s** (5.41-5.53) | 13.0 s | 15 MiB (13 PSS) | 1,020 MiB (392 PSS) |
| Playwright Test 1.65.1, Node 24.22.3 | 2 | 10.85 s (10.81-10.90) | 27.1 s | 759 MiB (647 PSS) | 2,098 MiB (572 PSS) |
| Playwright Test 1.65.1, Bun 1.4.5 | 2 | 10.23 s (10.19-10.37) | 25.6 s | 368 MiB (267 PSS) | 2,095 MiB (564 PSS) |
| **one worker each** | | | | | |
| bitti 0.4.0 | 1 | **6.86 s** (6.79-6.88) | 11.5 s | 14 MiB (12 PSS) | 721 MiB (332 PSS) |
| Playwright Test 1.65.1, Node 24.22.3 | 1 | 12.51 s (12.35-14.52) | 23.3 s | 471 MiB (417 PSS) | 1,060 MiB (383 PSS) |
| Playwright Test 1.65.1, Bun 1.4.5 | 1 | 11.92 s (11.91-11.95) | 22.2 s | 251 MiB (198 PSS) | 1,065 MiB (381 PSS) |

## chrome-headless-shell 159.0.8154.6

| tool | workers | time, median (range) | CPU | runner memory | browser memory |
|---|---|---|---|---|---|
| **each tool's default workers** | | | | | |
| bitti 0.4.0 | 2 | **4.17 s** (4.14-4.32) | 10.0 s | 15 MiB (12 PSS) | 648 MiB (249 PSS) |
| Playwright Test 1.65.1, Node 24.22.3 | 2 | 6.95 s (6.93-7.73) | 16.7 s | 757 MiB (649 PSS) | 1,001 MiB (315 PSS) |
| Playwright Test 1.65.1, Bun 1.4.5 | 2 | 6.33 s (6.25-6.75) | 15.1 s | 375 MiB (272 PSS) | 1,000 MiB (312 PSS) |
| **one worker each** | | | | | |
| bitti 0.4.0 | 1 | **5.67 s** (5.64-5.70) | 8.9 s | 14 MiB (12 PSS) | 420 MiB (203 PSS) |
| Playwright Test 1.65.1, Node 24.22.3 | 1 | 8.67 s (8.62-8.68) | 13.5 s | 475 MiB (410 PSS) | 506 MiB (222 PSS) |
| Playwright Test 1.65.1, Bun 1.4.5 | 1 | 8.05 s (8.02-8.07) | 12.5 s | 256 MiB (204 PSS) | 505 MiB (218 PSS) |

## Every run

| browser | workers | tool | each run's time |
|---|---|---|---|
| Chrome for Testing 159.0.8154.6 | one worker each | bitti 0.4.0 | 6.79 s, 6.87 s, 6.86 s, 6.88 s, 6.85 s |
| Chrome for Testing 159.0.8154.6 | one worker each | Playwright Test 1.65.1, Node 24.22.3 | 12.63 s, 14.52 s, 12.51 s, 12.46 s, 12.35 s |
| Chrome for Testing 159.0.8154.6 | one worker each | Playwright Test 1.65.1, Bun 1.4.5 | 11.92 s, 11.92 s, 11.93 s, 11.91 s, 11.95 s |
| Chrome for Testing 159.0.8154.6 | each tool's default workers | bitti 0.4.0 | 5.48 s, 5.53 s, 5.41 s, 5.47 s, 5.45 s |
| Chrome for Testing 159.0.8154.6 | each tool's default workers | Playwright Test 1.65.1, Node 24.22.3 | 10.90 s, 10.81 s, 10.81 s, 10.85 s, 10.89 s |
| Chrome for Testing 159.0.8154.6 | each tool's default workers | Playwright Test 1.65.1, Bun 1.4.5 | 10.23 s, 10.37 s, 10.19 s, 10.23 s, 10.21 s |
| chrome-headless-shell 159.0.8154.6 | one worker each | bitti 0.4.0 | 5.64 s, 5.69 s, 5.67 s, 5.65 s, 5.70 s |
| chrome-headless-shell 159.0.8154.6 | one worker each | Playwright Test 1.65.1, Node 24.22.3 | 8.62 s, 8.67 s, 8.67 s, 8.66 s, 8.68 s |
| chrome-headless-shell 159.0.8154.6 | one worker each | Playwright Test 1.65.1, Bun 1.4.5 | 8.02 s, 8.07 s, 8.03 s, 8.05 s, 8.05 s |
| chrome-headless-shell 159.0.8154.6 | each tool's default workers | bitti 0.4.0 | 4.15 s, 4.14 s, 4.17 s, 4.32 s, 4.19 s |
| chrome-headless-shell 159.0.8154.6 | each tool's default workers | Playwright Test 1.65.1, Node 24.22.3 | 7.73 s, 6.95 s, 6.97 s, 6.93 s, 6.94 s |
| chrome-headless-shell 159.0.8154.6 | each tool's default workers | Playwright Test 1.65.1, Bun 1.4.5 | 6.28 s, 6.25 s, 6.75 s, 6.59 s, 6.33 s |

## Versions and method

bitti built from the run's commit (bitti 0.4.0); @playwright/test 1.65.1 from npm, its latest at the run; Node v24.22.3; Bun 1.4.5, Playwright's CLI run with `bun --bun`, so its workers are Bun too.

The app is tastejs/todomvc's React example at the last commit before it was replaced, with the test ids, the edit field's label and the toggle text demo.playwright.dev has, and a newer todomvc-app-css (`bench/todomvc/app.patch`). The spec is create-playwright's, checked against its SHA-256, with only its URL changed. Playwright runs create-playwright's template config, chromium only, trace off, the list reporter. The port has a scenario for every test and an `expect` for every assertion; where bitti has no list form or chained locator, the check is written out item by item (`bench/todomvc/bt/`).

Every tool starts from a warm-up round that is not counted; then the tools take turns, the order flipped each round. Each run is sampled from `/proc` every 20 ms (`bench/todomvc/memsample.py`), the whole process tree, with orphaned browser processes kept in it.

To run it yourself, with bitti built, Node, Bun, and a Chrome for Playwright and bitti:

```sh
python3 bench/todomvc/setup.py --work /tmp/todomvc
python3 -m http.server 8765 --bind 127.0.0.1 --directory /tmp/todomvc/www &
python3 bench/todomvc/bench.py --bitti target/release/bitti --work /tmp/todomvc \
  --browser chrome=/path/to/chrome --browser-dir /path/to --bun "$(command -v bun)" --json results.json
python3 bench/todomvc/gen_docs.py results.json --page bench.md
```
