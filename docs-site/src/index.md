# bitti

Plain-text web tests that run on real Chrome — faster, with less memory:
one native binary drives the browser in-process through
[navigera](https://andrey-usa.github.io/navigera/), no Node, no transpile
step, no driver process, and no model at run time.

> **Name:** this project was renamed **primavera → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `primavera`
> and its messages still say `primavera`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

## A scenario

```text
# tests/shop.pv
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
PASS  tests/shop.pv:7  Adding a product counts it in the cart  (557 ms)
1 passed, 0 failed in 0.9 s
```

## Build and run

## Building

primavera links navigera as a library, checked out beside it:

```text
work/
  navigera/     github.com/andrey-usa/navigera (or navigera-private)
  primavera/    this repository
```

```console
$ cd primavera && cargo build --release
$ ./target/release/primavera run examples --base-url http://127.0.0.1:<port>
```

`examples/bench-site` runs against navigera's bench site
(`python3 ../navigera/bench/site/server.py --port 0`).

Next: the [.pv format](format.md), [running and CI](running.md),
the [CLI usage](cli.md) and [worked examples](examples.md).
