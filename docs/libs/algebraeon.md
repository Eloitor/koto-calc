# algebraeon

[Algebraeon](https://crates.io/crates/algebraeon) support for Koto:
arbitrary precision arithmetic, number theory, polynomials, and matrices.

The module provides the types [`NN`](#nn) (natural numbers),
[`ZZ`](#zz) (integers), [`Q`](#q) (rationals),
[`Poly`](#poly) (univariate polynomials), [`Mat`](#mat) (matrices),
[`Quat`](#quat) (Hamilton quaternions) and [`Alg`](#alg) (real algebraic
numbers), plus the module-level functions [`gcd`](#gcd) and [`lcm`](#lcm).

## NN

```kototype
|| -> Iterator
|Number| -> NN
```

Natural (non-negative integer) values with arbitrary precision.

Called with no arguments, `NN()` returns an iterator over the natural numbers
`0, 1, 2, ...`.

### Example

```koto
print! NN(5).factorial()
check! 120

print! NN(5) - NN(3)
check! 2

print! NN().take(4).to_list()
check! [0, 1, 2, 3]
```

## NN.bitcount

```kototype
|NN| -> Number
```

Returns the number of bits needed to represent the value.

### Example

```koto
print! NN(5).bitcount()
check! 3
```

## NN.is_prime

```kototype
|NN| -> Bool
```

Returns `true` if the value is prime.

### Example

```koto
print! NN(17).is_prime()
check! true

print! NN(12).is_prime()
check! false
```

## NN.is_squarefree

```kototype
|NN| -> Bool
```

Returns `true` if the value has no repeated prime factors.

### Example

```koto
print! NN(10).is_squarefree()
check! true

print! NN(12).is_squarefree()
check! false
```

## NN.factor

```kototype
|NN| -> [(NN, NN)]
```

Returns the prime factorization of the value as a list of
`(prime, exponent)` tuples.

### Example

```koto
print! NN(60).factor()
check! [(2, 2), (3, 1), (5, 1)]
```

## NN.factorial

```kototype
|NN| -> NN
```

Returns the factorial of the value.

### Example

```koto
print! NN(5).factorial()
check! 120
```

## NN.divisors

```kototype
|NN| -> [NN]
```

Returns the value's divisors in ascending order.

### Example

```koto
print! NN(12).divisors()
check! [1, 2, 3, 4, 6, 12]
```

## NN.euler_totient

```kototype
|NN| -> NN
```

Returns the value of [Euler's totient function](https://en.wikipedia.org/wiki/Euler%27s_totient_function),
the count of positive integers up to the value that are coprime to it.

### Example

```koto
print! NN(10).euler_totient()
check! 4
```

## NN.is_square

```kototype
|NN| -> Bool
```

Returns `true` if the value is a perfect square.

### Example

```koto
print! NN(16).is_square()
check! true

print! NN(18).is_square()
check! false
```

## NN.sqrt_floor

```kototype
|NN| -> NN
```

Returns the floor of the square root of the value.

### Example

```koto
print! NN(17).sqrt_floor()
check! 4
```

## NN.sqrt_ceil

```kototype
|NN| -> NN
```

Returns the ceiling of the square root of the value.

### Example

```koto
print! NN(17).sqrt_ceil()
check! 5
```

## NN.is_power_test

```kototype
|NN| -> (Bool, NN?, NN?)
```

Returns `(true, base, exponent)` if the value can be written as `base^exponent`
with `exponent > 1`, otherwise `(false, null, null)`.

### Example

```koto
print! NN(8).is_power_test()
check! (true, 2, 3)

print! NN(6).is_power_test()
check! (false, null, null)
```

## NN.primality_test

```kototype
|NN| -> String
```

Returns `'prime'` or `'composite'` (both `0` and `1` are reported as
`'composite'`).

### Example

```koto
print! NN(17).primality_test()
check! prime

print! NN(12).primality_test()
check! composite
```

## NN.primes

```kototype
|| -> Iterator
```

Returns an iterator over the prime numbers.

### Example

```koto
print! NN.primes().take(6).to_list()
check! [2, 3, 5, 7, 11, 13]
```

## ZZ

```kototype
|Number| -> ZZ
```

Integer values with arbitrary precision.

`ZZ` supports arithmetic (`+ - *`), comparisons, and assignment operators
(`+= -= *=`) with other `ZZ` values, `NN` values, and plain numbers.

### Example

```koto
print! ZZ(5) + ZZ(-3)
check! 2

print! ZZ(4) * ZZ(-2)
check! -8

print! ZZ(5) + NN(3)
check! 8
```

## ZZ.abs

```kototype
|ZZ| -> NN
```

Returns the absolute value of the integer as an `NN`.

### Example

```koto
print! ZZ(-9).abs()
check! 9
```

## ZZ.is_irreducible

```kototype
|ZZ| -> Bool
```

Returns `true` if the value is irreducible (i.e. a prime, up to sign).

### Example

```koto
print! ZZ(7).is_irreducible()
check! true

print! ZZ(9).is_irreducible()
check! false
```

## ZZ.is_square

```kototype
|ZZ| -> Bool
```

Returns `true` if the value is a perfect square.

### Example

```koto
print! ZZ(9).is_square()
check! true

print! ZZ(10).is_square()
check! false
```

## ZZ.factor

```kototype
|ZZ| -> [(ZZ, NN)]
```

Returns the prime factorization of the value as a list of
`(prime, exponent)` tuples. The sign is ignored.

### Example

```koto
print! ZZ(-12).factor()
check! [(2, 2), (3, 1)]
```

## ZZ.divmod

```kototype
|ZZ, ZZ| -> (ZZ, ZZ)
```

Returns the quotient and remainder of a floor division, with a
non-negative remainder.

### Example

```koto
print! ZZ(-7).divmod(ZZ(3))
check! (-3, 2)
```

## ZZ.div_floor

```kototype
|ZZ, ZZ| -> ZZ
```

Returns the quotient of a floor division.

### Example

```koto
print! ZZ(-7).div_floor(ZZ(3))
check! -3

print! ZZ(-13).div_floor(ZZ(5))
check! -3
```

## ZZ.mod

```kototype
|ZZ, ZZ| -> ZZ
```

Returns the non-negative remainder of a floor division, coherent with
[`div_floor`](#zz-div-floor).

### Example

```koto
print! ZZ(-7).mod(ZZ(3))
check! 2

print! ZZ(-13).mod(ZZ(5))
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
operators (`+= -= *= /=`) with other `Q` values, `NN` values, `ZZ` values and
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
|Q| -> ZZ
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
|Q| -> NN
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
|Q| -> NN
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
|Q| -> ZZ
```

Converts the value to a `ZZ` (the value must be a whole number).

### Example

```koto
print! Q(3).to_zz()
check! 3

print! Q(4, 2).to_zz()
check! 2
```

## Q.to_nn

```kototype
|Q| -> NN
```

Converts the value to an `NN` (the value must be a non-negative whole number).

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

Univariate polynomials over `ZZ` or `Q`.

The constructor takes a list of coefficients in ascending order, with the
first element being the constant term: `Poly([6, -5, 1])` represents
`6 - 5x + x^2`.

The coefficients are stored as `ZZ` when all of them are integers, and
promoted to `Q` when any of them is a fraction. Arithmetic (`+ - *`) works
with other polynomials and with `NN`/`ZZ`/`Q` scalars, promoting `ZZ` to `Q`
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
|Poly| -> NN
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
|Poly| -> [ZZ] | [Q]
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
|Poly, x: Number| -> ZZ | Q
```

Evaluates the polynomial at `x` (which may be a `Number`, `NN`, `ZZ` or `Q`).

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
|Poly| -> [(Poly, NN)]
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

Matrices over `ZZ` or `Q`, given row by row: `Mat([[1, 2], [3, 4]])` is the
`2x2` matrix with rows `[1, 2]` and `[3, 4]`.

The entries are stored as `ZZ` when all of them are integers, and promoted
to `Q` when any of them is a fraction. Arithmetic (`+ - *`) works with other
matrices and with `NN`/`ZZ`/`Q` scalars.

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
|Mat| -> NN
```

Returns the number of rows.

### Example

```koto
print! Mat([[1, 2], [3, 4]]).rows()
check! 2
```

## Mat.cols

```kototype
|Mat| -> NN
```

Returns the number of columns.

### Example

```koto
print! Mat([[1, 2], [3, 4]]).cols()
check! 2
```

## Mat.at

```kototype
|Mat, row: Number, col: Number| -> ZZ | Q
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
|Mat| -> ZZ | Q
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

Returns the inverse of the matrix over `Q` (a `ZZ` matrix is promoted to
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
|NN, NN| -> NN
```

Returns the greatest common divisor of two natural numbers.

### Example

```koto
print! gcd(NN(12), NN(18))
check! 6
```

## lcm

```kototype
|NN, NN| -> NN
```

Returns the least common multiple of two natural numbers.

### Example

```koto
print! lcm(NN(4), NN(6))
check! 12
```

## Quat

```kototype
|Number, Number, Number, Number| -> Quat
```

[Hamilton quaternions](https://en.wikipedia.org/wiki/Quaternion) over `Q`,
constructed from four coefficients `a + bi + cj + dk` (each
`Number`/`NN`/`ZZ`/`Q` argument is promoted to `Q`).

Multiplication is the Hamilton product, defined by
`i^2 = j^2 = k^2 = ijk = -1` with `i*j = k`, `j*k = i` and `k*i = j` (the
cross terms anti-commute: `j*i = -k`, ...), so multiplication is not
commutative. The product is computed directly on the coefficients: the
wrapper works around a bug in algebraeon 0.0.17 (upstream issue #244) that
produced wrong signs in the `i`/`j` cross terms of
`QuaternionAlgebraStructure::mul`.

`Quat` supports arithmetic (`+ - *`) with other `Quat` values and with
scalars (`Number`/`NN`/`ZZ`/`Q`, on either side), negation, and equality
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
takes a `Poly` (over `ZZ` or `Q`) or a coefficient list (as in
`Poly([...])`), and returns the **list of isolated real roots in increasing
order**, with multiplicity. Polynomials of degree `0` (including the zero
polynomial) and polynomials without real roots give an empty list.

Each `Alg` value is a root with an isolating interval, so comparisons are
exact: `<`, `<=`, `>`, `>=` and `==` work between two `Alg` values and
between an `Alg` and a scalar (`Q`/`NN`/`ZZ`/`Number`, compared exactly as
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
argument may be another `Alg` or a scalar (`Number`/`NN`/`ZZ`/`Q`),
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
