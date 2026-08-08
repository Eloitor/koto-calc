use crate::NN::NN;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::Integer;
use algebraeon::nzq::traits::Abs;
use algebraeon_rings::structure::MetaFactoringMonoid;

#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct ZZ(Integer);

#[koto_impl]
impl ZZ {
    pub fn make_koto_object(n: KNumber) -> KObject {
        let my_int = Integer::from(i64::from(n));
        KObject::from(Self(my_int))
    }

    #[koto_method]
    pub fn abs(&self) -> KValue {
        KValue::Object(KObject::from(NN::from(NN(self.0.clone().abs()))))
    }

    #[koto_method]
    pub fn is_irreducible(&self) -> KValue {
        self.0.is_irreducible().into()
    }

    #[koto_method]
    pub fn is_square(&self) -> KValue {
        self.0.is_square().into()
    }
}

impl KotoObject for ZZ {
    fn add(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() + other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            KValue::Object(other) if other.is_a::<NN>() => {
                let other = other.cast::<NN>().unwrap();
                let result = self.0.clone() + Integer::from(other.clone().0);
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            KValue::Number(other) if other.is_i64() => {
                let result = ZZ(self.0.clone() + Integer::from(i64::from(other)));
                Ok(KValue::Object(KObject::from(result)))
            }
            unexpected => unexpected_type("ZZ", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.0 == other.0)
            }
            unexpected => unexpected_type("Number", unexpected),
        }
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(Self(-self.0.clone()))))
    }

    fn multiply(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() * other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("ZZ", unexpected),
        }
    }

    fn subtract(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() - other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("ZZ", unexpected),
        }
    }

    fn add_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 += other.0.clone();
                Ok(())
            }
            KValue::Object(other) if other.is_a::<NN>() => {
                let other = other.cast::<NN>().unwrap();
                self.0 += Integer::from(other.0.clone());
                Ok(())
            }
            KValue::Number(other) if other.is_i64() => {
                self.0 += Integer::from(i64::from(other));
                Ok(())
            }
            unexpected => unexpected_type("ZZ, NN, or Number", unexpected),
        }
    }

    fn multiply_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 *= other.0.clone();
                Ok(())
            }
            KValue::Object(other) if other.is_a::<NN>() => {
                let other = other.cast::<NN>().unwrap();
                self.0 *= Integer::from(other.0.clone());
                Ok(())
            }
            KValue::Number(other) if other.is_i64() => {
                self.0 *= Integer::from(i64::from(other));
                Ok(())
            }
            unexpected => unexpected_type("ZZ, NN, or Number", unexpected),
        }
    }
}
