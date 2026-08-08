# Algebraeon — Exact Algebra

Algebraeon is koto-calc's exact-algebra toolkit. It keeps mathematical values
as integers, reduced fractions, algebraic roots, and symbolic structures rather
than silently rounding them to floating-point approximations. That means you
can factor large integers, compare algebraic numbers, invert rational matrices,
and calculate in finite fields without losing information.

> **Start here, then go deeper:** the [Full Reference](./libs/algebraeon.md)
> documents every available constructor, method, and module-level function.

Import the types you need from the built-in `algebraeon` module:

```koto
from algebraeon import Q, Poly, Mat, Alg
```

## The exact-algebra toolbox

| Type or family | What it does |
|---|---|
| `NN` | Arbitrary-precision natural numbers, with primes, factorization, divisors, and combinatorics. |
| `ZZ` | Arbitrary-precision signed integers and integer number theory. |
| `Q` | Reduced rational numbers, so values such as `1/3` stay exact. |
| `QSqrt` | Elements of quadratic fields `Q(sqrt(d))`, including exact conjugates, norms, and inverses. |
| `Alg` | Exact real algebraic numbers represented as isolated roots of polynomials. |
| `ComplexAlg` | Exact complex algebraic numbers, including polynomial roots and the imaginary unit. |
| `CF` | Finite and periodic continued fractions, convergents, and exact rational values. |
| `FF` | Prime and extension finite fields `GF(p^k)` with exact field arithmetic. |
| `Ideal` / `ZZn` | Ideals of the integers and residue rings such as `Z/12Z`. |
| `Poly` | Univariate polynomials over `ZZ` or `Q`, with evaluation, gcd, derivatives, and factorization. |
| `MultiPoly` | Symbolic multivariate integer polynomials with evaluation and symmetric-polynomial tools. |
| `PolyQuot` | Exact number fields presented as quotient rings `Q[x]/(f)`. |
| `Mat` | Integer and rational matrices with determinants, exact inverses, and LLL reduction. |
| `Perm` | Permutations with composition, inverses, signs, and cycle decomposition. |
| `Group` | Finite groups represented by multiplication tables, with standard constructors. |
| `Quat` | Hamilton quaternions over the rationals, with conjugate, norm, and inverse. |
| Stirling numbers | Exact first- and second-kind Stirling numbers via `NN` and `ZZ`. |

## A quick tour

Each example below is ready to paste into a koto-calc script or REPL.

### Fractions stay fractions

```koto
from algebraeon import Q

third = Q(1, 3)
print third + Q(1, 6) # 1/2
```

There is no intermediate binary floating-point value: the result is the
reduced fraction `1/2`.

### Factor arbitrary-precision integers

```koto
from algebraeon import NN

print NN(12345).factor() # [(3, 1), (5, 1), (823, 1)]
```

### Work with an exact square root

```koto
from algebraeon import Alg, Q

sqrt2 = Alg([-2, 0, 1])[1]
print sqrt2.min_poly() # -2 + x^2
print sqrt2 > Q(7, 5) # true
```

`sqrt2` is stored as an isolated root of `x^2 - 2`. Its usual display,
`1.414213562`, is only a readable approximation; the minimal polynomial and
comparisons remain exact.

### Factor polynomials

Coefficients are listed from the constant term upwards, so the polynomial
below is `6 - 5x + x^2`.

```koto
from algebraeon import Poly

f = Poly([6, -5, 1])
print f.factor() # [(-2 + x, 1), (-3 + x, 1)]
```

### Invert a matrix without rounding

```koto
from algebraeon import Mat

m = Mat([[1, 2], [3, 4]])
print m.inverse()     # [[-2, 1], [3/2, -1/2]]
print m.inverse() * m # [[1, 0], [0, 1]]
```

### Explore a finite group

```koto
from algebraeon import Group

c4 = Group.cyclic(4)
print c4          # C4 (size 4)
print c4.order(1) # 4
```

`Group` also constructs dihedral, symmetric, alternating, Klein four, and
quaternion groups, all backed by finite multiplication tables.

### Calculate in a finite field

```koto
from algebraeon import FF

gf7 = FF(7)
x = gf7.of(3)
print x.inverse()     # 5
print x * x.inverse() # 1
```

Prime fields are only the beginning: `FF(p, k)` constructs extension fields
using Algebraeon's Conway-polynomial database.

## Where to go next

The [Algebraeon Full Reference](./libs/algebraeon.md) contains detailed type
signatures and validated examples for the complete API. You can also read
about the underlying Rust library on the
[Algebraeon crate page](https://crates.io/crates/algebraeon).
