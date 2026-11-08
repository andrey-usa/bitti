# Worked examples

> **Name:** this project was renamed **bitti → bitti** on
> 2026-11-08 (bitti is Kazakh *бітті* — "it is done, finished"). Until the
> code-rename release lands, the binary is still invoked as `bitti`
> and its messages still say `bitti`; the pages below are verbatim
> from the current README and `--help`, so they use the old name.

From `examples/bench-site/` in the repo (they run against navigera's bench site).

## `shop.bt`

```text
# navigera's bench site (navigera/bench/site/server.py): a small shop.
# bitti run examples/bench-site --base-url http://127.0.0.1:<port>

before each
  goto /shop
  click --selector "#cookie-accept"

scenario "Adding a product counts it in the cart" @smoke
  click --text "Add to cart"
  expect --selector "#cart-count" text is 1
  goto /cart
  expect text "Your cart"
  expect --selector "#subtotal" text starts-with "$"

scenario "Every scenario starts with an empty cart" @smoke
  expect --selector "#cart-count" text is 0

scenario "Search narrows the list"
  fill --label "Search products" valve
  press Enter
  expect --role status text contains "Showing"
  expect --selector ".card" count >= 1
  expect no text "Titanium Bolt"

scenario "A coupon in a web component lowers the total" @cart
  click --text "Add to cart"
  expect --selector "#cart-count" text is 1
  goto /cart
  save subtotal = get text --selector "#subtotal"
  expect ${subtotal} matches '^\$[0-9]+\.[0-9]{2}$'
  fill --label "Coupon code" SAVE10
  click --role button --name "Apply coupon"
  expect text "Coupon SAVE10 applied"
  reload
  expect text "Discount (SAVE10)"
```

## `account.bt`

```text
set email = demo@acme.test

scenario "Signing in shows the orders"
  goto /login
  fill --label Email ${email}
  fill --label Password ${PASSWORD:-hunter2}
  click --role button --name "Sign in"
  expect url ends-with /account
  expect text "Signed in as"
  expect --selector "#pageinfo" text starts-with "Page 1 of"
  expect no errors

scenario "A wrong password stays on the sign-in page"
  goto /login
  fill --label Email ${email}
  fill --label Password wrong
  click --role button --name "Sign in"
  expect --role alert text contains "Wrong email or password"
  expect title contains "Sign in"

scenario "The returns FAQ opens"
  goto /docs
  click --text "Returns FAQ"
  expect text "days from delivery"
  expect --selector "#returns-faq" attr open is ""
```

## `data.bt`

```text
# Data rows, flows from another file, optional steps, page-state checks.
use flows/account.bt

scenario "Searching for ${term} shows only ${what}" @search
  examples
    | term  | what   |
    | valve | Valve  |
    | hinge | Hinge  |
  goto /shop
  run accept-cookies
  fill --label "Search products" ${term}
  press Enter
  expect --role status text contains "Showing"
  expect js "[...document.querySelectorAll('.card h3')].every((h) => h.textContent.includes('${what}'))"

scenario "A signed-in cart is kept by the server"
  run sign-in demo@acme.test ${PASSWORD:-hunter2}
  expect cookie sid
  goto /shop
  run accept-cookies
  click --text "Add to cart"
  expect request POST /api/cart status 200
  expect --selector "#cart-count" text is 1
  expect no request GET /api/tracking*
```

## `flows/account.bt`

```text
# Flows other files `use`.

flow sign-in(email, password)
  goto /login
  fill --label Email ${email}
  fill --label Password ${password}
  click --role button --name "Sign in"
  expect url ends-with /account

flow accept-cookies
  optional click --selector "#cookie-accept" --timeout-ms 500
```
