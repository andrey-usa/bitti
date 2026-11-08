# CLI usage

> **Name:** this project was renamed **bitti → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `bitti`
> and its messages still say `bitti`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

Verbatim usage text printed by the current build:

```text
bitti: plain-text web tests on real Chrome (through navigera)

usage:
  bitti run [<file.bt|dir>...] [options]   run scenarios (default: the current directory)
  bitti lint [<file.bt|dir>...]             check files without a browser
  bitti list [<file.bt|dir>...]             the scenarios a run would take
  bitti skill                               the agent skill (the format and commands, for an AI agent)
  bitti --version

run options:
  --base-url <url>      where relative URLs (`goto /cart`) point (else `base` in the file, BITTI_BASE_URL)
  --var <name=value>    a variable for ${name} (repeatable)
  --secrets <file>      NAME=value lines for ${secret:NAME} (the environment comes first); masked in output
  --grep <text>         only scenarios whose name contains it
  --tag <tag>           only scenarios with this @tag (repeatable: any of them)
  --workers <n|auto>    scenarios at once (default 1; auto: half the CPUs, at most 8)
  --isolation <how>     context (default): one browser, a browser context per worker; browser: a browser each
  --retries <n>         run a failing scenario again, up to n times; one that then passes is flaky
  --fail-on-flaky       a flaky scenario fails the run
  --shard <i/n>         run the i-th of n slices (for CI matrix jobs)
  --repeat <n>          run every scenario n times (flake hunting: each run is reported, then a pass count)
  --shuffle [<seed>]    run in a random order (the seed is printed; give it again to repeat the order)
  --reporter <name>     console (default), jsonl (one JSON object per scenario on stdout), github
                        (workflow annotations; on by itself under GitHub Actions); repeatable
  --junit <file>        also write JUnit XML
  --json                print what the scenarios saved, as one JSON object, at the end
  --timeout-ms <ms>     how long an expect retries (default 5000)
  --wait-ms <ms>        how long a step waits for its element (navigera's default 5000)
  --headed              show the browser
  --chromium <path>     the browser to run
  --evidence <dir>      failure screenshots (default .bitti/runs/<time>)
  --pause-on-failure    keep the browser at a failure, open to `navigera --attach`, until Enter (one worker)

exit codes: 0 all passed, 1 a scenario failed, 2 a file or an argument is wrong, 3 the browser didn't start
```
