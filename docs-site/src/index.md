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
browser](format.md#api-tests-without-a-browser)).

## Get bitti

Prebuilt binaries for static Linux (musl) x86_64/aarch64, macOS arm64 and
x86_64 and Windows x64, each with its SHA-256, are attached to every
[GitHub release](https://github.com/andrey-usa/bitti/releases) — this site
documents release **0.1.0**. For CI, the
[GitHub Action](running.md#github-actions) installs a release (`version`
picks it: `latest` by default, or a tag).

## Building

bitti links navigera as a library, checked out beside it. CI builds
against navigera's `master`; a release builds against the navigera tag it
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

Next: the [.bt format](format.md), [running and CI](running.md),
the [CLI usage](cli.md) and [worked examples](examples.md).
