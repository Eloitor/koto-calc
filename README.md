# koto-calc

An interactive calculator and REPL built in Rust on top of the
[Koto](https://github.com/koto-lang/koto) language, with an
[Algebraeon](https://crates.io/crates/algebraeon) module for exact algebra:
`NN` (natural numbers), `ZZ` (integers), `Q` (rationals), `Poly`
(polynomials), `Mat` (matrices), `Quat` (Hamilton quaternions), `Alg` (real
algebraic numbers), `ComplexAlg` (complex algebraic numbers), `Perm`/`Group`
(group theory), `FF` (finite fields), `CF` (continued fractions), and more.

## Installation

```bash
cargo build --release
# or to install globally:
cargo install --path .
```

## Quick Start

```bash
# Start the REPL
koto_calc

# Run a script
koto_calc examples/fibonacci.koto

# Evaluate an expression directly
koto_calc -e "print (1..10).sum()"

# Run tests embedded in a script
koto_calc -t script.koto
```

## Documentation

Full documentation is available at <https://eloitor.github.io/koto-calc/>.

- [Language Guide](docs/language_guide.md)
- [Core Library Reference](docs/core_lib/)
- [Library Modules](docs/libs/)
- [CLI Reference](docs/cli.md)

## License

MIT

koto-calc is built on the [Koto](https://github.com/koto-lang/koto) language
(also MIT), with the Algebraeon module providing exact arithmetic.
