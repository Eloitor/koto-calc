use crate::CF::CF;
use crate::NN::NN;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::Fraction;

/// A rational number, always stored in reduced form (numerator and denominator
/// are coprime, denominator is positive). Guaranteed by algebraeon's Rational.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct Q(pub Rational);

#[koto_impl]
impl Q {
    /// Converts a Koto value (Number, NN, ZZ, or Q) into an algebraeon Rational.
    pub fn rational_from_value(value: &KValue) -> Result<Rational> {
        match value {
            KValue::Number(n) => {
                if n.is_i64() {
                    Ok(Rational::from(Integer::from(i64::from(*n))))
                } else {
                    match Rational::try_from_float_simplest(f64::from(*n)) {
                        Ok(rational) => Ok(rational),
                        Err(()) => {
                            runtime_error!("cannot convert {} to a rational number", n)
                        }
                    }
                }
            }
            KValue::Object(object) => {
                if let Ok(nn) = object.cast::<NN>() {
                    Ok(Rational::from(nn.0.clone()))
                } else if let Ok(zz) = object.cast::<ZZ>() {
                    Ok(Rational::from(zz.to_integer()))
                } else if let Ok(q) = object.cast::<Q>() {
                    Ok(q.0.clone())
                } else {
                    unexpected_type("Number, NN, ZZ, or Q", value)
                }
            }
            unexpected => unexpected_type("Number, NN, ZZ, or Q", unexpected),
        }
    }

    #[koto_method]
    pub fn num(&self) -> KValue {
        let numerator: Integer = (&self.0).numerator();
        KValue::Object(KObject::from(ZZ::from_integer(numerator)))
    }

    #[koto_method]
    pub fn den(&self) -> KValue {
        let denominator: Natural = (&self.0).denominator();
        KValue::Object(KObject::from(NN(denominator)))
    }

    #[koto_method]
    pub fn is_integer(&self) -> KValue {
        self.0.is_integer().into()
    }

    #[koto_method]
    pub fn is_square(&self) -> KValue {
        self.0.is_square().into()
    }

    #[koto_method]
    pub fn sqrt_if_square(&self) -> KValue {
        match self.0.sqrt_if_square() {
            Some(sqrt) => KValue::Object(KObject::from(Self(sqrt))),
            None => KValue::Null,
        }
    }

    #[koto_method]
    pub fn height(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.0.height())))
    }

    #[koto_method]
    pub fn to_float(&self) -> KValue {
        KValue::from(f64::from(&self.0))
    }

    /// Converts to ZZ, erroring if the value is not an integer.
    #[koto_method]
    pub fn to_zz(&self) -> Result<KValue> {
        match Integer::try_from(&self.0) {
            Ok(integer) => Ok(KValue::Object(KObject::from(ZZ::from_integer(integer)))),
            Err(()) => runtime_error!("Q.to_zz() requires an integer value, got {}", self.0),
        }
    }

    /// Converts to NN, erroring if the value is not a non-negative integer.
    #[koto_method]
    pub fn to_nn(&self) -> Result<KValue> {
        match Natural::try_from(&self.0) {
            Ok(natural) => Ok(KValue::Object(KObject::from(NN(natural)))),
            Err(()) => runtime_error!("Q.to_nn() requires a non-negative integer value, got {}", self.0),
        }
    }

    /// The simplest rational number in the closed interval [self, other]
    /// (smallest denominator, then smallest numerator). Requires self <= other.
    #[koto_method]
    pub fn simplest_between(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [other] => {
                let other = Self::rational_from_value(other)?;
                if self.0 > other {
                    return runtime_error!(
                        "Q.simplest_between: expected self <= other, got {} and {}",
                        self.0,
                        other
                    );
                }
                Ok(KValue::Object(KObject::from(Self(
                    Rational::simplest_rational_in_closed_interval(&self.0, &other),
                ))))
            }
            unexpected => unexpected_args("|Q|", unexpected),
        }
    }

    /// The best rational approximation of self with denominator at most
    /// max_denominator (closest value, ties broken by smaller denominator).
    #[koto_method]
    pub fn approximate(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [max_den] => {
                let max_den = crate::CF::natural_from_value(max_den)?;
                if max_den == Natural::ZERO {
                    return runtime_error!("Q.approximate: max_denominator must be positive");
                }
                Ok(KValue::Object(KObject::from(Self(
                    self.0.clone().approximate(&max_den),
                ))))
            }
            unexpected => unexpected_args("|NN|", unexpected),
        }
    }

    /// The (finite) simple continued fraction of this rational number,
    /// e.g. Q(22, 7).to_cf() = CF([3, 7]).
    #[koto_method]
    pub fn to_cf(&self) -> KValue {
        let (num, den) = (&self.0).numerator_and_denominator();
        KValue::Object(KObject::from(CF::from_rational(num, den)))
    }
}

impl KotoObject for Q {
    fn add(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::rational_from_value(other)?;
        Ok(KValue::Object(KObject::from(Self(
            self.0.clone() + rhs,
        ))))
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::rational_from_value(other)?;
        Ok(KValue::Object(KObject::from(Self(
            self.0.clone() - rhs,
        ))))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::rational_from_value(other)?;
        Ok(KValue::Object(KObject::from(Self(
            self.0.clone() * rhs,
        ))))
    }

    fn divide(&self, other: &KValue) -> Result<KValue> {
        let rhs = Self::rational_from_value(other)?;
        if rhs == Rational::ZERO {
            return runtime_error!("division by zero");
        }
        Ok(KValue::Object(KObject::from(Self(
            self.0.clone() / rhs,
        ))))
    }

    fn add_assign(&mut self, other: &KValue) -> Result<()> {
        let rhs = Self::rational_from_value(other)?;
        self.0 += rhs;
        Ok(())
    }

    fn subtract_assign(&mut self, other: &KValue) -> Result<()> {
        let rhs = Self::rational_from_value(other)?;
        self.0 -= rhs;
        Ok(())
    }

    fn multiply_assign(&mut self, other: &KValue) -> Result<()> {
        let rhs = Self::rational_from_value(other)?;
        self.0 *= rhs;
        Ok(())
    }

    fn divide_assign(&mut self, other: &KValue) -> Result<()> {
        let rhs = Self::rational_from_value(other)?;
        if rhs == Rational::ZERO {
            return runtime_error!("division by zero");
        }
        self.0 = self.0.clone() / rhs;
        Ok(())
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(Self(-(&self.0)))))
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        let rhs = Self::rational_from_value(other)?;
        Ok(self.0 == rhs)
    }

    fn less(&self, other: &KValue) -> Result<bool> {
        let rhs = Self::rational_from_value(other)?;
        Ok(self.0 < rhs)
    }

    fn less_or_equal(&self, other: &KValue) -> Result<bool> {
        let rhs = Self::rational_from_value(other)?;
        Ok(self.0 <= rhs)
    }

    fn greater(&self, other: &KValue) -> Result<bool> {
        let rhs = Self::rational_from_value(other)?;
        Ok(self.0 > rhs)
    }

    fn greater_or_equal(&self, other: &KValue) -> Result<bool> {
        let rhs = Self::rational_from_value(other)?;
        Ok(self.0 >= rhs)
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        let (num, den) = (&self.0).numerator_and_denominator();
        if den == Natural::ONE {
            ctx.append(num.to_string());
        } else {
            ctx.append(format!("{}/{}", num, den));
        }
        Ok(())
    }
}
