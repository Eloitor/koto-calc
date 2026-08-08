# koto-calc Documentation

koto-calc is an interactive calculator and REPL built on the Koto language,
extended with Algebraeon for exact arithmetic and symbolic computation.

## Algebraeon: exact algebra, no rounding

> Factor arbitrary-precision integers, keep rational results as fractions,
> isolate exact algebraic roots, and compute with polynomials, matrices,
> finite fields, groups, quaternions, and more. Start with the
> **[Algebraeon Overview](./algebraeon.md)**, then use the
> **[Full Reference](./libs/algebraeon.md)** for the complete API.

## Sections

- [About koto-calc](./about.md) — Project overview, features, and current state
- [CLI Reference](./cli.md) — Command-line interface reference
- [Language Guide](./language_guide.md) — The Koto language
- **Algebraeon — Exact Algebra**
  - [Overview](./algebraeon.md) — Why exact algebra matters and a quick tour
  - [Full Reference](./libs/algebraeon.md) — Detailed API and validated examples
- [Core Library](./core_lib/) — Standard library modules (io, iterator, koto,
  list, map, number, os, range, string, test, tuple)
- [Library Modules](./libs/) — Extra modules: color, geometry, json, random,
  regex, tempfile, toml, yaml

## Code Examples

### `print!` and `check!`

The code examples in the docs make use of `print!` and `check!` placeholders
used by preprocessor tools:
- Scripts like `scripts/doccheck.py` validate that the code examples work
  correctly by checking the example's output against expectations defined by
  the `check!` commands.
- The [CLI's help command](../src/help.rs) replaces the `check!`
  commands with comments showing the expected output.

### `skip_check` and `skip_run`

Code examples tagged with `skip_run` will be checked to ensure that they can be
compiled, but won't be executed.

`skip_check` will check that the script can be compiled and executed,
but the script's output won't be validated.
