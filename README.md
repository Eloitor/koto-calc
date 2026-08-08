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

## Algebraeon: Exact Algebra

Algebraeon is what turns koto-calc into more than a general-purpose REPL. It
provides arbitrary-precision integers and rationals, exact algebraic numbers,
polynomials, matrices, finite fields, permutations and groups, quaternions,
continued fractions, and more — without hidden floating-point rounding.

```koto
from algebraeon import Q, Poly

print Q(1, 3) + Q(1, 6)        # 1/2
print Poly([6, -5, 1]).factor() # [(-2 + x, 1), (-3 + x, 1)]
```

Explore the
[Algebraeon exact-algebra toolbox](https://eloitor.github.io/koto-calc/algebraeon.html).

## Documentation

Full documentation is available at <https://eloitor.github.io/koto-calc/>.

- [Algebraeon Overview](docs/algebraeon.md)
- [Algebraeon Full Reference](docs/libs/algebraeon.md)
- [Language Guide](docs/language_guide.md)
- [Core Library Reference](docs/core_lib/)
- [Library Modules](docs/libs/)
- [CLI Reference](docs/cli.md)

## License

MIT

koto-calc is built on the [Koto](https://github.com/koto-lang/koto) language
(also MIT), with the Algebraeon module providing exact arithmetic.
