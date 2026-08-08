use crate::Q::Q;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::Rational;
use algebraeon_rings::approximation::{RationalInterval as AlgRationalInterval, RealApproximatePoint, Subset};

/// An exact real constant (`e()` or `pi()`) backed by algebraeon's
/// `RealApproximatePoint`, which can be refined to arbitrary precision.
///
/// The object is immutable from the caller's point of view: `to_float` and
/// `refine` only ever shrink the internal rational interval that contains the
/// constant (the underlying point is shared, so any copy of the object sees
/// the same, ever-tighter interval).
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct Real {
    point: RealApproximatePoint,
}

/// The current rational interval (a, b) containing the constant.
fn interval_of(point: &RealApproximatePoint) -> (Rational, Rational) {
    match point.lock().rational_interval_neighbourhood() {
        Subset::Singleton(rational) => (rational.clone(), rational),
        Subset::Interval(interval) => (interval.a().clone(), interval.b().clone()),
    }
}

#[koto_impl]
impl Real {
    /// The exact constant e (Euler's number).
    pub fn e() -> Self {
        Self {
            point: algebraeon_rings::approximation::e(),
        }
    }

    /// The exact constant pi.
    pub fn pi() -> Self {
        Self {
            point: algebraeon_rings::approximation::pi(),
        }
    }

    /// The current rational bounds of the constant, as a pair of Q values
    /// `[a, b]` with a <= value <= b.
    #[koto_method]
    pub fn bounds(&self) -> KValue {
        let (a, b) = interval_of(&self.point);
        KValue::List(KList::with_data(
            vec![
                KValue::Object(KObject::from(Q(a))),
                KValue::Object(KObject::from(Q(b))),
            ]
            .into(),
        ))
    }

    /// The current accuracy of the approximation: the length of the interval
    /// containing the constant, as a Q value (smaller means more precise).
    #[koto_method]
    pub fn accuracy(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.point.lock().length())))
    }

    /// Refines the internal interval once (making it strictly narrower for
    /// non-rational constants) and returns the Real itself so calls can be
    /// chained.
    #[koto_method]
    pub fn refine(&self) -> KValue {
        self.point.lock().refine();
        KValue::Object(KObject::from(self.clone()))
    }

    /// A floating point approximation with `decimals` decimal digits. The
    /// internal interval is refined as needed so that the rounded result is
    /// guaranteed correct; the object stays immutable (refinement only
    /// shrinks the interval).
    #[koto_method]
    pub fn to_float(&self, args: &[KValue]) -> Result<KValue> {
        let decimals = match args {
            [KValue::Number(n)] if n.is_i64() && i64::from(*n) >= 0 => i64::from(*n) as u32,
            unexpected => return unexpected_args("|NN|", unexpected),
        };
        // Refine until the interval is narrower than half of 10^-decimals, so
        // the midpoint rounds correctly to `decimals` decimal places.
        let mut target = Rational::ONE;
        for _ in 0..decimals {
            target = target / Rational::from(10);
        }
        target = target / Rational::from(2);
        self.point.lock().refine_to_length(&target);
        let (a, b) = interval_of(&self.point);
        let midpoint = (a + b) / Rational::from(2);
        let f = f64::from(&midpoint);
        let factor = 10f64.powi(decimals as i32);
        Ok(KValue::from((f * factor).round() / factor))
    }
}

impl KotoObject for Real {
    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        let (a, b) = interval_of(&self.point);
        ctx.append(format!("({}, {})", a, b));
        Ok(())
    }
}

/// An open interval (a, b) of rationals, mirroring algebraeon's
/// `RationalInterval`. Built with `RationalInterval(a, b)`; requires a < b.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct RationalInterval(pub AlgRationalInterval);

#[koto_impl]
impl RationalInterval {
    /// RationalInterval(a, b) builds the open interval (a, b), erroring
    /// unless a < b.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [a, b] => {
                let a = Q::rational_from_value(a)?;
                let b = Q::rational_from_value(b)?;
                if a >= b {
                    return runtime_error!(
                        "RationalInterval: expected a < b, got {} and {}",
                        a,
                        b
                    );
                }
                Ok(KValue::Object(KObject::from(Self(
                    AlgRationalInterval::new_unchecked(a, b),
                ))))
            }
            unexpected => unexpected_args("|Q, Q|", unexpected),
        }
    }

    /// The left endpoint a.
    #[koto_method]
    pub fn a(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.0.a().clone())))
    }

    /// The right endpoint b.
    #[koto_method]
    pub fn b(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.0.b().clone())))
    }

    /// The negated interval (-b, -a).
    #[koto_method]
    pub fn neg(&self) -> KValue {
        KValue::Object(KObject::from(Self(self.0.clone().neg())))
    }
}

impl KotoObject for RationalInterval {
    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format!("({}, {})", self.0.a(), self.0.b()));
        Ok(())
    }
}
