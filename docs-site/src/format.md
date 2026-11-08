# The .pv format

> **Name:** this project was renamed **primavera → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `primavera`
> and its messages still say `primavera`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

## The format

- Top-level lines: `base <url>`, `set NAME = value`, `use other.pv`,
  `flow name(a, b)`, `auth name [ttl 30m]`, `before all|each`, `after
  each|all`, `scenario "name" @tag…`. Steps are indented under them.
- A step is a navigera command line, written exactly as you would type it
  after `navigera -s work …`: `goto`, `click`, `fill`, `select`, `press`,
  `check`, `hover`, `scroll`, `upload`, `wait`, `eval`, `set`, `route`,
  `state load`, and the rest. Targets use navigera's locators (`--role`
  with `--name`, `--label`, `--text`, `--placeholder`, `--testid`,
  `--selector`, `--nth`). Refs from `ax` are refused: they change between
  runs.
- Words split like a shell's, with no expansion. `${NAME}` and
  `${NAME:-default}` come from `save`, a flow's arguments, the data row,
  `before all`, `--var`, `set`, then the environment; `$$` (or `\$` in
  double quotes) is a literal `$`. Also `${run.id}` (one per run: unique
  emails), `${scenario.name}`, and `${download.path}`/`${download.name}`
  after `expect download`.
- `${secret:NAME}`: from the environment, else `--secrets <file>`
  (`NAME=value` lines). Its value is masked as `***` in everything the run
  prints and saves (messages, the page's URL and title, saved values,
  JUnit and JSON lines).
- A default with spaces is one word: quote it whole,
  `"${GREETING:-hello there}"` (words split before `${…}` is filled in).
- `save NAME = <command>` keeps a command's result (`get text …`).
- `flow name(a, b)` at the top level holds steps; `run name x y` runs them
  with `${a}` and `${b}` set. `use other.pv` takes another file's flows
  and `set`s. A failure inside a flow names the flow's own line.
- `optional <step>`: a step that may not apply (a cookie banner that only
  sometimes shows); its failure is noted, not fatal.
- `if <expect clause>` with steps indented under it runs them when the
  clause holds now (or within its `--timeout-ms`): `if --selector
  "#banner" visible`, then `click --text Accept` indented below.
- `auth shopper [ttl 30m]` holds steps that sign in; `as shopper` in a
  scenario starts it signed in. The block runs once per run, in a browser
  context of its own, the first time a scenario needs it (the other
  workers wait for it), and its cookies and storage are saved under
  `.primavera/auth/` (keep that out of git); with a `ttl` the saved state
  is reused by later runs until it is that old.
- `before all` runs once for the file, before its first scenario; what it
  saves every scenario of the file sees. If it fails, none of them run.
  `after all` runs once after the last one (a failure there is reported as
  `(after all)`).
- `handoff until <expect clause>`: the steps only a person can do (2FA, a
  CAPTCHA). When the clause already holds, nothing happens; else the
  browser comes on screen, the run waits for the clause (5 minutes, or its
  `--timeout-ms`) or Enter, and the browser goes back off screen. Plain
  `handoff` waits for Enter. With no screen it fails at once and says so.
- Data rows: an indented `examples` line with a table under it, or
  `scenario "…" for each logins.csv`, makes one scenario per row, named
  from the row (`scenario "Ship to ${country}"`); the row's columns are
  variables.
- `expect` retries until it holds or its timeout (5 s, `--timeout-ms` on the
  line or the run) runs out:

| expect | |
|---|---|
| `url`/`title` `is`, `contains`, `starts-with`, `ends-with`, `matches` `<value>` | the active tab |
| `text "<t>"`, `no text "<t>"` [`--within <target>`] | visible text, frames included, or inside an element |
| `<target> visible`, `hidden`, `enabled`, `disabled`, `checked`, `unchecked`, `focused`, `editable`, `empty` | an element's state |
| `<target> text`, `value`, `attr <name>`, `style <property>` + a comparison | an element's text, value, attribute or computed style |
| `--selector <css> count` `is`, `>`, `>=`, `<`, `<=` `<n>` | how many match |
| `${var}` + a comparison | a saved value |
| `js "<expression>"` (or `eval`) [+ a comparison] | a script in the page: truthy, or its value compared |
| `request <METHOD> <url glob> [status <n>]`, `no request …` | a request the page made in this scenario (`*` in the URL matches anything) |
| `request … json <$.path>` + a comparison | a value in the last matching response's JSON (`$.items[0].sku`) |
| `request … count` + a comparison | how many matching requests |
| `cookie <name>` [+ a comparison], `no cookie <name>`, `cookie <name> absent` | a cookie of the scenario's browser context |
| `storage local\|session <key>` [+ a comparison \| `absent`] | Local or Session Storage |
| `download <file glob>`, `no download …` | a finished download (then `${download.path}`, `${download.name}`: the latest match; a second file of the same name arrives as `name (2).ext`) |
| `console` + a comparison, `no console "<t>"` | a console message (any level) |
| `no errors` | no page exceptions or console errors |
| `not …` | any of them, negated |

  `--soft` on any `expect` notes a failure and goes on; the scenario fails
  at its end with all of them.
