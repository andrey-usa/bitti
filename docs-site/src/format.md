# The .bt format

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
  `status`; a timeout fails it whatever is under it. Redirects are followed
  (up to 20, a cookie set on the way going to the next hop and to the
  browser); `--no-follow` gives the 3xx itself. `--insecure` accepts a
  certificate nothing vouches for (a local server's own). The client is
  built in (rustls, the operating system's certificates, the proxy from
  `https_proxy`/`http_proxy`/`no_proxy`): no process per request, and
  connections are kept alive between requests, so a call to a local
  server costs about a millisecond and a secret never reaches a command
  line.

  `--graphql <query>` (or `--graphql-file <f>`) with `--variables <json>`
  and `--operation <name>` sends a GraphQL request as its JSON body; an
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
  until Enter; with no terminal it waits `BITTI_PAUSE_S` seconds, and
  under CI not at all unless that is set. A run with a `pause` in it runs
  one worker.
- `clock install [--time 2024-02-02T08:00:00] [--paused]` puts the page's
  time under the scenario's control (navigera's `clock`): `Date`,
  `performance.now` and timers start there, in every page the scenario
  opens; `clock run-for 15m` (or `mm:ss`) advances it, firing the timers
  due on the way in order; `clock fast-forward`, `clock set <time>`,
  `clock pause-at <time>` and `clock resume` do the rest. A session
  timeout, a countdown or a date-dependent page is tested in
  milliseconds; the next scenario starts on real time.

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
  `.bitti/auth/` (keep that out of git); with a `ttl` the saved state
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
| `lcp`, `fcp`, `ttfb`, `load` `<`, `<=`, `>`, `>=` `<time>` (`2.5s`, `800ms`; a bare number is ms); `cls` + a comparison and a score (`0.1`) | a performance budget for the current page, as the browser measured it: largest and first contentful paint, time to first byte, the load event (ms from the navigation's start); the layout shift score (its largest session window, shifts after input left out). Not measured yet (the load event still to come) is retried |
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
  so each OS keeps its own); keep it with the tests. With none yet the run
  writes it and the step fails, so a person looks at it first;
  `--update-snapshots` (or `--update-screenshots`) writes or replaces
  baselines and passes. While it
  is taken, animations and transitions stop, the caret hides and each
  `--mask` element is painted over in magenta (a mask that matches nothing
  fails the step). An element is scrolled into view first, inside its
  scroll containers and frames, and cropped where it is on the page, in an
  iframe too. Two pixels differ when their
  colour distance is over `--threshold` (0 to 1). `--max-diff-pixels` and
  `--max-diff-ratio` (a share of the pixels, 0 to 1) set how many may
  differ; when both are given both must hold, and with neither none may. A
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
steps under it; a line two `step` blocks match is an error. Else it is a
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
step (passed, failed, skipped, with times), for test-management importers;
`--junit`, `--html`, `--allure` and the console report them like any
scenario.
