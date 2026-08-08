use crate::Alg::Alg;
use crate::NN::NN;
use crate::Poly::{Poly, PolyCoeff};
use crate::Q::Q;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::Fraction;
use algebraeon::sets::structure::MetaType;
use algebraeon_rings::isolated_algebraic::ComplexAlgebraic;
use algebraeon_rings::isolated_algebraic::RealAlgebraic;
use algebraeon_rings::structure::{
    AdditiveGroupSignature, AdditionSignature, ComplexConjugateSignature,
    ComplexSubsetSignature, MultiplicationSignature, TryReciprocalSignature,
};

/// A complex algebraic number: either a rational number, an isolated real
/// algebraic root, or an isolated complex root of a polynomial with integer
/// coefficients (algebraeon 0.0.17, module `isolated_algebraic::complex`).
///
/// `ComplexAlg` values are constructed through the koto `ComplexAlg(...)`
/// call:
/// - `ComplexAlg(poly)` (a `Poly` or a coefficient list) returns the LIST of
///   complex roots with multiplicity, in algebraeon's order (real roots
///   first, then the lower-half-plane conjugate before its upper-half-plane
///   partner, e.g. `ComplexAlg(Poly([1, 0, 1]))` is `[-i, i]`).
/// - `ComplexAlg(a, b)` with two scalars builds `a + b*i` exactly.
/// - `ComplexAlg(x)` with a single scalar builds the rational `x`.
///
/// Arithmetic (+, -, *, /) between `ComplexAlg` values and scalars is exact:
/// `ComplexAlgebraicCanonicalStructure` computes the sum/product polynomials
/// and identifies the resulting root, and `try_reciprocal` handles division
/// (with a clear error for a zero divisor). `ComplexAlg.i` is the imaginary
/// unit and `.real`/`.imag` return the real and imaginary parts as `Alg`.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct ComplexAlg(pub ComplexAlgebraic);

/// Formats an f64 with `decimals` places, trimming trailing zeros
/// (2.0 -> "2", -1.5 -> "-1.5").
fn format_decimal(value: f64, decimals: usize) -> String {
    let mut s = format!("{:.*}", decimals, value);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s == "-0" || s.is_empty() {
        s = "0".into();
    }
    s
}

/// Formats a rational as a decimal approximation with `decimals` places,
/// trimming trailing zeros (2.0 -> "2", 1/3 -> "0.333333333").
fn format_decimal_rat(r: &Rational, decimals: usize) -> String {
    let v = f64::from(r);
    let mut s = format!("{:.*}", decimals, v);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s == "-0" || s.is_empty() {
        s = "0".into();
    }
    s
}

#[koto_impl]
impl ComplexAlg {
    /// Builds a ComplexAlg from a Koto value: either an existing ComplexAlg
    /// or a scalar (Number/NN/ZZ/Q) which is interpreted as the exact real
    /// rational value.
    fn complex_from_value(value: &KValue) -> Result<ComplexAlg> {
        match value {
            KValue::Object(object) if object.is_a::<ComplexAlg>() => {
                Ok(object.cast::<ComplexAlg>()?.clone())
            }
            scalar => {
                let rat = Q::rational_from_value(scalar)?;
                Ok(ComplexAlg(ComplexAlgebraic::Real(RealAlgebraic::Rational(
                    rat,
                ))))
            }
        }
    }

    /// Builds a Poly from a Koto value: a `Poly` object or a coefficient list.
    fn poly_from_value(value: &KValue) -> Result<Poly> {
        match value {
            KValue::Object(object) if object.is_a::<Poly>() => {
                Ok(object.cast::<Poly>()?.clone())
            }
            KValue::List(list) => {
                let kobject = Poly::from_koto_list(list)?;
                Ok(kobject.cast::<Poly>()?.clone())
            }
            unexpected => unexpected_type("Poly or List", unexpected),
        }
    }

    /// Whether the value is a polynomial (Poly object or coefficient list),
    /// which is used to distinguish the roots constructor from the scalar one.
    fn is_poly_or_list(value: &KValue) -> bool {
        match value {
            KValue::List(_) => true,
            KValue::Object(object) => object.is_a::<Poly>(),
            _ => false,
        }
    }

    /// Constructor used by the koto `ComplexAlg(...)` call. See the type
    /// documentation for the three forms (roots of a polynomial, a + b*i,
    /// or a rational).
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            // ComplexAlg(Poly) / ComplexAlg([coeffs]) -> list of complex roots
            [value] if Self::is_poly_or_list(value) => {
                let poly = Self::poly_from_value(value)?;
                let roots: Vec<ComplexAlgebraic> = match &poly.poly {
                    PolyCoeff::ZZ(p) => {
                        if p.num_coeffs() == 0 {
                            vec![]
                        } else {
                            p.all_complex_roots()
                        }
                    }
                    PolyCoeff::QQ(p) => {
                        if p.num_coeffs() == 0 {
                            vec![]
                        } else {
                            p.all_complex_roots()
                        }
                    }
                };
                let values: Vec<KValue> = roots
                    .into_iter()
                    .map(|root| KValue::Object(KObject::from(ComplexAlg(root))))
                    .collect();
                Ok(KValue::List(KList::with_data(values.into())))
            }
            // ComplexAlg(a, b) = a + b*i
            [a, b] => {
                let a = Q::rational_from_value(a)?;
                let b = Q::rational_from_value(b)?;
                let structure = ComplexAlgebraic::structure();
                let a = ComplexAlgebraic::Real(RealAlgebraic::Rational(a));
                let b = ComplexAlgebraic::Real(RealAlgebraic::Rational(b));
                let bi = structure.mul(&b, &ComplexAlgebraic::i());
                let z = structure.add(&a, &bi);
                Ok(KValue::Object(KObject::from(ComplexAlg(z))))
            }
            // ComplexAlg(scalar) -> the exact rational
            [value] => {
                let rat = Q::rational_from_value(value)?;
                Ok(KValue::Object(KObject::from(ComplexAlg(
                    ComplexAlgebraic::Real(RealAlgebraic::Rational(rat)),
                ))))
            }
            unexpected => unexpected_args(
                "|Poly or List|, |scalar| or |scalar, scalar|",
                unexpected,
            ),
        }
    }

    /// The real part of the complex algebraic number, as an exact `Alg`
    /// (e.g. `ComplexAlg(Q(1), Q(2)).real()` is the rational 1).
    #[koto_method]
    pub fn real(&self) -> KValue {
        KValue::Object(KObject::from(Alg(self.0.real_part())))
    }

    /// The imaginary part of the complex algebraic number, as an exact `Alg`
    /// (e.g. `ComplexAlg(Q(1), Q(2)).imag()` is the rational 2).
    #[koto_method]
    pub fn imag(&self) -> KValue {
        KValue::Object(KObject::from(Alg(self.0.imag_part())))
    }

    /// The complex conjugate: a + b*i becomes a - b*i (real values are
    /// returned unchanged).
    #[koto_method]
    pub fn conjugate(&self) -> KValue {
        KValue::Object(KObject::from(ComplexAlg(
            ComplexAlgebraic::structure().conjugate(&self.0),
        )))
    }

    /// The minimal polynomial over Q, as a Poly (e.g. `ComplexAlg.i` has
    /// min_poly x^2 + 1 and a rational n/d has d*x - n).
    #[koto_method]
    pub fn min_poly(&self) -> KValue {
        let poly = self.0.min_poly();
        KValue::Object(KObject::from(Poly {
            poly: PolyCoeff::QQ(poly),
        }))
    }

    /// The degree of the minimal polynomial over Q (1 for rationals,
    /// 2 for the imaginary unit, ...).
    #[koto_method]
    pub fn degree(&self) -> KValue {
        KValue::Object(KObject::from(NN(Natural::from(self.0.degree()))))
    }

    /// A floating-point approximation as the list [real, imag] of two
    /// Numbers. The isolating box is refined to ~10^-18 before converting.
    #[koto_method]
    pub fn to_float(&self) -> KValue {
        let (re, im) = ComplexAlgebraic::structure().as_f64_real_and_imaginary_parts(&self.0);
        KValue::List(KList::with_data(
            vec![KValue::from(re), KValue::from(im)].into(),
        ))
    }
}

impl KotoObject for ComplexAlg {
    fn add(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::complex_from_value(other)?;
        Ok(KValue::Object(KObject::from(ComplexAlg(
            ComplexAlgebraic::structure().add(&self.0, &rhs.0),
        ))))
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::complex_from_value(other)?;
        Ok(KValue::Object(KObject::from(ComplexAlg(
            ComplexAlgebraic::structure().sub(&self.0, &rhs.0),
        ))))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::complex_from_value(other)?;
        Ok(KValue::Object(KObject::from(ComplexAlg(
            ComplexAlgebraic::structure().mul(&self.0, &rhs.0),
        ))))
    }

    fn divide(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::complex_from_value(other)?;
        let structure = ComplexAlgebraic::structure();
        match structure.try_reciprocal(&rhs.0) {
            Some(inv) => Ok(KValue::Object(KObject::from(ComplexAlg(
                structure.mul(&self.0, &inv),
            )))),
            None => runtime_error!("division by zero"),
        }
    }

    fn divide_rhs(&self, other: &KValue) -> Result<KValue> {
        // other / self = other * self^-1 (the field is commutative).
        // This makes `1 / i` (object on the RHS) compute the inverse.
        let lhs = Self::complex_from_value(other)?;
        let structure = ComplexAlgebraic::structure();
        match structure.try_reciprocal(&self.0) {
            Some(inv) => Ok(KValue::Object(KObject::from(ComplexAlg(
                structure.mul(&lhs.0, &inv),
            )))),
            None => runtime_error!("division by zero"),
        }
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(ComplexAlg(
            ComplexAlgebraic::structure().neg(&self.0),
        ))))
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        let other = Self::complex_from_value(other)?;
        Ok(self.0 == other.0)
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        match &self.0 {
            ComplexAlgebraic::Real(RealAlgebraic::Rational(rational)) => {
                // Exact rationals are shown as reduced fractions (like Q).
                let (num, den) = rational.numerator_and_denominator();
                if den == Natural::ONE {
                    ctx.append(num.to_string());
                } else {
                    ctx.append(format!("{}/{}", num, den));
                }
            }
            ComplexAlgebraic::Real(RealAlgebraic::Real(root)) => {
                // Irrational real roots are shown as a decimal approximation
                // (same convention as Alg).
                let mut root = root.clone();
                root.refine_to_accuracy_mut(&Rational::from_integers(
                    Integer::from(1),
                    Integer::from(10_000_000_000i64),
                ));
                let midpoint = (root.tight_a() + root.tight_b()) / Rational::TWO;
                ctx.append(format_decimal_rat(&midpoint, 9));
            }
            ComplexAlgebraic::Complex(_root) => {
                // Non-real roots are shown as a + b*i with decimals: the
                // isolating box is refined to ~10^-18 and the midpoints are
                // printed with 6 decimals (trailing zeros trimmed).
                let (re, im) =
                    ComplexAlgebraic::structure().as_f64_real_and_imaginary_parts(&self.0);
                let re_s = format_decimal(re, 6);
                let im_abs_s = format_decimal(im.abs(), 6);
                let im_neg = im < 0.0;
                if re_s == "0" {
                    if im_abs_s == "1" {
                        ctx.append(if im_neg { "-i" } else { "i" });
                    } else {
                        ctx.append(format!(
                            "{}{}i",
                            if im_neg { "-" } else { "" },
                            im_abs_s
                        ));
                    }
                } else {
                    ctx.append(format!(
                        "{} {} {}i",
                        re_s,
                        if im_neg { "-" } else { "+" },
                        im_abs_s
                    ));
                }
            }
        }
        Ok(())
    }
}
