# bitti

Plain-text web tests that run on real Chrome. One native binary drives the
browser in-process through [navigera](https://github.com/andrey-usa/navigera):
no Node, no transpile step, no driver process, and no model at run time.

```text
# tests/shop.bt
base http://127.0.0.1:8000

before each
  goto /shop
  click --selector "#cookie-accept"

scenario "Adding a product counts it in the cart" @smoke
  click --text "Add to cart"
  expect --selector "#cart-count" text is 1
  goto /cart
  expect text "Your cart"
```

```console
$ bitti run tests --junit report.xml
PASS  tests/shop.bt:7  Adding a product counts it in the cart  (557 ms)
1 passed, 0 failed in 0.9 s
```

API tests are `.bt` files too: a scenario of `http` steps and their
checks runs with no browser at all, in milliseconds ([API tests without a
browser](#api-tests-without-a-browser)).

Status: 0.1.0. Everything below works in this build.

## The format

- Top-level lines: `base <url>`, `set NAME = value`, `use other.bt`,
  `flow name(a, b)`, `step "<phrase>"` (a Gherkin step's steps, see
  below), `auth name [ttl 30m]`, `before all|each`, `after each|all`,
  `scenario "name" @tag…`. Steps are indented under them.
- A step is a navigera command without the `navigera -s <session>`
  prefix (`goto /cart`, `click --text Add`): `goto`, `click`, `fill`, `select`, `press`,
  `check`, `hover`, `scroll`, `upload`, `wait`, `eval`, `set`, `route`,
  `state load`, and the rest. Targets use navigera's locators (`--role`
  with `--name`, `--label`, `--text`, `--placeholder`, `--testid`,
  `--selector`, `--near <text>` for the one in the card or row that shows
  the text, `--nth`). Refs from `ax` (`click 12`) are an error: they
  change between runs. `check --label Terms` ticks a box; `check
  "<question>"` with no locator asks navigera a yes/no question about
  the page (a model call).
- `set` steps are navigera's: `set viewport 390 844`, `set device "iPhone 16 Pro"`
  (55 devices), `set geo 52.52 13.405`, `set locale de-DE`, and
  `set permissions grant|deny|reset geolocation notifications camera
  microphone clipboard` (also `midi`, `sensors`, `background-sync`), which a
  page then sees as granted, denied or still to be asked. (A `set NAME = value` at the top of a file is
  a variable.)
- A relative path a step reads (`upload`, `state load`, `eval --file`, `expect js --file`,
  `init-script --file`, `route --body-file`, `route --har`) is found next to the `.bt`
  file, as `use` and `for each` files are, so a suite runs the same from
  any directory. Files a step writes (`screenshot`, `state save`) land in
  the current directory.
- Network: `route "*/api/*" --body '{…}'` answers in the server's place,
  `route --har api.har` replays a recorded HAR (navigera's `network --har`
  writes one; requests it has no entry for fail, or reach the network with
  `--not-found continue`), and `route "*/api/me" --modify --replace free
  pro` changes the real response (`--set-header 'Name: value'` too).
  `tests/fixtures/har.bt` has both.
- Words split like a shell's, with no expansion; inside double quotes
  `\"` and `\\` stand for themselves and `\n`, `\r` for a line break
  (a textarea's value). `${NAME}` and
  `${NAME:-default}` come from `save`, a flow's arguments, the data row,
  `before all`, `--var`, `set`, then the environment; `$$` (or `\$`) is a
  literal `$`, and quotes don't stop `${…}` (`'${x}'` is filled in too). Also `${run.id}` (one per run: unique
  emails), `${scenario.name}`, and `${download.path}`/`${download.name}`
  after `expect download`.
- `${secret:NAME}`: from the environment, else `--secrets <file>`
  (`NAME=value` lines; the environment wins). Its value is masked as `***` in everything the run
  prints and saves (messages, the page's URL and title, saved values,
  JUnit and JSON lines).
- A default with spaces is one word: quote it whole,
  `"${GREETING:-hello there}"` (words split before `${…}` is filled in).
- A `set` value can hold `${…}` too (a default, a `--var`, another `set`,
  a secret, the environment): it is filled in once, when a scenario starts.
- `save NAME = <command>` keeps a command's result (`get text …`).
- `print <command>` shows a read command's result in the run's output
  (`print get text --testid total`, `print eval "…"`, `print ax --selector
  main`): no check and no wait, for a look at the page or a value while a
  test is being written. `print http` shows the last `http` answer (its
  status, headers and body), `print body`, `print json <$.path>` and `print
  header <name>` a part of it (JSON indented), and `print "order ${id}"`
  a line with its variables filled in.
- `exec [--timeout-ms <ms>] <program> <args…>` runs a program (no shell:
  `sh -c "…"` or `pwsh -c "…"` for one) in the `.bt` file's directory,
  with `BITTI_BASE_URL` set, for what a browser can't reach: seed a
  database (`psql`, `sqlplus`), call a script. Its exit code and output
  become `${exec.exit}`, `${exec.stdout}` and `${exec.stderr}`, and
  `expect exit 0`, `expect stdout contains SHIPPED`, `expect stderr …`
  check them (the first 16 MB of each). A nonzero exit fails the step,
  with the last line of its stderr, unless a step indented under it checks
  `exit`. It is stopped after 60 s (`--timeout-ms`), and that fails the
  step whatever is under it.

  ```text
  exec ./scripts/order-status.sh ${order}
    expect exit 0
    expect stdout contains SHIPPED
  ```
- `http <METHOD> <url> [--json <text> | --body <text> | --json-file <f> |
  --body-file <f> | --form name=value… | --multipart name=value|name=@file…]
  [--query name=value]… [--user name:password | --bearer <token>]
  [--header "Name: value"]… [--no-follow] [--insecure] [--timeout-ms <ms>]`
  makes a
  request outside the page (an API to set up or check what the UI did, or
  an API test of its own: see [API tests](#api-tests-without-a-browser); a
  relative URL takes the base URL). `--form` sends an urlencoded form,
  `--multipart` a `multipart/form-data` one (`@file` uploads the file beside
  the `.bt`, `;type=image/png` after it sets its type), `--query` adds an
  encoded `name=value` to the URL, and `--user ada:${secret:PW}` or
  `--bearer ${secret:TOKEN}` sets the `Authorization` header (Basic or
  Bearer). The scenario's cookies go with it and
  the cookies it sets come back to the browser, so an API sign-in signs
  the page in. Its answer is `${http.status}`, `${http.body}` and
  `${http.ms}` (how long it took); `expect
  status 201`, `expect json $.role is shopper`, `expect header Location
  ends-with /u/1`, `expect body contains …`, `expect time < 300ms` check it,
  and `save id = json
  $.id` (or `header <name>`) keeps a value. A 4xx or 5xx fails the step,
  with the start of the body, unless a step indented under it checks
  `status`. A timeout fails it whatever is under it. Redirects are followed
  (up to 20, a cookie set on the way going to the next hop and to the
  browser). `--no-follow` gives the 3xx itself. `--insecure` accepts a
  certificate nothing vouches for (a local server's own). The client is
  built in (rustls, the operating system's certificates, the proxy from
  `https_proxy`/`http_proxy`/`no_proxy`): no process per request, and
  connections are kept alive between requests, so a call to a local
  server costs about a millisecond and a secret never reaches a command
  line.

  `--graphql <query>` (or `--graphql-file <f>`) with `--variables <json>`
  and `--operation <name>` sends a GraphQL request as its JSON body. An
  answer with `errors` (GraphQL sends them with a 200) fails the step,
  unless a step under it checks `json $.errors`.

  ```text
  http POST /api/users --json '{"email": "${email}", "role": "shopper"}'
    expect status 201
    expect json $.role is shopper
  save user = json $.id
  ```
- `retry [--timeout-ms <ms>]` with steps indented under it runs them
  again until all of them pass, a little later each time (0.1, 0.25, 0.5,
  then every 1 s), for what happens in the background: a job that
  finishes, an order that ships, a mail that is sent. Each `expect` in it
  looks once (unless it says `--timeout-ms`): the block is the wait. It
  gives up after its `--timeout-ms` (the run's expect timeout, 5 s, by
  default) with the last try's failure.

  ```text
  retry --timeout-ms 20000
    http GET /api/orders/${order}
      expect json $.status is SHIPPED
  ```
- `pause` stops the run there, the browser as the steps before left it,
  open to a second navigera session (the command to attach is printed),
  until Enter. With no terminal it waits `BITTI_PAUSE_S` seconds, and
  under CI not at all unless that is set. A run with a `pause` in it runs
  one worker.
- `clock install [--time 2024-02-02T08:00:00] [--paused]` puts the page's
  time under the scenario's control (navigera's `clock`): `Date`,
  `performance.now` and timers start there, in every page the scenario
  opens; `clock run-for 15m` (or `mm:ss`) advances it, firing the timers
  due on the way in order; `clock fast-forward`, `clock set <time>`,
  `clock pause-at <time>` and `clock resume` do the rest. A session
  timeout, a countdown or a date-dependent page is tested in
  milliseconds, and the next scenario starts on real time.

  ```text
  clock install --time 2024-02-02T08:00:00 --paused
  goto /account
  clock run-for 14:59
  expect text "Signed in"
  clock run-for 1s
  expect text "Session expired"
  ```
- `flow name(a, b)` at the top level holds steps; `run name x y` runs them
  with `${a}` and `${b}` set. `use other.bt` takes another file's flows
  and `set`s. A failure inside a flow names the flow's own line.
- `optional <step>`: a step that may not apply (a cookie banner that only
  sometimes shows); its failure is noted, not fatal.
- `if <expect clause>` with steps indented under it runs them when the
  clause holds now (or within its `--timeout-ms`): `if --selector
  "#banner" visible`, then `click --text Accept` indented below.
- `auth shopper [ttl 30m]` (`s`, `m`, `h`, `d`, or plain seconds) holds steps that sign in; `as shopper` in a
  scenario starts it signed in. The block runs once per run, in a browser
  context of its own, the first time a scenario needs it (the other
  workers wait for it), and its cookies and storage are saved under
  `.bitti/auth/` (keep that out of git). With a `ttl`, the saved state
  is reused by later runs until it is that old.
- `before all` runs once for the file, before its first scenario, and every
  scenario of the file sees what it saves. If it fails, none of them run.
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
  line or the run) runs out; `exit`, `stdout`, `stderr`, `status`, `body`,
  `json` and `header` read the last `exec` or `http` answer once. `is`
  compares numbers as numbers (`1` is `1.0`). A glob (`request`,
  `download`) has `*` for any run of characters and matches the whole URL
  or file name:

| expect | Checks |
|---|---|
| `url`/`title` `is` (or `==`), `contains`, `starts-with`, `ends-with`, `matches` `<value>` | the active tab |
| `text "<t>"`, `no text "<t>"` [`--within <target>`] | visible text, frames included, or inside an element |
| `<target> visible`, `hidden`, `enabled`, `disabled`, `checked`, `unchecked`, `focused`, `editable`, `empty` | an element's state |
| `<target> text`, `value`, `attr <name>`, `style <property>` + a comparison | an element's text, value, attribute or computed style |
| `--selector <css> count` `is`, `>`, `>=`, `<`, `<=` `<n>` | how many match (exactly one `--selector`) |
| `${var}` + a comparison | a saved value |
| `js "<expression>"` (or `eval`) [+ a comparison] | a script in the page: truthy, or its value compared |
| `js --file <script.js>` [+ a comparison] | the same, from a file next to the `.bt` (an expression, or a function body that `return`s its answer): for a check across rows or anything too long to quote |
| `request <METHOD> <url glob> [status <n>]`, `no request <METHOD> <url glob>` | a request the page made in this scenario (`*` in the URL matches anything) |
| `request … json <$.path>` + a comparison | a value in the last matching response's JSON: `$`, `.key`, `[0]`, `["key"]` (`$.items[0].sku`) |
| `request … body` + a comparison | the last matching response's body, as text (`body contains "added to cart"`) |
| `request … count` + a comparison | how many matching requests |
| `cookie <name>` [+ a comparison], `no cookie <name>`, `cookie <name> absent` | a cookie of the scenario's browser context |
| `storage local\|session <key>` [+ a comparison \| `absent`] | Local or Session Storage |
| `download <file glob>`, `no download …` | a finished download (then `${download.path}`, `${download.name}`: the latest match). Each scenario's downloads go to a directory of its own beside its evidence (`.bitti/runs/<time>/<scenario>/downloads`, kept only when something came), so a name never carries over from another scenario; a second file of the same name in one scenario arrives as `name (2).ext` |
| `download <file glob> compare <expected file>` [`--key <columns>`] [compara flags…] | the download is the same table as the expected file beside the `.bt` ([compara](https://github.com/andrey-usa/compara) on the PATH, or `BITTI_COMPARA`): CSV, NDJSON or Parquet, either side; compara's flags (`-i`, `--trim`, `--tolerance`) pass through. A difference fails the step with compara's counts and the reports it kept beside the scenario's downloads: `--report html,json,rows` (the HTML report, the JSON summary of counts and column statistics, CSVs of the changed, added and removed rows) or `none`; the run's `--compare-report` (`compare_report` in `bitti.toml`, default `html`) when the step doesn't say, and compara's own `-o`, `--json` or `--export-dir` win |
| `console` + a comparison, `no console "<t>"` | a console message (any level) |
| `no errors` | no page exceptions or console errors |
| `accessible` [`--selector <css>`] [`--tags wcag2a,wcag2aa,…`] [`--ignore <rule>,…`] | [axe-core](https://github.com/dequelabs/axe-core) (built in) finds no violation in the page, or under the selector: WCAG 2.0 and 2.1 A and AA by default; `--tags` replaces that set (`--tags wcag2a,wcag2aa,best-practice` to add axe's best practices); a failure names each rule, its impact, the first elements and what to fix |
| `lcp`, `fcp`, `ttfb`, `load`, `inp` `<`, `<=`, `>`, `>=` `<time>` (`2.5s`, `800ms`; a bare number is ms); `cls` + a comparison and a score (`0.1`) | a performance budget for the current page, as the browser measured it (navigera's `vitals`): largest and first contentful paint, time to first byte, the load event (ms from the navigation's start); the layout shift score (its largest session window, shifts after input left out); interaction to next paint, the slowest of the scenario's clicks and key presses on this page so far (put it after them). A miss names the element behind it (`3120 (#hero)`). Not measured yet (the load event still to come, no interaction yet) is retried. Chrome keeps only interactions of 104 ms or more, so `inp < 200ms` passes when every one was quicker, and a budget under 104 ms is refused. Run under `set cpu 4` and `set network fast-3g` to measure as a mid-tier phone |
| `exit` [`is`] `<n>`, `stdout`/`stderr` + a comparison | what the last `exec` gave |
| `status` [`is`] `<n>`, `body` + a comparison, `json <$.path>` [+ a comparison], `header <name>` [+ a comparison] | the last `http` answer; a JSON object or array compared with `is` is compared as JSON (key order and `1` vs `1.0` don't matter), with `contains` as a part of it (`json $ contains '{"role": "shopper"}'`: the keys it names, at any depth; for an array, items that some item matches) |
| `json <$.path> count` + a comparison | how many items the array (or keys the object) at the path has |
| `json [<$.path>] schema <file.json>[#<pointer>]` | the answer (or the value at the path) is valid against a JSON Schema beside the `.bt`, or one inside a larger document (`openapi.json#/components/schemas/Item`: an OpenAPI 3 document in JSON, its `$ref`s read in the whole document); a failure lists where it isn't (`$[1].price: expected number, got string "5"`). See [API tests](#api-tests-without-a-browser) for what it reads |
| `json [<$.path>] snapshot <name>` [`--ignore <$.path>,…`] | the answer (or the value at the path) is what its baseline beside the `.bt` holds, `<file>.bt.snapshots/<name>.json`, compared as JSON; `--ignore $.id,$.items[*].created` leaves out what changes from run to run. With no baseline yet the run writes it and the step fails, so a person reads it first; `--update-snapshots` writes or replaces it. A difference names each path (`$.total: 38, was 36`) |
| `time` `<`, `<=`, `>`, `>=` `<time>` (`300ms`, `1.5s`) | how long the last `http` exchange took, redirects included |
| `ax [<target>] matches` + the tree indented under it | the page's (or the target's) accessibility tree, as `navigera ax` prints it, holds those lines: partially (`button` matches `button "Place order"`) and in order, a line indented under another inside what that one matched, refs ignored, `/regex/` (spaces allowed) for a name or a value |
| `screenshot <name> [<target>]` [`--full-page`] [`--threshold 0.2`] [`--max-diff-pixels <n>` \| `--max-diff-ratio <r>`] [`--mask <css>`]… | the viewport (or the element, or the whole page) as its baseline shows it, pixel by pixel; see below |
| `not …` | any of them, negated |

  `--soft` on any `expect` notes a failure and goes on; the scenario fails
  at its end with all of them. A `${var}` can stand for any compared value
  (a count, a regex), checked once it is filled in; `--timeout-ms` takes a
  number as written.

  ```text
  expect ax --role form --name Checkout matches
    - textbox "Email" value=/.+@example\.com/
    - combobox "Country" value="Canada"
    - button "Place order"
  ```

  Each template line matches a tree line holding its words in order,
  indented lines inside the line above's subtree. Sibling `- text:` lines
  count as one text on both sides, so a template keeps matching however
  navigera splits adjacent text into lines.

  A screenshot's baseline is beside the `.bt` file, in
  `<file>.bt.screenshots/<name>-<os>.png` (fonts differ between systems,
  so each OS keeps its own). Keep it with the tests. With none yet the run
  writes it and the step fails, so a person looks at it first.
  `--update-snapshots` (or `--update-screenshots`) writes or replaces
  baselines and passes. While it
  is taken, animations and transitions stop, the caret hides and each
  `--mask` element is painted over in magenta (a mask that matches nothing
  fails the step). An element is scrolled into view first, inside its
  scroll containers and frames, and cropped where it is on the page, in an
  iframe too. Two pixels differ when their
  colour distance is over `--threshold` (0 to 1). `--max-diff-pixels` and
  `--max-diff-ratio` (a share of the pixels, 0 to 1) set how many may
  differ. When both are given, both must hold, and with neither, none may. A
  mismatch writes this run's image and a diff (the differing pixels red) to
  the evidence directory and names both.

  Copy the lines from `navigera ax` and keep the ones that matter. A miss
  names the first line that has no place and the nearest line of its role
  (`no line matches \`- combobox "Country" value="Mexico"\`; nearest:
  \`- combobox "Country" value="Canada" options: …\``, the line as `ax`
  prints it).

## API tests without a browser

A scenario whose steps never touch a page (`http`, `exec`, their checks,
checks of saved values and of cookies, `save … = json|header`, `print` of
an answer, `retry`, `wait --ms`, a `flow` or an `auth` made of those) runs
with no browser at all, and a run of only such scenarios starts none:
nothing to install on the machine, and each request costs about a
millisecond on a kept-alive connection.

```text
base http://127.0.0.1:8000

auth api
  http POST /api/login --form user=ada --form "password=${secret:PW}"

scenario "An order can be placed and ships" @api
  as api
  http POST /api/orders --json '{"sku": "sku-1", "qty": 2}'
    expect status 201
    expect json $ schema order.schema.json
    expect time < 500ms
  save order = json $.id
  retry --timeout-ms 20000
    http GET /api/orders/${order}
      expect json $ contains '{"status": "SHIPPED", "items": [{"sku": "sku-1"}]}'
```

```console
$ bitti run tests/api
PASS  tests/api/orders.bt:6  An order can be placed and ships  (412 ms)
1 passed, 0 failed in 415 ms (no browser)
```

- Such a scenario keeps its cookies the way a browser would (domain, path,
  `Secure`, expiry; a cookie set on a redirect goes with the next hop):
  what its sign-in sets goes with its later requests, and `expect cookie
  sid` checks one.
- An `auth` of `http` steps signs in with no browser too, and saves its
  cookies in navigera's `state save` format: a browser scenario `as` the
  same auth starts signed in, and an API scenario `as` an auth that signed
  in through the page takes its cookies.
- A failure shows the request and what came back:

  ```text
  FAIL  orders.bt:6  An order can be placed and ships  (31 ms)
        line 10: expect json $ schema order.schema.json
        expected json $ to match schema order.schema.json; it doesn't: $.total: expected number, got string "36.00"
        last http: POST http://127.0.0.1:8000/api/orders -> 201 (9 ms, 61 B, application/json)
          {"id": "ORD-7", "status": "NEW", "total": "36.00", "items": [...]}
  ```
- `--trace` records each step with the requests it made, and `--html`,
  `--junit`, `--allure` report these scenarios as any other.
- The schema check reads what an API's contract uses, in drafts 4 to
  2020-12 alike: `type` (and OpenAPI 3.0's `nullable`), `enum`, `const`,
  `properties`, `required`, `additionalProperties`, `patternProperties`,
  `propertyNames`, `min/maxProperties`, `dependentRequired`, `items` (or a
  list of them), `prefixItems`, `contains`, `min/maxItems`, `uniqueItems`,
  `min/maxLength`, `pattern`, `format` (`date-time`, `date`, `time`,
  `email`, `uuid`, `uri`, `ipv4`, `ipv6`), `minimum`/`maximum` and their
  exclusive forms, `multipleOf`, `allOf`, `anyOf`, `oneOf`, `not`,
  `if`/`then`/`else`, and `$ref` within the file (`#/$defs/…`,
  `#/definitions/…`). Other keywords are ignored, as validators do.

## Gherkin

`.feature` files run as they are, beside `.bt` files (no generated code):
`Feature`, `Background` (before each scenario), `Scenario`/`Example`,
`Scenario Outline` (or `Scenario Template`) with `Examples` (or
`Scenarios`) tables, several allowed, each with its own tags (`<column>`
filled in; one scenario per row, named `Name (col=value, …)` when the name
has no `<…>`), `Rule` (a `Background` under it runs before that rule's
scenarios only), tags (a feature's go on all its scenarios), and
`Given`/`When`/`Then`/`And`/`But`/`*` steps, all alike. English keywords
only.

A step is one of the run's own `step` blocks, from any `.bt` file it loads:

```text
step "I sign in with {password}"
  fill --label Password ${password}
  click --text "Sign in"
```

`{name}` takes a `"…"` or `'…'` value or one word and is `${name}` in the
steps under it. A line two `step` blocks match is an error. Else it is a
phrase of the built-in book: `I open "/cart"` (`go to`, `navigate to`,
`visit`, `am on`), `I click "Apply"` (`click on`; by visible text), `I fill
"Email" with "a@b.c"` (`fill in`, `enter`; by label), `I type "x" into
"Search"`, `I select "Canada" from "Country"` (or `in`), `I check "Terms"`,
`I uncheck …`, `I press "Enter"`, `I wait for "Saved"`, `I see "Saved"` (`I
should see`), `I don't see "Error"` (`should not`, `shouldn't`, `do not`),
`the URL is|contains|starts with|ends with|matches "…"` (`url`, `page URL`),
`the title is|contains … "…"` (`page title`). An icon button or a field
with no label needs a `step` block (`click --role …`, `--placeholder`). A
step nothing matches is a file error that says how to write it. Data tables and doc strings under a
step aren't supported yet.

`--cucumber <file>` writes Cucumber JSON for the `.feature` files, step by
step (passed, failed, skipped, with times), for test-management importers.
`--junit`, `--html`, `--allure` and the console report them like any
scenario.

## Running

```text
bitti run [<file.bt|file.feature|dir>...] [--base-url <url>] [--var k=v] [--grep <text>] [--grep-invert <text>]
              [--tag <tag|expression>] [--last-failed] [--max-failures <n>|-x] [--forbid-only|--allow-only]
              [--workers <n|auto>|-j <n>] [--isolation context|browser] [--retries <n>] [--fail-on-flaky] [--shard <i/n>]
              [--repeat <n>] [--shuffle [<seed>]] [--slowmo <ms>]
              [--reporter console|jsonl|github] [--junit <file>] [--cucumber <file>] [--allure <dir>] [--html <file>] [--json]
              [--timeout-ms <ms>] [--wait-ms <ms>] [--headed] [--chromium <path>] [--engine chrome|lightpanda]
              [--electron <app> [--electron-arg <arg>]...] [--attach <port|ws://…>]
              [--secrets <file>] [--evidence <dir>] [--pause-on-failure] [--trace on-failure|all] [--video on-failure|all] [--update-snapshots]
              [--compare-report html,json,rows|none] [--config <file>] [--env <name>]
bitti lint [<file.bt|file.feature|dir>...]
bitti list [<file.bt|file.feature|dir>...]
bitti trace <dir> [<line>]
bitti fix [<file>:<line>] [--use <n>] [--all [--min <p>]]
bitti navigera <command…>
bitti from-session <name> [--scenario <name>] [--out <file.bt>]
bitti record <url> [--out <file.bt>] [--scenario <name>] [--headless] [--for <seconds>]
bitti record --extend <file.bt> [--scenario <name>] [--base-url <url>] [--secrets <file>]
bitti mcp [--headed] [--chromium <path>]
bitti skill          # the agent skill (what an agent reads to write tests)
bitti <command> --help   # that command's usage and options
bitti --version
```

`--base-url` falls back to `base` in the file, then `BITTI_BASE_URL`.
`--workers auto` is half the CPUs, at most 8.

- `--tag smoke` runs the scenarios tagged `@smoke`; a tag expression picks
  by several, as Cucumber writes them: `--tag "smoke and not slow"`,
  `--tag "(checkout or cart) and not wip"`. Repeated, any of them.
  `--grep-invert <text>` leaves out the names that contain it.
- Some tags say how a scenario runs: `@skip` and `@fixme` aren't run and
  are reported as skipped (`SKIP`, JUnit `<skipped/>`, Allure `skipped`);
  `@slow` gives each check three times its time; `@serial` (or
  `@exclusive`) runs the scenario while no other one runs, for one that
  changes what the others read (an app setting, a shared account): the
  others run in parallel first, then each `@serial` one alone; `@only` runs only the
  scenarios that have it, a focus while working on one. It is an error
  with `--forbid-only`, which is on by itself under CI (`CI` set), so a
  focus left in by mistake doesn't pass a build on a few scenarios
  (`--allow-only` runs just those anyway).
- `--max-failures <n>` starts no scenario once n have failed (`-x`: at the
  first), and says how many were not run; the run fails.
- `--last-failed` (`--lf`) runs only the scenarios that failed in the last
  run in this directory (`.bitti/last-run.json`). With none, it says
  so and exits 0.
- `--scenario-timeout-ms <ms>` (`scenario_timeout_ms` in `bitti.toml`)
  is the most one scenario's steps may take: no step starts once it is
  up, and a step's wait (an `expect`, a `retry`, an `http` or `exec`, its
  own `--timeout-ms` too) stops there, the failure saying so. `@slow`
  triples it. Off by default. `--global-timeout-ms <ms>`
  (`global_timeout_ms`) does the same for the whole run: no scenario
  starts after it, those running stop there, and the run fails with the
  count of scenarios it never ran.
- `--slowmo <ms>` waits that long before each step that acts on the page
  (`goto`, `click`, `fill` …), to follow a `--headed` run by eye; checks
  and `http` steps aren't slowed.

- A click that opens a tab or a popup (`target=_blank`, `window.open`)
  moves the scenario to it, as navigera does: the next steps read and act
  there. `tab-select 0` goes back to the first tab; `tab-close` closes the
  one you're on.
- Every scenario starts in a fresh browser context (navigera `reset
  --all`: no cookies, storage, tabs, routes, init scripts, `set`
  emulation or dialog answer from the scenario before), in about 40 ms
  instead of a new browser's few hundred. `--workers n` runs n at once,
  each in a browser context of one shared browser (about 40% less memory
  at 4 workers than a browser each; `--isolation browser` gives each its
  own). The longest start first (by `.bitti/timings.json`, which each
  run updates; the first run goes in file order).
- `--engine lightpanda` (or `engine = "lightpanda"` in a profile, or
  `NAVIGERA_ENGINE=lightpanda`) runs on
  [Lightpanda](https://github.com/lightpanda-io/browser) (`$LIGHTPANDA_BIN`
  or `lightpanda` on the PATH), a new Lightpanda for each scenario (it has
  one context per process): less memory, no layout. It is a smoke tier, not
  a replacement: hover-revealed elements, new tabs, screenshots and `ax`
  pixels need Chrome. On the 40-scenario suite's 25 site scenarios it
  passes 22, and is not faster: 22 s against Chrome's 11 s at one worker
  (half of it the three failures' waits).
- `--electron <app>` (or `electron = "./app"` and `electron_args = [...]`
  in `bitti.toml`) tests an Electron desktop app instead of a web
  page: the scenarios drive its windows as tabs, through navigera's
  `start --electron`. `<app>` is the app's directory or main script (the
  Electron in its `node_modules` runs it), a packaged app's executable or a
  macOS `.app` bundle. Each scenario gets a fresh app, so none sees what the
  one before left in its storage, and they run one at a time (two copies of
  an app would share its user data): `--workers` becomes 1. A window the
  app opens itself (Settings, a second document) becomes the scenario's
  tab, and one that closes itself hands back to the one before. `expect js
  --main "<code>"` checks the app's main process (a function gets the
  `electron` module: `({ app }) => app.getName()`), and `eval --main` runs
  code there as a step. On a Linux
  CI machine, run under `xvfb-run`. The example: `tests/fixtures/electron`,
  run in CI on Linux, macOS and Windows against navigera's fixture app.
- `--attach <port|ws://…>` (or `attach = "9222"` in `bitti.toml`) runs
  against a browser or an Electron app that is already running with a
  DevTools port: the build a developer has open (`npm start --
  --remote-debugging-port=9222`), or a Chrome they started. In an app, the
  scenarios run one after the other in the one app, each going on where
  the one before left it, and the app keeps running after the run. In a
  browser, each scenario gets a clean context of its own. One worker. The
  example: `tests/fixtures/electron-attached`.
- `--html report.html` writes the run as one page that needs nothing
  else to open: failed scenarios first and open, each with its steps and
  their times, the failure (message, where the page was, what came
  closest, the note) and its screenshot in the page (up to 2 MB; a
  bigger one is linked). Flaky ones show how their first try failed.
  Secrets are masked as in the console. A screenshot two scenarios share
  (a failed `before all`) is in the page once; a bigger one, and any once
  the page holds 10 MB of them (failures first), is a link relative to the
  report, so upload the report and `.bitti/runs/`
  together. With `--repeat`, a pass count per scenario comes first. A run
  that finds no scenario (exit 2) writes no report.
- `--allure <dir>` writes Allure 2 results for `allure generate <dir>` or an
  Allure server: one `<uuid>-result.json` per scenario run with its steps,
  times, status (a flaky one passed, marked flaky), labels (the file, the
  `.feature`'s name, tags, the worker) and the failure's screenshot
  attached. The directory is kept, so several runs or shards can feed one
  report; the history id is the same from run to run.
- `--video on-failure` (or `all`) records what the page showed while the
  scenario ran, a JPEG each time its pixels changed (none while it is
  still), into `video/` in the scenario's evidence; `video/index.html`
  plays the frames at their pace. The console and JSON lines name it and
  `--html` links it. A passing scenario's video is dropped under
  `on-failure`. Recording cost no measurable time on the examples.
- `--trace on-failure` (or `all`) records each step of a scenario: where
  the page was, the console messages and the requests it brought, in
  `trace.jsonl` beside the scenario's screenshot, with the page's ax tree
  at a failure in `ax.txt`. `bitti trace <dir>` prints it (a run's
  directory lists its traces); `bitti trace <dir> <line>` shows one
  step in full, with the tree when it is the one that failed. Each step
  also keeps the page as it was after it, beside `trace.jsonl`: `NNN-L<line>.ax.txt`
  (its ax tree) and `NNN-L<line>.jpg` (the viewport, JPEG quality 60, about
  10 to 50 kB), both named in the step's line (`ax`, `shot`); `trace <dir> <line>`
  shows them. In `--html` a traced scenario's steps are a timeline: a row
  per step (line, step, time, a bar against the slowest) that opens to its
  screenshot, ax tree, console messages, requests and error, with no script
  in the page (the images follow the 2 MB and 10 MB limits above). Secrets
  are masked in the text files and the console as everywhere; a screenshot
  is pixels, as the failure's own is.
- `--watch` runs the files, then waits: each scenario file that changes
  (or appears) runs again, its own scenarios only, until Ctrl-C. The files
  are looked at every 300 ms (no file-watching dependency) and an edit
  counts once it has stayed quiet for one more look. A file that no longer
  parses is reported and the watching goes on. The files a `use` or a flow
  brings in are not watched on their own: save the scenario file to run it.
  `--junit` and `--html` hold the latest run.
- `bitti merge <dir|file.jsonl>...` joins the results of shards into one
  console summary, one `--junit` and one `--html`, as if one run had made
  them: save each shard's `--reporter jsonl` output
  (`bitti run --shard 1/3 --reporter jsonl > shard-1.jsonl`), put the
  files in one place and name it. A directory is searched for `.jsonl`
  files. The steps and traces stay with the shards, so the report has each
  scenario's status, time and failure; the time is the longest shard's.
  Exit 1 when a scenario failed (`--fail-on-flaky` counts a flaky one),
  2 when there are no results.
- `--retries n` runs a failing scenario again. One that passes then is
  reported FLAKY with how its first try failed. `--fail-on-flaky` fails
  the run on one. `--shard i/n` takes every n-th scenario, for CI matrix
  jobs.
- `--repeat 20 --shuffle` hunts flakes and order dependence: every
  scenario runs 20 times (`name #k`), in a random order whose seed is
  printed (`--shuffle <seed>` runs that order again), then a pass count per
  scenario, FLAKY where it isn't all of them.
- `--reporter jsonl` prints one JSON object per scenario as it finishes
  (the console moves to stderr); `github` writes workflow annotations on
  the failing lines, beside the console (on by itself under GitHub
  Actions). `--json` prints
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

  When that list is unsure (the best under 0.6, or the first two within
  0.1) and a decision model is set up for navigera (`TYPESAFE_API_KEY`, or
  `NAVIGERA_DECIDER_URL`), bitti also asks it which element the step
  meant (navigera's `find`, on the failing page) and adds its pick, marked
  `(decider)`: first when the model is at least 0.5 sure, else last (the
  list stays at three). There is one call per such failure and none otherwise. The
  list covers frames too: a step with `--frame` keeps it, and `fix` changes
  only the locator.
- `bitti fix <file>:<line> [--use <n>]` puts the last run's n-th
  "nearest on the page" into that step (only its locator flags change, and
  only if the line still reads as it did); `bitti fix --all [--min
  0.6]` takes each failure's first suggestion that scored that much, and
  plain `bitti fix` lists them. The failure block prints the command.
  A click that something else covers (a consent bar, a modal) gets that
  thing's buttons instead, "Accept", "Close", "Got it" … first, as an
  `optional click` that `fix` puts before the step.
  A `.feature` step is left alone: change its `step` block instead.
- `bitti navigera …` is navigera itself, built in: explore a page the
  way an agent does (`bitti navigera -s work start`, `goto`, `ax`,
  `click 12`, `fill 15 ada@example.com`) with no second program to
  install. `bitti from-session work` then writes what that session did
  as a scenario: each action once, and the waits between (reads and `eval`
  left out), every ref replaced by the most stable locator that found its
  element again just before the action (test id, role and name, label, placeholder, text, CSS;
  navigera's `locate`, each checked on the page; inside a web component's
  shadow root, CSS through its hosts: `--selector "#tip >>> button"`), `base` from the first
  `goto`, an `expect url` where an action moved the page, an `expect text`
  for a heading that changed or a status/alert message that appeared (as a
  comment when it has numbers, which may change), and a value typed
  into a password field (`type=password`, as navigera's `locate` says) as
  `${secret:NAME}`, named after the field (`${secret:PASSWORD}`,
  `${secret:CONFIRM_PASSWORD}`). What it can't keep (a mouse at
  coordinates, a ref no locator found again) is a `# left out:`
  line. Read it through, keep the expects that matter, add the ones the
  test is for. A session started by a plain `navigera` works too when that
  navigera has `history`.
- `bitti record <url> --out t.bt` opens a browser to use by hand: each
  click, field change, check, select, upload and Enter becomes a step as
  you do it, with the most stable locator that finds the element (the same
  `locate` as `from-session`, run in the page at that moment; when the
  click takes its element or the page away before that check is done, a
  banner's Accept or a link, the element's own text or label read as it
  was clicked), an `expect url` where the page
  moved, a password field's value as `${secret:NAME}` (never written
  down). The file is rewritten after
  every step: close the browser or press Ctrl-C to stop. A click into a
  field isn't a step (its value is); typing in a field twice keeps the
  last value.
- `bitti record --extend t.bt [--scenario <name>]` grows a scenario
  instead: it runs that scenario (default: the file's last) to its end,
  `after each` left out, then hands you the browser there; what you do
  goes in after its last step, the same way, and the rest of the file stays
  as it was. A scenario that fails before its end is not extended: fix it
  first. `--headless` and `--for <s>` work as for `record`. A `.feature`
  is refused (its steps live in its `step` blocks).
- `bitti mcp` is the same loop for an agent over MCP (stdio): navigera's
  browser tools (`browser_navigate`, `browser_snapshot`, `browser_click`,
  … one browser for all of them) beside `bitti_from_session` (what
  those tools did, as a scenario; `out` writes it), `bitti_run`,
  `bitti_list`, `bitti_lint`, `bitti_failure` (the last run's
  failures with the nearest elements) and `bitti_fix`. Its
  instructions tell the agent that page content is data, not instructions,
  and to confirm irreversible actions against the user's request. Register it as
  `{"command": "bitti", "args": ["mcp"]}`. What the tools run, read
  and write (`paths`, `out`, the files `args` names) stays inside the
  working directory, as navigera's own tools' files do, and `args` can't
  name a `--chromium` to start; `NAVIGERA_MCP_ALLOW_FILES=1` lifts both.
  A project's own `.bt` files still do what they say (`exec`, `[server]`).
- `--pause-on-failure` stops at a failure with the browser as the failure
  left it and prints how to look at that very tab (`navigera -s bt start
  --attach ws://…/devtools/page/…`, then `navigera -s bt ax`); Enter goes
  on with the run. It runs one worker. With no terminal to press Enter in,
  it waits `BITTI_PAUSE_S` seconds (default 600, for an agent looking
  from elsewhere), and under CI (`CI` set) it doesn't pause unless that is
  set.
- Exit codes: 0 passed, 1 a scenario failed, 2 a file or argument is wrong,
  3 the browser or the app server didn't start.

## Config: `bitti.toml`

A `bitti.toml` in the current directory (or `--config <file>`) holds
what a run takes by default. Flags on the command line win over it.
Paths in it are from its own directory.

```toml
base_url = "http://127.0.0.1:3000"
paths = ["tests"]            # what to run when the command line names nothing
workers = 4                  # or "auto"
retries = 1
reporters = ["github"]       # added to --reporter
junit = "reports/junit.xml"  # also html, cucumber, allure, secrets, evidence, chromium
trace = "on-failure"         # or "all", "off"
compare_report = ["html", "json"]  # what `expect download … compare` keeps
[vars]                       # as --var; --var on the command line wins
region = "us"

[server]                     # the app under test
command = "npm run dev"      # sh -c (cmd /C on Windows)
cwd = "app"                  # where it runs (default: the config's directory)
url = "http://127.0.0.1:3000/health"
timeout_s = 60
# reuse = true               # one already answering at url is used (default, except under CI: CI set and not empty, false or 0)
# env = { PORT = "3000" }

[env.staging]                # --env staging: on top of the rest
base_url = "https://staging.example.com"
[env.staging.vars]
region = "eu"
```

The `[server]` starts before the browser and is ready when its `url`
answers (any status under 500). Its output goes to `.bitti/server.log`,
and it is stopped with everything it started when the run ends. One that
exits or doesn't answer in time ends the run with exit 3 and the end of its
log. Other keys: `timeout_ms`, `wait_ms`, `isolation`, `fail_on_flaky`,
`headed`, `slowmo`, `max_failures`, `forbid_only`. An unknown key is an
error.

## GitHub Actions

```yaml
- uses: andrey-usa/bitti@next
  with:
    paths: tests
    base-url: http://127.0.0.1:3000
    args: --workers auto --retries 1 --junit report.xml
```

It runs on the runner's Chrome (Linux, macOS, Windows). `version` picks a
release (`latest` by default, or a tag). With no release binary for the
runner, it builds bitti from the action's own commit (`source` asks for
that), with navigera at `navigera-ref`. Failures are annotations on the
lines that failed. The `evidence` output is the directory holding their
screenshots, for `actions/upload-artifact`.

## Container

`Dockerfile` builds an image with bitti and Debian's headless Chromium,
amd64 or arm64, from the directory holding both checkouts (see Building):

```console
$ docker build -f bitti/Dockerfile -t bitti .
$ docker run --rm --network host -v "$PWD/tests:/tests:ro" bitti run /tests --base-url http://127.0.0.1:3000
```

It runs as an unprivileged user (any uid, so OpenShift's random ones too),
with a read-only root filesystem if asked (`HOME=/tmp`, a `/tmp` to write
to, and a writable working directory: `.bitti/` with the last run,
timings and evidence goes there, `/work` by default), and without Chrome's own sandbox: a container without user
namespaces can't give Chrome one, and the container is the boundary.
`examples/kubernetes/job.yaml` is a Job: the tests from a ConfigMap, JUnit
and evidence in `/work/out`, a memory-backed `/dev/shm`.

## Building

bitti links navigera as a library, checked out beside it. CI builds
against navigera's `master`. A release builds against the navigera tag it
names:

```text
work/
  navigera/     git clone https://github.com/andrey-usa/navigera (master)
  bitti/    this repository
```

```console
$ cd bitti && cargo build --release
$ ./target/release/bitti run examples --base-url http://127.0.0.1:<port>
```

`examples/bench-site` runs against navigera's bench site
(`python3 ../navigera/bench/site/server.py --port 0`).

## License

MIT. `vendor/axe-core` is axe-core, unmodified, under the Mozilla Public
License 2.0 (its `LICENSE` beside it).
