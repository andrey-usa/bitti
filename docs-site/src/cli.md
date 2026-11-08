# CLI usage

Verbatim `--help` output of the released build, **bitti 0.1.0**:

```console
$ bitti --version
bitti 0.1.0
```

## `bitti --help`

```text
bitti: plain-text web tests on real Chrome (through navigera)

usage: bitti <command> [...]

commands:
  run [<file|dir>...]    run scenarios (default: every .bt and .feature file
                         under the current directory)
  lint [<file|dir>...]   check files without a browser
  list [<file|dir>...]   the scenarios a run would take
  merge <dir|file>...    join the jsonl results of shards into one report
  trace [<dir>] [<line>] a trace a run recorded (--trace): every step, or one
  fix [<file>:<line>]    put a suggestion from the last run into a failing step
  record <url>           a browser to use by hand; what you do becomes a
                         scenario
  from-session <name>    what a navigera session did, as a scenario
  navigera <command…>    navigera's commands, built in
                         (bitti navigera -s work goto <url>)
  mcp                    an MCP server: navigera's browser tools and
                         bitti's own
  skill                  the agent skill: the .bt format, steps and expect forms
  --version

`bitti <command> --help` lists a command's options. Steps, locators and
expect forms: `bitti skill`.

exit codes: 0 all passed, 1 a scenario failed, 2 a file or an argument is
wrong, 3 the browser or the app server didn't start
```

## `bitti run --help`

```text
usage: bitti run [<file.bt|file.feature|dir>...] [options]

Runs the scenarios in the files, or in every .bt and .feature file under
the directories (default: the current one).

options:
  --base-url <url>       where relative URLs (`goto /cart`) point (else `base`
                         in the file, BITTI_BASE_URL)
  --var <name=value>     a variable for ${name} (repeatable)
  --secrets <file>       NAME=value lines for ${secret:NAME} (the environment
                         comes first); masked in output
  --grep <text>          only scenarios whose name contains it
  --grep-invert <text>   leave out scenarios whose name contains it
  --tag <expr>           only scenarios with this @tag, or a tag expression:
                         "smoke and not slow", "(a or b) and not wip"
                         (repeatable: any of them)
  --last-failed          only the scenarios that failed in the last run (also
                         --lf)
  --max-failures <n>     start no scenario once n have failed (-x: 1); the rest
                         are reported as not run
  --forbid-only          a scenario tagged @only is an error (on by itself under
                         CI); without it, @only runs just those
  --workers <n|auto>     scenarios at once (default 1; auto: half the CPUs, at
                         most 8); also -j <n>
  --isolation <how>      context (default): one browser, a browser context per
                         worker; browser: a browser each
  --retries <n>          run a failing scenario again, up to n times; one that
                         then passes is flaky
  --fail-on-flaky        a flaky scenario fails the run
  --shard <i/n>          run the i-th of n slices (for CI matrix jobs)
  --repeat <n>           run every scenario n times (flake hunting: each run is
                         reported, then a pass count)
  --shuffle [<seed>]     run in a random order (the seed is printed; give it
                         again to repeat the order)
  --reporter <name>      console (default), jsonl (one JSON object per scenario
                         on stdout), github (workflow annotations; on by itself
                         under GitHub Actions); repeatable
  --config <file>        settings, a [server] and [env.<name>] profiles
                         (default: ./bitti.toml)
  --env <name>           the config's [env.<name>] profile on top of the rest
  --update-snapshots     a screenshot or JSON snapshot that differs from its
                         baseline (or has none) becomes it (also
                         --update-screenshots)
  --compare-report <k>   what `expect download … compare` keeps: html, json,
                         rows (comma-separated) or none (default html)
  --trace <when>         on-failure or all: each step's page (ax tree, JPEG),
                         console and requests beside the screenshot in
                         trace.jsonl; read it with `bitti trace`, or in --html
  --video <when>         on-failure or all: what the page showed, frame by
                         frame, in the scenario's evidence (video/index.html
                         plays it; linked from --html)
  --junit <file>         also write JUnit XML
  --cucumber <file>      also write Cucumber JSON for the .feature files (step
                         by step)
  --allure <dir>         also write Allure results (a JSON file per scenario
                         run, screenshots attached)
  --html <file>          also write the run as one HTML page (steps, times,
                         failures with their screenshot)
  --json                 print what the scenarios saved, as one JSON object, at
                         the end
  --timeout-ms <ms>      how long an expect retries (default 5000)
  --scenario-timeout-ms <ms> the most one scenario may take (its steps; hooks
                         around it aside); @slow triples it
  --global-timeout-ms <ms> the most the whole run may take: no scenario starts
                         after it, running ones stop there, and the run fails
  --wait-ms <ms>         how long a step waits for its element (default 5000)
  --headed               show the browser
  --slowmo <ms>          wait this long before each step that acts on the page
                         (to watch a --headed run)
  --chromium <path>      the browser to run
  --engine lightpanda    run on Lightpanda (no layout: a fast smoke tier;
                         screenshots and pixel checks fail)
  --electron <app>       test an Electron app: its directory or main script
                         (Electron from its node_modules) or an executable; a
                         fresh app per scenario, one at a time
  --electron-arg <arg>   an argument for the app (repeatable;
                         --electron-arg=<arg> for one starting with -)
  --attach <port|ws://…> run against a browser or an Electron app already
                         running with a DevTools port (one worker; an app
                         attached to goes on from one scenario to the next)
  --evidence <dir>       failure screenshots and downloads (default
                         .bitti/runs/<time>; a directory you name is used as
                         given, links included)
  --watch                after the run, run again each scenario file that
                         changes (polled every 300 ms), until Ctrl-C
  --pause-on-failure     keep the browser at a failure, open to `navigera
                         --attach`, until Enter (one worker)
```

## `bitti lint --help`

```text
usage: bitti lint [<file|dir>...] [--var <name=value>] [--secrets <file>]

Checks .bt and .feature files without a browser: syntax, unknown steps,
and variables nothing sets (warnings: the run's environment may set them).
```

## `bitti list --help`

```text
usage: bitti list [<file|dir>...] [--grep <text>] [--tag <tag>]
                      [--shard <i/n>]

The scenarios a run with the same options would take, one per line:
file:line, name and tags.
```

## `bitti merge --help`

```text
usage: bitti merge <dir|file.jsonl>... [--junit <file>] [--html <file>]
                       [--fail-on-flaky]

Joins the results of shards (`run --shard i/n --reporter jsonl`, saved
to a file each) into one console summary, one --junit and one --html, as
if one run had done it. A directory is searched for .jsonl files. Steps
and traces stay with the shards: the joined report has each scenario's
status, time and failure. The time is the longest shard's, as shards
run side by side. Exit: 0 all passed, 1 one failed, 2 nothing to read.
```

## `bitti trace --help`

```text
usage: bitti trace [<dir>] [<line>]

A trace that `run --trace on-failure|all` recorded: every step with its
page, console and requests, or the step at <line> in full, with its
screenshot file and ax tree. <dir> is a run's directory
(.bitti/runs/<time>) or one scenario's in it; by default, the
latest run's.
```

## `bitti fix --help`

```text
usage: bitti fix [<file>:<line>] [--use <n>] [--all [--min <p>]]

Puts a suggestion from the last run into the step that failed. A run keeps,
for each step whose element wasn't found, the elements on the page most like
it (.bitti/last-run.json).

  bitti fix                 list each failing step and its suggestions
  bitti fix shop.bt:12      put suggestion 1 into that step
  --use <n>                     put suggestion n instead
  --all                         fix every failing step with its first
                                suggestion, if it scores --min or more
  --min <p>                     the least score --all takes (0 to 1; 0.6)
```

## `bitti record --help`

```text
usage: bitti record <url> [--out <file.bt>] [--scenario <name>] [--headless]
                        [--for <seconds>]
       bitti record --extend <file.bt> [--scenario <name>]
                        [--base-url <url>] [--secrets <file>]

Opens a browser to use by hand; what you do becomes a scenario. With
--extend, runs the scenario to its end first, then records more steps
into it. Needs a display.
```

## `bitti from-session --help`

```text
usage: bitti from-session <navigera session> [--scenario <name>]
                              [--out <file.bt>]

Writes what a navigera session did as a scenario, its refs turned into
locators that hold between runs.
```

## `bitti mcp --help`

```text
usage: bitti mcp [--headed] [--chromium <path>]

An MCP server on standard input and output: navigera's browser tools and
bitti_from_session, bitti_run, bitti_failure and bitti_fix.
```

## `bitti skill --help`

```text
usage: bitti skill

Prints the agent skill: the .bt format, steps and expect forms.
```
