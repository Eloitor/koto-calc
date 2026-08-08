use koto_runtime::{IsIterable, KIteratorOutput, KotoVm, Result, derive::*, prelude::*};

use algebraeon::nzq::Natural;
use algebraeon_rings::structure::MetaFactoringMonoid;

#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct NNIterator {
    pub counter: Natural,
}

#[koto_impl]
impl NNIterator {
    // Empty implementation block to satisfy Koto requirements
}

#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct NN(pub Natural);

#[koto_impl]
impl NN {
    pub fn make_koto_object(n: KNumber) -> KObject {
        let my_int = Natural::from(u64::from(n));
        KObject::from(Self(my_int))
    }

    #[koto_method]
    pub fn bitcount(&self) -> KValue {
        KValue::from(self.0.bitcount())
    }

    #[koto_method]
    pub fn is_prime(&self) -> KValue {
        KValue::from(self.0.is_irreducible())
    }

    #[koto_method]
    pub fn factor(&self) -> KValue {
        match self.0.clone().factor().into_powers() {
            Some(factors) => {
                let koto_factors: Vec<KValue> = factors
                    .into_iter()
                    .map(|(prime, exp)| {
                        let prime_val = KValue::Object(KObject::from(NN(prime)));
                        let exp_val = KValue::Object(KObject::from(NN(exp)));
                        KValue::Tuple(vec![prime_val, exp_val].into())
                    })
                    .collect();

                KValue::List(KList::with_data(koto_factors.into()))
            }
            None => KValue::Null,
        }
    }

    #[koto_method]
    pub fn factorial(&self) -> KValue {
        KValue::Object(KObject::from(NN::from(NN(self.0.factorial()))))
    }

    #[koto_method]
    pub fn is_square(&self) -> KValue {
        self.0.is_square().into()
    }

    #[koto_method]
    pub fn sqrt_ceil(&self) -> KValue {
        KValue::Object(KObject::from(NN::from(NN(self.0.sqrt_ceil()))))
    }

    #[koto_method]
    pub fn sqrt_floor(&self) -> KValue {
        KValue::Object(KObject::from(NN::from(NN(self.0.sqrt_floor()))))
    }

    pub fn generator(_ctx: &mut CallContext) -> Result<KValue> {
        // Create a new iterator object
        let nn_iterator = NNIterator {
            counter: Natural::ZERO,
        };
        Ok(KValue::Object(KObject::from(nn_iterator)))
    }
}

impl KotoObject for NNIterator {
    fn is_iterable(&self) -> IsIterable {
        IsIterable::ForwardIterator
    }

    fn iterator_next(&mut self, _vm: &mut KotoVm) -> Option<KIteratorOutput> {
        let result = KValue::Object(KObject::from(NN(self.counter.clone())));
        self.counter += Natural::ONE;
        Some(KIteratorOutput::Value(result))
    }
}

impl KotoObject for NN {
    fn add(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() + other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("NN", unexpected),
        }
    }

    fn subtract(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() - other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("NN", unexpected),
        }
    }

    fn subtract_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 -= other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("NN", unexpected),
        }
    }

    fn multiply(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() * other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("NN", unexpected),
        }
    }

    fn add_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 += other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("NN", unexpected),
        }
    }

    fn multiply_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 *= other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("NN", unexpected),
        }
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

    fn less(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.0 < other.0)
            }
            unexpected => unexpected_type("Number", unexpected),
        }
    }

    fn less_or_equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.0 <= other.0)
            }
            unexpected => unexpected_type("Number", unexpected),
        }
    }

    fn greater(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.0 > other.0)
            }
            unexpected => unexpected_type("Number", unexpected),
        }
    }

    fn greater_or_equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.0 >= other.0)
            }
            unexpected => unexpected_type("Number", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(self.0.to_string());
        Ok(())
    }
}
