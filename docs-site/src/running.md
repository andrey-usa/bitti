# Running

> **Name:** this project was renamed **bitti → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `bitti`
> and its messages still say `bitti`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

## Running

```text
bitti run [<file.bt|dir>...] [--base-url <url>] [--var k=v] [--grep <text>] [--tag <tag>]
              [--workers <n|auto>] [--isolation context|browser] [--retries <n>] [--fail-on-flaky] [--shard <i/n>]
              [--repeat <n>] [--shuffle [<seed>]]
              [--reporter console|jsonl|github] [--junit <file>] [--json]
              [--timeout-ms <ms>] [--wait-ms <ms>] [--headed] [--chromium <path>]
              [--secrets <file>] [--pause-on-failure]
bitti lint [<file.bt|dir>...]
bitti list [<file.bt|dir>...]
```

- Every scenario starts in a fresh browser context (navigera `reset
  --all`: no cookies, storage, tabs, routes, init scripts, `set`
  emulation or dialog answer from the scenario before), in about 40 ms
  instead of a new browser's few hundred. `--workers n` runs n at once,
  each in a browser context of one shared browser (about 40% less memory
  at 4 workers than a browser each; `--isolation browser` gives each its
  own); the longest (from `.bitti/timings.json`) start first.
- `--retries n` runs a failing scenario again; one that passes then is
  reported FLAKY with how its first try failed. `--fail-on-flaky` fails
  the run on one. `--shard i/n` takes every n-th scenario, for CI matrix
  jobs.
- `--repeat 20 --shuffle` hunts flakes and order dependence: every
  scenario runs 20 times (`name #k`), in a random order whose seed is
  printed (`--shuffle <seed>` runs that order again), then a pass count per
  scenario, FLAKY where it isn't all of them.
- `--reporter jsonl` prints one JSON object per scenario as it finishes
  (the console moves to stderr); `github` writes workflow annotations on
  the failing lines (on by itself under GitHub Actions). `--json` prints
  what the scenarios saved, for a pipeline's next step.
- A failure prints the step, what was expected and seen, where the page
  was, a screenshot (under `.bitti/runs/`), and a note when an earlier
  click changed nothing on the page (navigera's `no_effect`), the usual
  reason the next check fails. When the element isn't there, it lists the
  closest ones on the page as locators to paste in:

  ```text
  FAIL  shop.bt:21  Accept the cookies  (611 ms)
        line 23: click --role button --name "Accept everything"
        click role="button" name="Accept everything": … no element …
        nearest on the page:
          1. button "Accept all"   --role button --name "Accept all"   0.63
  ```
- `--pause-on-failure` stops at a failure with the browser as the failure
  left it and prints how to look at that very tab (`navigera -s bt start
  --attach ws://…/devtools/page/…`, then `navigera -s bt ax`); Enter goes
  on with the run. It runs one worker. With no terminal to press Enter in,
  it waits `PRIMAVERA_PAUSE_S` seconds (default 600, for an agent looking
  from elsewhere), and under CI (`CI` set) it doesn't pause unless that is
  set.
- Exit codes: 0 passed, 1 a scenario failed, 2 a file or argument is wrong,
  3 the browser didn't start.
