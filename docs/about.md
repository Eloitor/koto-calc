# About koto-calc

koto-calc is an interactive calculator with a REPL, built in Rust. It runs the
[Koto](https://koto.dev) language and extends it with an
[Algebraeon](https://crates.io/crates/algebraeon) module for exact arithmetic
and symbolic computation.

```koto
print 'Hello, World!'

square = |n| n * n
print! '8 squared is {square 8}'
check! 8 squared is 64
```

koto-calc is a fork of the Koto CLI, focused on mathematical computing. The
Koto language was created in 2020 as an embeddable scripting language for Rust
applications; koto-calc inherits its syntax and runtime while adding algebraic
types.

## Features

- **Koto language** — Simple, expressive syntax with fast compilation and a
  rich iterator model. See the [language guide](./language_guide.md) for
  details.
- **Exact algebra** — Arbitrary-precision integers, rationals, algebraic
  numbers, polynomials, matrices, quaternions, finite fields, group theory,
  and more via the [algebraeon module](./libs/algebraeon.md).
- **REPL** — Interactive read-eval-print loop with syntax highlighting,
  tab completion, and the `help` command for built-in documentation.
- **Tests in the language** — First-class test support: `@test` functions,
  `assert` statements, and the `--tests` / `--import_tests` flags.
- **Scripting** — Run `.koto` scripts with arguments accessible via `os.args`.

## Current State

koto-calc is in early development. The Koto language itself is maturing but
has not yet reached 1.0. Early adopter feedback is welcome.

## Attribution

koto-calc is built on [Koto](https://github.com/koto-lang/koto) (MIT license),
originally designed as an extension language for Rust applications. The core
language, runtime, standard library modules, and library modules (color,
geometry, json, random, regex, tempfile, toml, yaml) are derived from Koto.

The algebraeon module and the calculator orientation are koto-calc additions.
