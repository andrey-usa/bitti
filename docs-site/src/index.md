# bitti

Plain-text web tests that run on real Chrome — faster, with less memory:
one native binary drives the browser in-process through
[navigera](https://andrey-usa.github.io/navigera/), no Node, no transpile
step, no driver process, and no model at run time.

> **Name:** this project was renamed **bitti → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `bitti`
> and its messages still say `bitti`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

## A scenario

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

## Build and run

## Building

bitti links navigera as a library, checked out beside it:

```text
work/
  navigera/     github.com/andrey-usa/navigera (or navigera-private)
  bitti/    this repository
```

```console
$ cd bitti && cargo build --release
$ ./target/release/bitti run examples --base-url http://127.0.0.1:<port>
```

`examples/bench-site` runs against navigera's bench site
(`python3 ../navigera/bench/site/server.py --port 0`).

Next: the [.bt format](format.md), [running and CI](running.md),
the [CLI usage](cli.md) and [worked examples](examples.md).
