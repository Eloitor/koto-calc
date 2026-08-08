use crate::Poly::{Poly, PolyCoeff};
use crate::Q::Q;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::Fraction;
use algebraeon_rings::isolated_algebraic::RealAlgebraic;

/// A real algebraic number: either a rational number or an isolated real
/// root of a polynomial with integer coefficients (algebraeon 0.0.17,
/// module `isolated_algebraic::real`).
///
/// `Alg` values are constructed through the koto `Alg(...)` call, which
/// isolates the real roots of a `Poly` (or a coefficient list) and returns
/// the list of roots in increasing order. Roots are returned with
/// multiplicity (as algebraeon's `Polynomial::real_roots` does).
///
/// Comparison between two `Alg` values is exact: it refines the isolating
/// intervals until the roots can be decided to be equal or different
/// (`cmp_mut`/`cmp_rat_mut` of algebraeon).
///
/// Arithmetic between algebraic numbers is NOT exposed in this wrapper:
/// algebraeon implements it through `RealAlgebraicStructure` (sum/product
/// polynomials + root identification), but the scope of this bead is
/// construction, comparison, refinement and decimal approximation.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct Alg(pub RealAlgebraic);

/// Formats a rational as a decimal approximation with `decimals` places,
/// trimming trailing zeros (2.0 -> "2", 1/3 -> "0.333333333").
fn format_decimal(r: &Rational, decimals: usize) -> String {
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
impl Alg {
    /// Builds an Alg from a Koto value: either an `Alg`, or a scalar
    /// (Number/NN/ZZ/Q) which is interpreted as the exact rational number.
    fn alg_from_value(value: &KValue) -> Result<Alg> {
        match value {
            KValue::Object(object) if object.is_a::<Alg>() => {
                Ok(object.cast::<Alg>()?.clone())
            }
            scalar => {
                let rat = Q::rational_from_value(scalar)?;
                Ok(Alg(RealAlgebraic::Rational(rat)))
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

    /// Constructor used by the koto `Alg(...)` call: isolates the real roots
    /// of the polynomial and returns the LIST of roots (in increasing order,
    /// with multiplicity). Degree-0 polynomials (including the zero
    /// polynomial) and polynomials without real roots give an empty list.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [value] => {
                let poly = Self::poly_from_value(value)?;
                let roots: Vec<RealAlgebraic> = match &poly.poly {
                    PolyCoeff::ZZ(p) => {
                        if p.num_coeffs() == 0 {
                            vec![]
                        } else {
                            p.all_real_roots()
                        }
                    }
                    PolyCoeff::QQ(p) => {
                        if p.num_coeffs() == 0 {
                            vec![]
                        } else {
                            p.all_real_roots()
                        }
                    }
                };
                let values: Vec<KValue> = roots
                    .into_iter()
                    .map(|root| KValue::Object(KObject::from(Alg(root))))
                    .collect();
                Ok(KValue::List(KList::with_data(values.into())))
            }
            unexpected => unexpected_args("|Poly or List|", unexpected),
        }
    }

    /// The exact comparison of two algebraic numbers: -1 if self < other,
    /// 0 if equal, 1 if self > other. Also accepts a scalar (Number/NN/ZZ/Q)
    /// which is compared exactly as a rational.
    #[koto_method]
    pub fn cmp(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [other] => {
                let other = Self::alg_from_value(other)?;
                let ordering = self.0.clone().cmp_mut(&mut other.0.clone());
                let result = match ordering {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                Ok(KValue::Number(result.into()))
            }
            unexpected => unexpected_args("|Alg or scalar|", unexpected),
        }
    }

    /// The width of the isolating interval (an exact rational). Rational
    /// values have accuracy 0.
    #[koto_method]
    pub fn accuracy(&self) -> KValue {
        let accuracy = match &self.0 {
            RealAlgebraic::Rational(_) => Rational::ZERO,
            RealAlgebraic::Real(root) => root.accuracy(),
        };
        KValue::Object(KObject::from(Q(accuracy)))
    }

    /// Returns a new Alg whose isolating interval has been refined to the
    /// requested (positive) accuracy. Rational values are returned unchanged.
    #[koto_method]
    pub fn refine(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [accuracy] => {
                let target = Q::rational_from_value(accuracy)?;
                if target <= Rational::ZERO {
                    return runtime_error!(
                        "Alg.refine() requires a positive accuracy, got {}",
                        target
                    );
                }
                let mut refined = self.clone();
                if let RealAlgebraic::Real(root) = &mut refined.0 {
                    root.refine_to_accuracy_mut(&target);
                }
                Ok(KValue::Object(KObject::from(refined)))
            }
            unexpected => unexpected_args("|Q|", unexpected),
        }
    }

    /// The minimal polynomial of the algebraic number, with rational
    /// coefficients (a Poly over Q). For a rational value n/d it is
    /// d*x - n.
    #[koto_method]
    pub fn min_poly(&self) -> KValue {
        let poly = self.0.min_poly();
        KValue::Object(KObject::from(Poly {
            poly: PolyCoeff::QQ(poly),
        }))
    }

    /// A floating-point approximation (Number). The isolating interval is
    /// refined to accuracy 10^-15 before converting the midpoint.
    #[koto_method]
    pub fn to_float(&self) -> KValue {
        match &self.0 {
            RealAlgebraic::Rational(rational) => KValue::from(f64::from(rational)),
            RealAlgebraic::Real(root) => {
                let mut root = root.clone();
                root.refine_to_accuracy_mut(&Rational::from_integers(
                    Integer::from(1),
                    Integer::from(1_000_000_000_000_000i64),
                ));
                let midpoint = (root.tight_a() + root.tight_b()) / Rational::TWO;
                KValue::from(f64::from(&midpoint))
            }
        }
    }
}

impl KotoObject for Alg {
    fn equal(&self, other: &KValue) -> Result<bool> {
        let other = Self::alg_from_value(other)?;
        Ok(self.0.clone().cmp_mut(&mut other.0.clone()).is_eq())
    }

    fn less(&self, other: &KValue) -> Result<bool> {
        let other = Self::alg_from_value(other)?;
        Ok(self.0.clone().cmp_mut(&mut other.0.clone()).is_lt())
    }

    fn less_or_equal(&self, other: &KValue) -> Result<bool> {
        let other = Self::alg_from_value(other)?;
        Ok(!self.0.clone().cmp_mut(&mut other.0.clone()).is_gt())
    }

    fn greater(&self, other: &KValue) -> Result<bool> {
        let other = Self::alg_from_value(other)?;
        Ok(self.0.clone().cmp_mut(&mut other.0.clone()).is_gt())
    }

    fn greater_or_equal(&self, other: &KValue) -> Result<bool> {
        let other = Self::alg_from_value(other)?;
        Ok(!self.0.clone().cmp_mut(&mut other.0.clone()).is_lt())
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        match &self.0 {
            RealAlgebraic::Rational(rational) => {
                // Exact rationals are shown as reduced fractions (like Q).
                let (num, den) = rational.numerator_and_denominator();
                if den == Natural::ONE {
                    ctx.append(num.to_string());
                } else {
                    ctx.append(format!("{}/{}", num, den));
                }
            }
            RealAlgebraic::Real(root) => {
                // Irrational roots are shown as a decimal approximation:
                // the interval is refined to accuracy 10^-10 and the
                // midpoint is printed with 9 decimals (trailing zeros
                // trimmed). Refinement always converges, so no interval
                // fallback is needed.
                let mut root = root.clone();
                root.refine_to_accuracy_mut(&Rational::from_integers(
                    Integer::from(1),
                    Integer::from(10_000_000_000i64),
                ));
                let midpoint = (root.tight_a() + root.tight_b()) / Rational::TWO;
                ctx.append(format_decimal(&midpoint, 9));
            }
        }
        Ok(())
    }
}
