# algebraeon

[Algebraeon](https://crates.io/crates/algebraeon) support for Koto:
arbitrary precision arithmetic, number theory, polynomials, and matrices.

The module provides the types [`N`](#n) (natural numbers),
[`Z`](#z) (integers), [`Q`](#q) (rationals),
[`Poly`](#poly) (univariate polynomials), [`Mat`](#mat) (matrices),
[`Quat`](#quat) (Hamilton quaternions) and [`Alg`](#alg) (real algebraic
numbers), plus the module-level functions [`gcd`](#gcd) and [`lcm`](#lcm).

## Naming convention

Basic number domains use ASCII forms of their mathematical symbols: `N`
(naturals), `Z` (integers), and `Q` (rationals). `Zn(n)` is the residue ring
ℤ/nℤ. Other structures use PascalCase names or established acronyms, for
example `Poly`, `Mat`, `ComplexAlg`, `FF`, and `CF`.

The former constructors `NN`, `ZZ`, and `ZZn` remain compatibility aliases
during the 0.2 transition. They construct the same canonical `N`, `Z`, and
`Zn` runtime types.

## N

```kototype
|| -> Iterator
|Number| -> N
```

Natural (non-negative integer) values with arbitrary precision.

Called with no arguments, `N()` returns an iterator over the natural numbers
`0, 1, 2, ...`.

### Example

```koto
print! N(5).factorial()
check! 120

print! N(5) - N(3)
check! 2

print! N().take(4).to_list()
check! [0, 1, 2, 3]
```

## N.bitcount

```kototype
|N| -> Number
```

Returns the number of bits needed to represent the value.

### Example

```koto
print! N(5).bitcount()
check! 3
```

## N.is_prime

```kototype
|N| -> Bool
```

Returns `true` if the value is prime.

### Example

```koto
print! N(17).is_prime()
check! true

print! N(12).is_prime()
check! false
```

## N.is_squarefree

```kototype
|N| -> Bool
```

Returns `true` if the value has no repeated prime factors.

### Example

```koto
print! N(10).is_squarefree()
check! true

print! N(12).is_squarefree()
check! false
```

## N.factor

```kototype
|N| -> [(N, N)]
```

Returns the prime factorization of the value as a list of
`(prime, exponent)` tuples.

### Example

```koto
print! N(60).factor()
check! [(2, 2), (3, 1), (5, 1)]
```

## N.factorial

```kototype
|N| -> N
```

Returns the factorial of the value.

### Example

```koto
print! N(5).factorial()
check! 120
```

## N.divisors

```kototype
|N| -> [N]
```

Returns the value's divisors in ascending order.

### Example

```koto
print! N(12).divisors()
check! [1, 2, 3, 4, 6, 12]
```

## N.euler_totient

```kototype
|N| -> N
```

Returns the value of [Euler's totient function](https://en.wikipedia.org/wiki/Euler%27s_totient_function),
the count of positive integers up to the value that are coprime to it.

### Example

```koto
print! N(10).euler_totient()
check! 4
```

## N.is_square

```kototype
|N| -> Bool
```

Returns `true` if the value is a perfect square.

### Example

```koto
print! N(16).is_square()
check! true

print! N(18).is_square()
check! false
```

## N.sqrt_floor

```kototype
|N| -> N
```

Returns the floor of the square root of the value.

### Example

```koto
print! N(17).sqrt_floor()
check! 4
```

## N.sqrt_ceil

```kototype
|N| -> N
```

Returns the ceiling of the square root of the value.

### Example

```koto
print! N(17).sqrt_ceil()
check! 5
```

## N.is_power_test

```kototype
|N| -> (Bool, N?, N?)
```

Returns `(true, base, exponent)` if the value can be written as `base^exponent`
with `exponent > 1`, otherwise `(false, null, null)`.

### Example

```koto
print! N(8).is_power_test()
check! (true, 2, 3)

print! N(6).is_power_test()
check! (false, null, null)
```

## N.primality_test

```kototype
|N| -> String
```

Returns `'prime'` or `'composite'` (both `0` and `1` are reported as
`'composite'`).

### Example

```koto
print! N(17).primality_test()
check! prime

print! N(12).primality_test()
check! composite
```

## N.primes

```kototype
|| -> Iterator
```

Returns an iterator over the prime numbers.

### Example

```koto
print! N.primes().take(6).to_list()
check! [2, 3, 5, 7, 11, 13]
```

## Z

```kototype
|Number| -> Z
```

Integer values with arbitrary precision.

`Z` supports arithmetic (`+ - *`), comparisons, and assignment operators
(`+= -= *=`) with other `Z` values, `N` values, and plain numbers.

### Example

```koto
print! Z(5) + Z(-3)
check! 2

print! Z(4) * Z(-2)
check! -8

print! Z(5) + N(3)
check! 8
```

## Z.abs

```kototype
|Z| -> N
```

Returns the absolute value of the integer as an `N`.

### Example

```koto
print! Z(-9).abs()
check! 9
```

## Z.is_irreducible

```kototype
|Z| -> Bool
```

Returns `true` if the value is irreducible (i.e. a prime, up to sign).

### Example

```koto
print! Z(7).is_irreducible()
check! true

print! Z(9).is_irreducible()
check! false
```

## Z.is_square

```kototype
|Z| -> Bool
```

Returns `true` if the value is a perfect square.

### Example

```koto
print! Z(9).is_square()
check! true

print! Z(10).is_square()
check! false
```

## Z.factor

```kototype
|Z| -> [(Z, N)]
```

Returns the prime factorization of the value as a list of
`(prime, exponent)` tuples. The sign is ignored.

### Example

```koto
print! Z(-12).factor()
check! [(2, 2), (3, 1)]
```

## Z.divmod

```kototype
|Z, Z| -> (Z, Z)
```

Returns the quotient and remainder of a floor division, with a
non-negative remainder.

### Example

```koto
print! Z(-7).divmod(Z(3))
check! (-3, 2)
```

## Z.div_floor

```kototype
|Z, Z| -> Z
```

Returns the quotient of a floor division.

### Example

```koto
print! Z(-7).div_floor(Z(3))
check! -3

print! Z(-13).div_floor(Z(5))
check! -3
```

## Z.mod

```kototype
|Z, Z| -> Z
```

Returns the non-negative remainder of a floor division, coherent with
[`div_floor`](#z-div-floor).

### Example

```koto
print! Z(-7).mod(Z(3))
check! 2

print! Z(-13).mod(Z(5))
check! 2
```

## Q

```kototype
|Number| -> Q
|Number, Number| -> Q
```

Rational numbers, stored as reduced fractions `num / den`.

The denominator must be non-zero. A single argument is treated as a whole
number. `Q` supports arithmetic (`+ - * /`), comparisons, and assignment
operators (`+= -= *= /=`) with other `Q` values, `N` values, `Z` values and
plain numbers.

The display form is the reduced fraction `num/den`, or just `num` when the
denominator is `1`.

### Example

```koto
print! Q(6, 4)
check! 3/2

print! Q(1, 2) + Q(1, 3)
check! 5/6

print! Q(1, 2) / Q(3, 4)
check! 2/3

print! Q(0.5)
check! 1/2

print! Q(3)
check! 3
```

## Q.num

```kototype
|Q| -> Z
```

Returns the numerator of the reduced fraction.

### Example

```koto
print! Q(3, 2).num()
check! 3

print! Q(-3, 2).num()
check! -3
```

## Q.den

```kototype
|Q| -> N
```

Returns the denominator of the reduced fraction.

### Example

```koto
print! Q(3, 2).den()
check! 2
```

## Q.is_integer

```kototype
|Q| -> Bool
```

Returns `true` if the value is a whole number.

### Example

```koto
print! Q(4, 2).is_integer()
check! true

print! Q(3, 2).is_integer()
check! false
```

## Q.is_square

```kototype
|Q| -> Bool
```

Returns `true` if the value is a perfect square.

### Example

```koto
print! Q(4, 9).is_square()
check! true

print! Q(2, 3).is_square()
check! false
```

## Q.sqrt_if_square

```kototype
|Q| -> Q?
```

Returns the square root of the value if it is a perfect square,
otherwise `null`.

### Example

```koto
print! Q(4, 9).sqrt_if_square()
check! 2/3

print! Q(2, 3).sqrt_if_square()
check! null
```

## Q.height

```kototype
|Q| -> N
```

Returns the height of the value: `max(|num|, den)` of the reduced fraction.

### Example

```koto
print! Q(3, 2).height()
check! 3

print! Q(-2, 5).height()
check! 5
```

## Q.to_float

```kototype
|Q| -> Number
```

Converts the value to a floating point number.

### Example

```koto
print! Q(3, 2).to_float()
check! 1.5
```

## Q.to_zz

```kototype
|Q| -> Z
```

Converts the value to a `Z` (the value must be a whole number).

### Example

```koto
print! Q(3).to_zz()
check! 3

print! Q(4, 2).to_zz()
check! 2
```

## Q.to_nn

```kototype
|Q| -> N
```

Converts the value to an `N` (the value must be a non-negative whole number).

### Example

```koto
print! Q(4, 2).to_nn()
check! 2

print! Q(0).to_nn()
check! 0
```

## Poly

```kototype
|List| -> Poly
```

Univariate polynomials over `Z` or `Q`.

The constructor takes a list of coefficients in ascending order, with the
first element being the constant term: `Poly([6, -5, 1])` represents
`6 - 5x + x^2`.

The coefficients are stored as `Z` when all of them are integers, and
promoted to `Q` when any of them is a fraction. Arithmetic (`+ - *`) works
with other polynomials and with `N`/`Z`/`Q` scalars, promoting `Z` to `Q`
when needed.

The display form shows the terms in ascending order of degree, e.g.
`6 - 5x + x^2`.

### Example

```koto
a = Poly([6, -5, 1])
print! a
check! 6 - 5x + x^2

print! a + Poly([1, 1])
check! 7 - 4x + x^2

print! a * Q(1, 2)
check! 3 - (5/2)x + (1/2)x^2
```

## Poly.degree

```kototype
|Poly| -> N
```

Returns the degree of the polynomial (the zero polynomial has degree `0`).

### Example

```koto
print! Poly([6, -5, 1]).degree()
check! 2

print! Poly([7]).degree()
check! 0
```

## Poly.coeffs

```kototype
|Poly| -> [Z] | [Q]
```

Returns the coefficients in ascending order, starting with the constant term.

### Example

```koto
print! Poly([6, -5, 1]).coeffs()
check! [6, -5, 1]

print! Poly([3, Q(-5, 2), Q(1, 2)]).coeffs()
check! [3, -5/2, 1/2]
```

## Poly.eval

```kototype
|Poly, x: Number| -> Z | Q
```

Evaluates the polynomial at `x` (which may be a `Number`, `N`, `Z` or `Q`).

### Example

```koto
a = Poly([6, -5, 1])
print! a.eval(2)
check! 0

print! a.eval(Q(1, 2))
check! 15/4
```

## Poly.derivative

```kototype
|Poly| -> Poly
```

Returns the derivative of the polynomial.

### Example

```koto
print! Poly([6, -5, 1]).derivative()
check! -5 + 2x

print! Poly([5]).derivative()
check! 0
```

## Poly.gcd

```kototype
|Poly, Poly| -> Poly
```

Returns the monic greatest common divisor of two polynomials,
promoting to `Q` if needed.

### Example

```koto
print! Poly([6, -5, 1]).gcd(Poly([2, -3, 1]))
check! -2 + x
```

## Poly.factor

```kototype
|Poly| -> [(Poly, N)]
```

Returns the irreducible factorization of the polynomial as a list of
`(factor, exponent)` tuples.

### Example

```koto
print! Poly([6, -5, 1]).factor()
check! [(-2 + x, 1), (-3 + x, 1)]

print! Poly([1, 0, 1]).factor()
check! [(1 + x^2, 1)]
```

## Mat

```kototype
|List of lists| -> Mat
```

Matrices over `Z` or `Q`, given row by row: `Mat([[1, 2], [3, 4]])` is the
`2x2` matrix with rows `[1, 2]` and `[3, 4]`.

The entries are stored as `Z` when all of them are integers, and promoted
to `Q` when any of them is a fraction. Arithmetic (`+ - *`) works with other
matrices and with `N`/`Z`/`Q` scalars.

The display form is a list of rows, e.g. `[[1, 2], [3, 4]]`.

### Example

```koto
m = Mat([[1, 2], [3, 4]])
print! m
check! [[1, 2], [3, 4]]

print! m * Mat([[5, 6], [7, 8]])
check! [[19, 22], [43, 50]]

print! m.det()
check! -2
```

## Mat.rows

```kototype
|Mat| -> N
```

Returns the number of rows.

### Example

```koto
print! Mat([[1, 2], [3, 4]]).rows()
check! 2
```

## Mat.cols

```kototype
|Mat| -> N
```

Returns the number of columns.

### Example

```koto
print! Mat([[1, 2], [3, 4]]).cols()
check! 2
```

## Mat.at

```kototype
|Mat, row: Number, col: Number| -> Z | Q
```

Returns the entry at the given row and column (zero-based).

### Example

```koto
print! Mat([[1, 2], [3, 4]]).at(1, 0)
check! 3
```

## Mat.transpose

```kototype
|Mat| -> Mat
```

Returns the transposed matrix.

### Example

```koto
print! Mat([[1, 2], [3, 4]]).transpose()
check! [[1, 3], [2, 4]]
```

## Mat.mul

```kototype
|Mat, Mat| -> Mat
```

Returns the matrix product (also available as the `*` operator).

### Example

```koto
m = Mat([[1, 2], [3, 4]])
print! m.mul(Mat([[5, 6], [7, 8]]))
check! [[19, 22], [43, 50]]
```

## Mat.det

```kototype
|Mat| -> Z | Q
```

Returns the determinant (only defined for square matrices).

### Example

```koto
print! Mat([[1, 2], [3, 4]]).det()
check! -2
```

## Mat.inverse

```kototype
|Mat| -> Mat
```

Returns the inverse of the matrix over `Q` (a `Z` matrix is promoted to
`Q`). An error is thrown if the matrix is singular.

### Example

```koto
m = Mat([[1, 2], [3, 4]])
print! m.inverse()
check! [[-2, 1], [3/2, -1/2]]

print! m.inverse() * m
check! [[1, 0], [0, 1]]
```

## Mat.lll

```kototype
|Mat| -> Mat
```

Returns the [LLL-reduced](https://en.wikipedia.org/wiki/Lenstra%E2%80%93Lenstra%E2%80%93Lov%C3%A1sz_lattice_basis_reduction_algorithm)
basis of the lattice generated by the rows of an integer matrix.
An error is thrown if the matrix contains fractions.

### Example

```koto
print! Mat([[1, 1], [1, 2]]).lll()
check! [[-1, 0], [0, 1]]
```

## gcd

```kototype
|N, N| -> N
```

Returns the greatest common divisor of two natural numbers.

### Example

```koto
print! gcd(N(12), N(18))
check! 6
```

## lcm

```kototype
|N, N| -> N
```

Returns the least common multiple of two natural numbers.

### Example

```koto
print! lcm(N(4), N(6))
check! 12
```

## Quat

```kototype
|Number, Number, Number, Number| -> Quat
```

[Hamilton quaternions](https://en.wikipedia.org/wiki/Quaternion) over `Q`,
constructed from four coefficients `a + bi + cj + dk` (each
`Number`/`N`/`Z`/`Q` argument is promoted to `Q`).

Multiplication is the Hamilton product, defined by
`i^2 = j^2 = k^2 = ijk = -1` with `i*j = k`, `j*k = i` and `k*i = j` (the
cross terms anti-commute: `j*i = -k`, ...), so multiplication is not
commutative. The product is computed directly on the coefficients: the
wrapper works around a bug in algebraeon 0.0.17 (upstream issue #244) that
produced wrong signs in the `i`/`j` cross terms of
`QuaternionAlgebraStructure::mul`.

`Quat` supports arithmetic (`+ - *`) with other `Quat` values and with
scalars (`Number`/`N`/`Z`/`Q`, on either side), negation, and equality
(`==`, `!=`).

The display form is e.g. `1 + 2i - 3j + (1/2)k`: zero terms are omitted,
the coefficient `1` is dropped on `i`/`j`/`k`, and fractional coefficients
are parenthesized.

### Example

```koto
q = Quat(1, 2, 3, 4)
print! q
check! 1 + 2i + 3j + 4k

print! q + Quat(1, -2, -3, -4)
check! 2

print! q * 2
check! 2 + 4i + 6j + 8k

print! 1 - q
check! -2i - 3j - 4k

print! Quat(1, 2, 0, 0) * Quat(3, 4, 0, 0)
check! -5 + 10i

# Hamilton rules: i*j = k, j*i = -k, i*i = -1
i = Quat(0, 1, 0, 0)
j = Quat(0, 0, 1, 0)
k = Quat(0, 0, 0, 1)
print! i * j
check! k

print! j * i
check! -k

print! i * i
check! -1

# Associativity: (i*j)*k = -1
print! (i * j) * k
check! -1
```

## Quat.conjugate

```kototype
|Quat| -> Quat
```

Returns the conjugate `a - bi - cj - dk`.

### Example

```koto
print! Quat(1, 2, 3, 4).conjugate()
check! 1 - 2i - 3j - 4k

print! Quat(1, 2, 3, 4).conjugate().conjugate()
check! 1 + 2i + 3j + 4k
```

## Quat.norm

```kototype
|Quat| -> Q
```

Returns the reduced norm `a^2 + b^2 + c^2 + d^2`.

### Example

```koto
print! Quat(1, 2, 3, 4).norm()
check! 30

print! Quat(3, -4, 0, 0).norm()
check! 25
```

## Quat.trace

```kototype
|Quat| -> Q
```

Returns the reduced trace `2a`.

### Example

```koto
print! Quat(1, 2, 3, 4).trace()
check! 2

print! Quat(3, -4, 0, 0).trace()
check! 6
```

## Quat.coeffs

```kototype
|Quat| -> (Q, Q, Q, Q)
```

Returns the four coefficients as a tuple `(a, b, c, d)`.

### Example

```koto
print! Quat(1, 2, 3, 4).coeffs()
check! (1, 2, 3, 4)

print! Quat(Q(1, 2), 0, 0, 0).coeffs()
check! (1/2, 0, 0, 0)
```

## Quat.to_float

```kototype
|Quat| -> (Number, Number, Number, Number)
```

Converts the four coefficients to floating point numbers.

### Example

```koto
print! Quat(1, 2, 3, 4).to_float()
check! (1.0, 2.0, 3.0, 4.0)
```

## Alg

```kototype
|Poly | List| -> [Alg]
```

Real algebraic numbers: exact real roots of polynomials. The constructor
takes a `Poly` (over `Z` or `Q`) or a coefficient list (as in
`Poly([...])`), and returns the **list of isolated real roots in increasing
order**, with multiplicity. Polynomials of degree `0` (including the zero
polynomial) and polynomials without real roots give an empty list.

Each `Alg` value is a root with an isolating interval, so comparisons are
exact: `<`, `<=`, `>`, `>=` and `==` work between two `Alg` values and
between an `Alg` and a scalar (`Q`/`N`/`Z`/`Number`, compared exactly as
a rational). Arithmetic between algebraic numbers is not exposed.

The display form is a decimal approximation with 9 significant decimals
(e.g. `1.414213562`), or the exact reduced fraction for rational values
(e.g. `6`).

### Example

```koto
roots = Alg(Poly([-2, 0, 1]))  # roots of x^2 - 2
print! roots
check! [-1.414213562, 1.414213562]

print! size(roots)
check! 2

print! roots[0] < roots[1]
check! true

print! Alg(Poly([1, -2, 1]))  # (x - 1)^2, multiplicity kept
check! [1, 1]

print! Alg(Poly([1, 0, 1]))  # x^2 + 1 has no real roots
check! []

print! Alg([-2, 0, 1])  # coefficient list form
check! [-1.414213562, 1.414213562]
```

## Alg.cmp

```kototype
|Alg, Alg | Number| -> Number
```

Exact comparison: `-1` if smaller, `0` if equal, `1` if greater. The
argument may be another `Alg` or a scalar (`Number`/`N`/`Z`/`Q`),
compared exactly as a rational.

### Example

```koto
roots = Alg(Poly([-2, 0, 1]))
print! roots[0].cmp(roots[1])
check! -1

sqrt2 = roots[1]
print! sqrt2.cmp(Q(141, 100))  # sqrt(2) > 141/100
check! 1
```

## Alg.accuracy

```kototype
|Alg| -> Q
```

Returns the width of the isolating interval (an exact rational). Rational
values have accuracy `0`.

### Example

```koto
print! Alg(Poly([-2, 0, 1]))[0].accuracy() > Q(0)
check! true

print! Alg(Poly([-6, 1]))[0].accuracy()  # rational root
check! 0
```

## Alg.refine

```kototype
|Alg, accuracy: Q | Number| -> Alg
```

Returns a new `Alg` whose isolating interval has been refined to the
requested (positive) accuracy. Rational values are returned unchanged.

### Example

```koto
sqrt2 = Alg(Poly([-2, 0, 1]))[1]
r = sqrt2.refine(Q(1, 1000))
print! r.accuracy() < Q(1, 1000)
check! true

print! r.cmp(sqrt2)
check! 0
```

## Alg.min_poly

```kototype
|Alg| -> Poly
```

Returns the minimal polynomial of the algebraic number (a `Poly` over `Q`).
For a rational value `n/d` it is `d*x - n`.

### Example

```koto
sqrt2 = Alg(Poly([-2, 0, 1]))[1]
print! sqrt2.min_poly()
check! -2 + x^2

print! Alg(Poly([-6, 1]))[0].min_poly()
check! -6 + x
```

## Alg.to_float

```kototype
|Alg| -> Number
```

Returns a floating point approximation; the isolating interval is refined
to accuracy `10^-15` before converting the midpoint.

### Example

```koto
sqrt2 = Alg(Poly([-2, 0, 1]))[1]
print! sqrt2.to_float()
check! 1.4142135623730951

print! Alg(Poly([-6, 1]))[0].to_float()
check! 6.0
```

## Ideal

```kototype
|Number, ...| -> Ideal
```

Ideals of `Z` are principal. `Ideal(a, b, ...)` is the ideal generated by
its integer arguments; its canonical non-negative generator is displayed with
suffix `Z`. Thus `Ideal(4, 6)` is `2Z`.

### Example

```koto
print! Ideal(4, 6)
check! 2Z

print! Ideal(6).generator()
check! 6

print! Ideal(6).contains(12)
check! true

print! Ideal(6).contains(5)
check! false
```

## Ideal.contains

```kototype
|Ideal, Number| -> Bool
```

Tests whether an integer belongs to the ideal. `N` and `Z` values are also
accepted.

### Example

```koto
print! Ideal(6).contains(Z(-12))
check! true

print! Ideal(0).contains(3)
check! false
```

## Ideal.generator

```kototype
|Ideal| -> N
```

Returns the canonical non-negative generator.

### Example

```koto
print! Ideal(-4, 6).generator()
check! 2
```

## Ideal.sum

```kototype
|Ideal, Ideal| -> Ideal
```

Returns the sum of two ideals. In `Z`, this is the ideal generated by the
greatest common divisor of their generators.

### Example

```koto
print! Ideal(6).sum(Ideal(15))
check! 3Z
```

## Ideal.intersect

```kototype
|Ideal, Ideal| -> Ideal
```

Returns the intersection of two ideals. In `Z`, its generator is the least
common multiple of the two generators.

### Example

```koto
print! Ideal(6).intersect(Ideal(15))
check! 30Z
```

## Ideal.product

```kototype
|Ideal, Ideal| -> Ideal
```

Returns the product of two ideals.

### Example

```koto
print! Ideal(6).product(Ideal(15))
check! 90Z
```

## Ideal.quotient

```kototype
|Ideal, Ideal| -> Ideal
```

Returns the ideal quotient `(I : J) = {x in Z : xJ subset I}`.

### Example

```koto
print! Ideal(6).quotient(Ideal(2))
check! 3Z
```

## Ideal.equals

```kototype
|Ideal, Ideal| -> Bool
```

Tests equality of ideals by comparing their canonical generators. The `==`
operator can be used as well.

### Example

```koto
print! Ideal(4, 6).equals(Ideal(2))
check! true

print! Ideal(6) == Ideal(-6)
check! true
```

## Zn

```kototype
|Number| -> Zn
```

The ring `Zn(n)` is ℤ/nℤ, the ring of integers modulo a positive modulus `n`.
Call `.of(x)` to create the residue class of an integer. Classes from the same
ring support `+`, `-`, `*` and unary `-`; their display is `[x] mod n` with a
canonical representative.

### Example

```koto
print! Zn(6)
check! Zn

print! Zn(6).of(7) + Zn(6).of(5)
check! [0] mod 6

print! -Zn(6).of(1)
check! [5] mod 6
```

## Zn.of

```kototype
|Zn, Number| -> ZnElement
```

Creates a residue class, reducing the argument modulo the ring modulus.
`N` and `Z` values are accepted too.

### Example

```koto
print! Zn(7).of(-1)
check! [6] mod 7

print! Zn(6).of(7) * Zn(6).of(5)
check! [5] mod 6
```

## ZnElement.inverse

```kototype
|ZnElement| -> ZnElement
```

Returns a multiplicative inverse. It errors when the residue is not coprime
to the modulus.

### Example

```koto
print! Zn(7).of(5).inverse()
check! [3] mod 7
```

## FF

```kototype
|Number| -> FF
|Number, Number| -> FF
```

Finite fields are written `GF(p)` or `GF(p^k)`, where `p` is prime. `FF(p)`
constructs the prime field, while `FF(p, k)` constructs the extension field
using the Conway polynomial from Algebraeon's database. Elements are made
with `.of(x)`.

### Example

```koto
print! FF(7)
check! GF(7)

print! FF(7).char(), FF(7).degree()
check! (7, 1)

print! FF(2, 2)
check! GF(2^2)
```

## FF.of

```kototype
|FF, Number| -> FFElement
|FF, List| -> FFElement
```

For `GF(p)`, `.of(x)` reduces an integer modulo `p`. For `GF(p^k)`, a list
contains coefficients in ascending degree order, `[c0, c1, ...]`, and is
reduced modulo the Conway polynomial.

### Example

```koto
print! FF(7).of(-1)
check! 6

x = FF(2, 2).of([0, 1])
print! x * x
check! x + 1

print! x.coeffs()
check! [0, 1]
```

## FFElement.inverse

```kototype
|FFElement| -> FFElement
```

Returns the multiplicative inverse of a non-zero finite-field element.

### Example

```koto
print! FF(7).of(3).inverse()
check! 5
```

## FFElement.order

```kototype
|FFElement| -> N
```

Returns the multiplicative order of a non-zero element.

### Example

```koto
print! FF(7).of(3).order()
check! 6
```

## FFElement.pow

```kototype
|FFElement, Number| -> FFElement
```

Raises an element to an integer power. Negative powers use the
multiplicative inverse.

### Example

```koto
print! FF(7).of(2).pow(-1)
check! 4

print! FF(2, 2).of([0, 1]).pow(3)
check! 1
```

## CF

```kototype
|List| -> CF
```

A simple continued fraction. `CF([a0, a1, ...])` is finite and coefficients
after the first must be positive. Its display uses `[a0, a1, ...]`.

### Example

```koto
print! CF([3, 7])
check! [3, 7]

print! CF([3, 7]).value()
check! 22/7

print! CF([3, 7]).convergent(1)
check! 22/7
```

## CF.periodic

```kototype
|List, List| -> CF
```

Constructs an infinite periodic continued fraction from its initial and
repeating parts. For example, `[1; 2, 2, ...]` represents the continued
fraction expansion of the square root of 2.

### Example

```koto
s2 = CF.periodic([1], [2])
print! s2
check! [1; 2]

print! s2.convergent(3)
check! 17/12
```

## CF.value

```kototype
|CF| -> Q
```

Returns the exact rational value of a finite continued fraction. It is not
defined for periodic or infinite continued fractions.

### Example

```koto
print! CF([3, 7]).value()
check! 22/7
```

## CF.convergent

```kototype
|CF, N| -> Q
```

Returns the convergent at index `n`, starting at index zero. Convergents also
work for periodic and infinite continued fractions.

### Example

```koto
print! CF.periodic([1], [2]).convergent(4)
check! 41/29
```

## CF.take

```kototype
|CF, N| -> [Z]
```

Returns the first `n` coefficients as a list of `Z` values. A finite
continued fraction stops when its coefficients run out.

### Example

```koto
print! CF([3, 7]).take(4)
check! [3, 7]
```

## CF.to_float

```kototype
|CF| -> Number
```

Returns a floating-point approximation. Finite fractions are evaluated
exactly before conversion; infinite fractions use a convergent.

### Example

```koto
print! CF([3, 7]).to_float()
check! 3.142857142857143
```

## Perm

```kototype
|List| -> Perm
```

A permutation is given by its list of images, indexed from zero. For example,
`Perm([1, 2, 0])` maps `0 -> 1`, `1 -> 2` and `2 -> 0`. Composition `p * q`
applies `q` first and then `p`.

### Example

```koto
p = Perm([1, 2, 0])
q = Perm([0, 2, 1])
print! p
check! [1, 2, 0]

print! p * q
check! [1, 0]

print! p.inverse()
check! [2, 0, 1]
```

## Perm.compose

```kototype
|Perm, Perm| -> Perm
```

Composes two permutations with the same convention as `*`: the argument is
applied first.

### Example

```koto
print! Perm([1, 2, 0]).compose(Perm([0, 2, 1]))
check! [1, 0]
```

## Perm.inverse

```kototype
|Perm| -> Perm
```

Returns the inverse permutation.

### Example

```koto
p = Perm([1, 2, 0])
print! p.compose(p.inverse())
check! []
```

## Perm.sign

```kototype
|Perm| -> Number
```

Returns `1` for an even permutation and `-1` for an odd permutation.

### Example

```koto
print! Perm([0, 2, 1]).sign()
check! -1

print! Perm([1, 2, 0]).sign()
check! 1
```

## Perm.cycles

```kototype
|Perm| -> [[Number]]
```

Returns the disjoint cycles. Fixed points are omitted.

### Example

```koto
print! Perm([2, 0, 1, 4, 3]).cycles()
check! [[0, 2, 1], [3, 4]]
```

## Perm.cycle_shape

```kototype
|Perm| -> [Number]
```

Returns the sorted lengths of the non-trivial disjoint cycles.

### Example

```koto
print! Perm([2, 0, 1, 4, 3]).cycle_shape()
check! [2, 3]
```

## Perm.call

```kototype
|Perm, Number| -> Number
```

Returns the image of an index under the permutation.

### Example

```koto
p = Perm([1, 2, 0])
print! p.call(2)
check! 0
```

## Perm.all

```kototype
|Number| -> [Perm]
```

Returns all permutations in `S_n`.

### Example

```koto
print! size(Perm.all(3))
check! 6
```

## Group

```kototype
|| -> Group
```

`Group` provides finite groups represented by multiplication tables. The
constructors are `cyclic(n)`, `dihedral(n)`, `symmetric(n)`,
`alternating(n)`, `klein4()`, `quaternion()` and `trivial()`.

### Example

```koto
print! Group.cyclic(4)
check! C4 (size 4)

print! Group.symmetric(3)
check! S3 (size 6)
```

## Group.size

```kototype
|Group| -> Number
```

Returns the number of elements in the group.

### Example

```koto
print! Group.dihedral(3).size()
check! 6
```

## Group.order

```kototype
|Group, Number| -> Number
```

Returns the order of the element at the given table index. The identity is
index `0`.

### Example

```koto
print! Group.cyclic(4).order(1)
check! 4

print! Group.cyclic(4).order(2)
check! 2
```

## Group.is_abelian

```kototype
|Group| -> Bool
```

Returns whether the group operation is commutative.

### Example

```koto
print! Group.cyclic(4).is_abelian()
check! true

print! Group.dihedral(3).is_abelian()
check! false
```

## Group.conjugacy_classes

```kototype
|Group| -> [[Number]]
```

Returns the conjugacy classes as sorted lists of element indices.

### Example

```koto
print! Group.dihedral(3).conjugacy_classes()
check! [[0], [1, 2, 4], [3, 5]]
```

## ComplexAlg

```kototype
|Poly | List| -> [ComplexAlg]
|Number| -> ComplexAlg
|Number, Number| -> ComplexAlg
```

Complex algebraic numbers are exact roots of polynomials. With a `Poly` or a
coefficient list, `ComplexAlg(...)` returns all complex roots with
multiplicity. With one scalar it constructs a rational real value; with two
scalars `ComplexAlg(a, b)` constructs `a + b*i`. The imaginary unit is
`ComplexAlg.i()`.

### Example

```koto
i = ComplexAlg.i()
print! i
check! i

print! i * i
check! -1

print! size(ComplexAlg(Poly([1, 0, 1])))
check! 2
```

## ComplexAlg.real

```kototype
|ComplexAlg| -> Alg
```

Returns the exact real part as an `Alg` value.

### Example

```koto
print! ComplexAlg(Q(1), Q(2)).real()
check! 1
```

## ComplexAlg.imag

```kototype
|ComplexAlg| -> Alg
```

Returns the exact imaginary part as an `Alg` value.

### Example

```koto
print! ComplexAlg(Q(1), Q(2)).imag()
check! 2
```

## ComplexAlg.conjugate

```kototype
|ComplexAlg| -> ComplexAlg
```

Returns the complex conjugate.

### Example

```koto
print! ComplexAlg(Q(1), Q(2)).conjugate()
check! 1 - 2i
```

## ComplexAlg.min_poly

```kototype
|ComplexAlg| -> Poly
```

Returns the minimal polynomial over `Q`.

### Example

```koto
print! ComplexAlg(Q(1), Q(2)).min_poly()
check! 5 - 2x + x^2
```

## ComplexAlg.degree

```kototype
|ComplexAlg| -> N
```

Returns the degree of the minimal polynomial.

### Example

```koto
print! ComplexAlg.i().degree()
check! 2
```

## ComplexAlg.to_float

```kototype
|ComplexAlg| -> [Number, Number]
```

Returns a floating-point approximation as `[real, imag]`.

### Example

```koto
print! ComplexAlg(Q(1), Q(2)).to_float()
check! [1.0, 2.0]
```

## legendre

```kototype
|Number, Number| -> Z
```

Returns the Legendre symbol `(a / p)` as `-1`, `0` or `1`. The bottom
argument must be an odd prime.

### Example

```koto
print! legendre(2, 7)
check! 1

print! legendre(3, 7)
check! -1
```

## jacobi

```kototype
|Number, Number| -> Z
```

Returns the Jacobi symbol `(a / n)` for an odd positive `n`; `n` need not be
prime.

### Example

```koto
print! jacobi(2, 9)
check! 1

print! jacobi(3, 9)
check! 0
```

## kronecker

```kototype
|Number, Number| -> Z
```

Returns the Kronecker symbol `(a / n)`, extending the Jacobi symbol to even,
negative and zero bottom arguments.

### Example

```koto
print! kronecker(2, 8)
check! 0

print! kronecker(3, 8)
check! -1
```

## eulers_constant

```kototype
|| -> CF
```

Returns the infinite continued fraction for Euler's number `e`:
`[2; 1, 2, 1, 1, 4, ...]`.

### Example

```koto
e_cf = eulers_constant()
print! e_cf.take(9)
check! [2, 1, 2, 1, 1, 4, 1, 1, 6]

print! e_cf.convergent(5)
check! 87/32
```

## Z.ideal

```kototype
|Z| -> Ideal
```

Returns the principal ideal generated by the integer. The generator is
canonicalized, so negative integers produce the same ideal as their absolute
values.

### Example

```koto
print! Z(-6).ideal()
check! 6Z

print! Z(0).ideal()
check! 0Z
```
