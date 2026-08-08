use koto_runtime::{IsIterable, KIteratorOutput, KotoVm, Result, derive::*, prelude::*};

use algebraeon::nzq::Natural;
use algebraeon::nzq::combinatorics::{stirling_number1_signed, stirling_number2};
use algebraeon::nzq::primes;
use algebraeon::nzq::traits::Abs;
use crate::Perm::usize_from_value;
use algebraeon::rings::natural::factorization::primes::{PrimalityTestResult, primality_test};
use algebraeon::rings::natural::functions::{IsPowerTestResult, is_power_test};
use algebraeon_rings::structure::{MetaFactoringMonoid, MetaFactoringMonoidNaturalExponent, UniqueFactorizationMonoidSignature};

#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
#[koto(type_name = "NIterator")]
pub struct NNIterator {
    pub counter: Natural,
}

#[koto_impl]
impl NNIterator {
    // Empty implementation block to satisfy Koto requirements
}

/// Infinite iterator over the prime numbers (2, 3, 5, 7, ...)
#[derive(KotoType)]
#[koto(type_name = "NPrimesIterator")]
pub struct NNPrimesIterator {
    iter: Box<dyn Iterator<Item = usize>>,
}

impl NNPrimesIterator {
    pub fn new() -> Self {
        Self {
            iter: Box::new(primes()),
        }
    }
}

// The underlying iterator is not cloneable, so copies start a fresh prime iterator.
impl KotoCopy for NNPrimesIterator {
    fn copy(&self) -> KObject {
        KObject::from(NNPrimesIterator::new())
    }

    fn deep_copy(&self) -> KObject {
        self.copy()
    }
}

#[koto_impl]
impl NNPrimesIterator {
    // Empty implementation block to satisfy Koto requirements
}

#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
#[koto(type_name = "N")]
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
    pub fn is_squarefree(&self) -> KValue {
        KValue::from(self.0.is_squarefree())
    }

    #[koto_method]
    pub fn divisors(&self) -> KValue {
        let factored = self.0.clone().factor();
        let factorizations = Natural::structure_ref().factorizations();
        match factorizations.divisors(&factored) {
            Some(divisors) => {
                let mut divisors: Vec<Natural> = divisors.collect();
                divisors.sort();
                let koto_divisors: Vec<KValue> = divisors
                    .into_iter()
                    .map(|d| KValue::Object(KObject::from(NN(d))))
                    .collect();
                KValue::List(KList::with_data(koto_divisors.into()))
            }
            None => KValue::List(KList::with_data(vec![].into())),
        }
    }

    #[koto_method]
    pub fn euler_totient(&self) -> KValue {
        let factored = self.0.clone().factor();
        let phi = Natural::structure_ref()
            .factorizations()
            .euler_totient(&factored);
        KValue::Object(KObject::from(NN(phi)))
    }

    #[koto_method]
    pub fn is_power_test(&self) -> KValue {
        match is_power_test(&self.0) {
            IsPowerTestResult::Power(base, exp) => KValue::Tuple(
                vec![
                    KValue::Bool(true),
                    KValue::Object(KObject::from(NN(base))),
                    KValue::Object(KObject::from(NN(Natural::from(exp)))),
                ]
                .into(),
            ),
            IsPowerTestResult::Zero | IsPowerTestResult::One | IsPowerTestResult::No => {
                KValue::Tuple(
                    vec![
                        KValue::Bool(false),
                        KValue::Null,
                        KValue::Null,
                    ]
                    .into(),
                )
            }
        }
    }

    #[koto_method]
    pub fn primality_test(&self) -> KValue {
        match primality_test(&self.0) {
            PrimalityTestResult::Prime => KValue::from("prime"),
            PrimalityTestResult::Zero
            | PrimalityTestResult::One
            | PrimalityTestResult::Composite => KValue::from("composite"),
        }
    }

    /// Stirling number of the first kind (unsigned): NN(4).stirling1(NN(2)) = NN(11).
    /// Errors if k > n.
    #[koto_method]
    pub fn stirling1(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [k] => {
                let k = usize_from_value(k)?;
                let n: usize = self
                    .0
                    .clone()
                    .try_into()
                    .map_err(|_| koto_runtime::Error::from("number too large"))?;
                if k > n {
                    return runtime_error!("N.stirling1: k must be <= n (got k={}, n={})", k, n);
                }
                let s = stirling_number1_signed(n, k)
                    .map_err(|_| koto_runtime::Error::from("invalid Stirling input"))?
                    .abs();
                Ok(KValue::Object(KObject::from(NN(s))))
            }
            unexpected => unexpected_args("|N|", unexpected),
        }
    }

    /// Stirling number of the second kind: NN(5).stirling2(NN(2)) = NN(15).
    /// Errors if k > n.
    #[koto_method]
    pub fn stirling2(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [k] => {
                let k = usize_from_value(k)?;
                let n: usize = self
                    .0
                    .clone()
                    .try_into()
                    .map_err(|_| koto_runtime::Error::from("number too large"))?;
                if k > n {
                    return runtime_error!("N.stirling2: k must be <= n (got k={}, n={})", k, n);
                }
                let s = stirling_number2(n, k)
                    .map_err(|_| koto_runtime::Error::from("invalid Stirling input"))?;
                Ok(KValue::Object(KObject::from(NN(s))))
            }
            unexpected => unexpected_args("|N|", unexpected),
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

impl KotoObject for NNPrimesIterator {
    fn is_iterable(&self) -> IsIterable {
        IsIterable::ForwardIterator
    }

    fn iterator_next(&mut self, _vm: &mut KotoVm) -> Option<KIteratorOutput> {
        match self.iter.next() {
            Some(p) => Some(KIteratorOutput::Value(KValue::Object(KObject::from(
                NN(Natural::from(p)),
            )))),
            None => None,
        }
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
            unexpected => unexpected_type("N natural", unexpected),
        }
    }

    fn subtract(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() - other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("N natural", unexpected),
        }
    }

    fn subtract_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 -= other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("N natural", unexpected),
        }
    }

    fn multiply(&self, rhs: &KValue) -> Result<KValue> {
        match rhs {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let result = self.0.clone() * other.0.clone();
                Ok(KValue::Object(KObject::from(Self(result))))
            }
            unexpected => unexpected_type("N natural", unexpected),
        }
    }

    fn add_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 += other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("N natural", unexpected),
        }
    }

    fn multiply_assign(&mut self, other: &KValue) -> Result<()> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                self.0 *= other.0.clone();
                Ok(())
            }
            unexpected => unexpected_type("N natural", unexpected),
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
