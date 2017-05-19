# ca

`ca` is an arbitrary precision calculator. It can do simple linear math with fully accurate results, and exposes this with a simple REPL.

## Features

* Basic operators (+ - * / % ^ =)
* No loss of precision, accurate representation of repeating decimals
* Implicit multiplication (adjacent products) with incorrect precedence
* Variables and variable binding (`a := 2b`, `b := .5`)
* Partial evaluation, leaving unknowns and recursive definitions in place
* Partial Unicode glyph support
* Function application with or without parens (floor 3.5)
* An incomplete set of functions (floor, ceil, round, trunc, fract, abs)
* Half-baked functionality: sets (like `(1,2,3)` or `()`)

## Guide

To try `ca`, just clone the repository and run `cargo run` with Rust stable:

```
$ git clone https://github.com/portify/ca
$ cd ca
$ cargo run
```

## Contributing

Please and thanks.
