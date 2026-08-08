# koto-calc Documentation

koto-calc is an interactive calculator and REPL built on the Koto language,
extended with an algebraic types module (algebraeon) for exact arithmetic.

## Sections

- [About koto-calc](./about.md) — Project overview, features, and current state
- [Language Guide](./language_guide.md) — The Koto language
- [Core Library](./core_lib/) — Standard library modules (io, iterator, koto,
  list, map, number, os, range, string, test, tuple)
- [Library Modules](./libs/) — Extra modules: algebraeon, color, geometry,
  json, random, regex, tempfile, toml, yaml
- [CLI Reference](./cli.md) — Command-line interface reference

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
