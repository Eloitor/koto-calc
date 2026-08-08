use crate::Q::Q;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational, RationalCanonicalStructure};
use algebraeon::nzq::traits::{Abs, Fraction};
use algebraeon::sets::structure::{EqSignature, MetaType};
use algebraeon_rings::quaternion_algebra::{
    QuaternionAlgebraBasis, QuaternionAlgebraElement, QuaternionAlgebraStructure,
};
use algebraeon_rings::structure::{
    AdditiveGroupSignature, AdditionSignature, FreeModuleSignature, SemiModuleSignature,
};

/// The Hamilton quaternion algebra over Q, i.e. the quaternion algebra
/// (a, b) = (-1, -1) over Q with i^2 = j^2 = k^2 = i*j*k = -1.
///
/// Creating the structure is cheap (it just stores the two parameters),
/// so we build it on demand for each operation instead of storing it.
fn hamilton() -> QuaternionAlgebraStructure<RationalCanonicalStructure> {
    QuaternionAlgebraStructure::new(Rational::structure(), -Rational::ONE, -Rational::ONE)
}

/// A Hamilton quaternion over Q, x + y*i + z*j + w*k, backed by algebraeon's
/// `QuaternionAlgebraElement` for all arithmetic (multiplication follows the
/// quaternion algebra (-1, -1) over Q).
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct Quat(pub QuaternionAlgebraElement<Rational>);

impl PartialEq for Quat {
    fn eq(&self, other: &Self) -> bool {
        hamilton().equal(&self.0, &other.0)
    }
}

impl Eq for Quat {}

#[koto_impl]
impl Quat {
    /// Builds a quaternion from its four coefficients x + y*i + z*j + w*k.
    pub fn from_rationals(x: Rational, y: Rational, z: Rational, w: Rational) -> Self {
        let h = hamilton();
        let mut v = h.from_component(&QuaternionAlgebraBasis::R, &x);
        v = h.add(&v, &h.from_component(&QuaternionAlgebraBasis::I, &y));
        v = h.add(&v, &h.from_component(&QuaternionAlgebraBasis::J, &z));
        v = h.add(&v, &h.from_component(&QuaternionAlgebraBasis::K, &w));
        Self(v)
    }

    /// The four coefficients (x, y, z, w) such that self = x + y*i + z*j + w*k.
    pub fn coefficients(&self) -> (Rational, Rational, Rational, Rational) {
        let h = hamilton();
        (
            h.to_component(&QuaternionAlgebraBasis::R, &self.0).into_owned(),
            h.to_component(&QuaternionAlgebraBasis::I, &self.0).into_owned(),
            h.to_component(&QuaternionAlgebraBasis::J, &self.0).into_owned(),
            h.to_component(&QuaternionAlgebraBasis::K, &self.0).into_owned(),
        )
    }

    /// Hamilton product of two quaternions, computed directly on the
    /// coefficients: with q = x + y*i + z*j + w*k and i^2 = j^2 = k^2 =
    /// i*j*k = -1 (so i*j = k, j*k = i, k*i = j), the product is
    ///
    ///   real: x1*x2 - y1*y2 - z1*z2 - w1*w2
    ///   i:    x1*y2 + y1*x2 + z1*w2 - w1*z2
    ///   j:    x1*z2 + z1*x2 - y1*w2 + w1*y2
    ///   k:    x1*w2 + w1*x2 + y1*z2 - z1*y2
    ///
    /// NOTE: algebraeon 0.0.17's `QuaternionAlgebraStructure::mul` has a
    /// sign bug on the i/j cross terms (it returns non-associative results:
    /// (i*j)*k = -1 but i*(j*k) = 1). The real part and the k-coefficient
    /// are correct, so only `mul` is bypassed; add/sub/neg/conjugate/
    /// reduced_norm/reduced_trace from the structure are correct and are
    /// used as-is.
    fn mul_quat(&self, other: &Self) -> Self {
        let (x1, y1, z1, w1) = self.coefficients();
        let (x2, y2, z2, w2) = other.coefficients();
        let real = x1.clone() * x2.clone() - y1.clone() * y2.clone()
            - z1.clone() * z2.clone() - w1.clone() * w2.clone();
        let i = x1.clone() * y2.clone() + y1.clone() * x2.clone()
            + z1.clone() * w2.clone() - w1.clone() * z2.clone();
        let j = x1.clone() * z2.clone() + z1.clone() * x2.clone()
            - y1.clone() * w2.clone() + w1.clone() * y2.clone();
        let k = x1 * w2 + w1 * x2 + y1 * z2 - z1 * y2;
        Self::from_rationals(real, i, j, k)
    }

    /// Converts a Koto value (Number, NN, ZZ, or Q) into a Rational.
    fn rational_from_value(value: &KValue) -> Result<Rational> {
        Q::rational_from_value(value)
    }

    /// Constructor used by the koto `Quat(...)` call: exactly four
    /// coefficients (Q/Number/NN/ZZ, promoted to Q).
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [x, y, z, w] => {
                let quat = Self::from_rationals(
                    Self::rational_from_value(x)?,
                    Self::rational_from_value(y)?,
                    Self::rational_from_value(z)?,
                    Self::rational_from_value(w)?,
                );
                Ok(KValue::Object(KObject::from(quat)))
            }
            unexpected => unexpected_args("|Q, Q, Q, Q|", unexpected),
        }
    }

    /// The conjugate x - y*i - z*j - w*k.
    #[koto_method]
    pub fn conjugate(&self) -> KValue {
        KValue::Object(KObject::from(Self(hamilton().conjugate(&self.0))))
    }

    /// The reduced norm x*conj(x) = x^2 + y^2 + z^2 + w^2 (a rational).
    #[koto_method]
    pub fn norm(&self) -> KValue {
        KValue::Object(KObject::from(Q(hamilton().reduced_norm(&self.0))))
    }

    /// The reduced trace q + conj(q) = 2*x (a rational).
    #[koto_method]
    pub fn trace(&self) -> KValue {
        KValue::Object(KObject::from(Q(hamilton().reduced_trace(&self.0))))
    }

    /// The four coefficients as a tuple of Q values: (x, y, z, w).
    #[koto_method]
    pub fn coeffs(&self) -> KValue {
        let (x, y, z, w) = self.coefficients();
        KValue::Tuple(
            vec![
                KValue::Object(KObject::from(Q(x))),
                KValue::Object(KObject::from(Q(y))),
                KValue::Object(KObject::from(Q(z))),
                KValue::Object(KObject::from(Q(w))),
            ]
            .into(),
        )
    }

    /// The four coefficients as a tuple of approximate floats.
    #[koto_method]
    pub fn to_float(&self) -> KValue {
        let (x, y, z, w) = self.coefficients();
        KValue::Tuple(
            vec![
                KValue::from(f64::from(&x)),
                KValue::from(f64::from(&y)),
                KValue::from(f64::from(&z)),
                KValue::from(f64::from(&w)),
            ]
            .into(),
        )
    }
}

/// Formats a quaternion as x + y*i + z*j + w*k with correct signs, omitting
/// zero terms and the coefficient 1 on i/j/k. Coefficients are reduced
/// fractions (parenthesized when not integers, e.g. (1/2)k).
fn format_quat((x, y, z, w): (Rational, Rational, Rational, Rational)) -> String {
    let mut out = String::new();
    let terms = [(x, ""), (y, "i"), (z, "j"), (w, "k")];
    let mut first = true;
    for (coeff, suffix) in terms {
        if coeff == Rational::ZERO {
            continue;
        }
        let (num, den) = coeff.numerator_and_denominator();
        let negative = num < Integer::ZERO;
        let num_abs = num.abs();
        // Fractional coefficients are parenthesized on i/j/k terms (e.g.
        // (1/2)k) to avoid ambiguity, but the real part is shown like Q
        // (e.g. 1/3).
        let mag = if den == Natural::ONE {
            num_abs.to_string()
        } else if suffix.is_empty() {
            format!("{}/{}", num_abs, den)
        } else {
            format!("({}/{})", num_abs, den)
        };
        if !first {
            out.push_str(if negative { " - " } else { " + " });
        } else if negative {
            out.push('-');
        }
        // The coefficient 1 is omitted for i/j/k terms.
        let omit_coeff = !suffix.is_empty() && mag == "1";
        if !omit_coeff {
            out.push_str(&mag);
        }
        out.push_str(suffix);
        first = false;
    }
    if first {
        out.push('0');
    }
    out
}

impl KotoObject for Quat {
    fn add(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let rhs = other.cast::<Self>().unwrap();
                Ok(KValue::Object(KObject::from(Self(
                    hamilton().add(&self.0, &rhs.0),
                ))))
            }
            unexpected => {
                // Scalar addition adds to the real part.
                let scalar = Self::rational_from_value(unexpected)?;
                let h = hamilton();
                let mut sum = h.from_component(&QuaternionAlgebraBasis::R, &scalar);
                sum = h.add(&sum, &self.0);
                Ok(KValue::Object(KObject::from(Self(sum))))
            }
        }
    }

    fn add_rhs(&self, other: &KValue) -> Result<KValue> {
        // scalar + quat == quat + scalar
        self.add(other)
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let rhs = other.cast::<Self>().unwrap();
                Ok(KValue::Object(KObject::from(Self(
                    hamilton().sub(&self.0, &rhs.0),
                ))))
            }
            unexpected => {
                // Scalar subtraction subtracts from the real part.
                let scalar = Self::rational_from_value(unexpected)?;
                let h = hamilton();
                let mut diff = h.from_component(&QuaternionAlgebraBasis::R, &scalar);
                diff = h.sub(&self.0, &diff);
                Ok(KValue::Object(KObject::from(Self(diff))))
            }
        }
    }

    fn subtract_rhs(&self, other: &KValue) -> Result<KValue> {
        // scalar - quat
        let scalar = Self::rational_from_value(other)?;
        let h = hamilton();
        let mut diff = h.from_component(&QuaternionAlgebraBasis::R, &scalar);
        diff = h.sub(&diff, &self.0);
        Ok(KValue::Object(KObject::from(Self(diff))))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let rhs = other.cast::<Self>().unwrap();
                Ok(KValue::Object(KObject::from(self.mul_quat(&rhs))))
            }
            unexpected => {
                let scalar = Self::rational_from_value(unexpected)?;
                Ok(KValue::Object(KObject::from(Self(
                    hamilton().scalar_mul(&self.0, &scalar),
                ))))
            }
        }
    }

    fn multiply_rhs(&self, other: &KValue) -> Result<KValue> {
        // scalar * quat == quat * scalar (scalars are central)
        self.multiply(other)
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(Self(hamilton().neg(&self.0)))))
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let rhs = other.cast::<Self>().unwrap();
                Ok(hamilton().equal(&self.0, &rhs.0))
            }
            unexpected => {
                // A scalar equals a quaternion iff the quaternion is that
                // scalar (zero i/j/k parts).
                let scalar = Self::rational_from_value(unexpected)?;
                let (x, y, z, w) = self.coefficients();
                Ok(x == scalar && y == Rational::ZERO && z == Rational::ZERO && w == Rational::ZERO)
            }
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format_quat(self.coefficients()));
        Ok(())
    }
}
