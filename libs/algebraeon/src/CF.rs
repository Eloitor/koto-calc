use crate::NN::NN;
use crate::Q::Q;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::DivMod;
use algebraeon_rings::continued_fraction::{
    IrrationalSimpleContinuedFraction, PeriodicSimpleContinuedFraction, SimpleContinuedFraction,
};

/// Converts a koto value into an Integer (accepts i64 Number, NN and ZZ).
pub(crate) fn integer_from_value(value: &KValue) -> Result<Integer> {
    match value {
        KValue::Number(n) if n.is_i64() => Ok(Integer::from(i64::from(*n))),
        KValue::Object(object) => {
            if let Ok(nn) = object.cast::<NN>() {
                Ok(Integer::from(nn.0.clone()))
            } else if let Ok(zz) = object.cast::<ZZ>() {
                Ok(zz.to_integer())
            } else {
                unexpected_type("Number, NN, or ZZ", value)
            }
        }
        unexpected => unexpected_type("Number, NN, or ZZ", unexpected),
    }
}

/// Converts a koto value into a non-negative Natural (accepts i64 Number, NN
/// and ZZ; negative values are rejected).
pub(crate) fn natural_from_value(value: &KValue) -> Result<Natural> {
    match value {
        KValue::Number(n) if n.is_i64() => {
            let v = i64::from(*n);
            if v < 0 {
                return runtime_error!("expected a non-negative integer, got {}", v);
            }
            Ok(Natural::from(v as u64))
        }
        KValue::Object(object) => {
            if let Ok(nn) = object.cast::<NN>() {
                Ok(nn.0.clone())
            } else if let Ok(zz) = object.cast::<ZZ>() {
                let i = zz.to_integer();
                if i < Integer::ZERO {
                    return runtime_error!("expected a non-negative integer, got {}", i);
                }
                Ok(Natural::try_from(i).unwrap())
            } else {
                unexpected_type("Number, NN, or ZZ", value)
            }
        }
        unexpected => unexpected_type("Number, NN, or ZZ", unexpected),
    }
}

/// The kind of coefficients backing a [`CF`]: a finite list, a periodic list
/// (initial part followed by a repeating part) or an infinite irrational
/// generator (e.g. `eulers_constant()`).
#[derive(Clone, Debug)]
enum CfKind {
    Finite(Vec<Integer>),
    Periodic {
        initial: Vec<Integer>,
        repeats: Vec<Integer>,
    },
    Irrational(IrrationalSimpleContinuedFraction),
}

/// A simple continued fraction `[a0, a1, a2, ...]` (all coefficients after the
/// first are positive). Built with `CF([a0, a1, ...])` (finite), with
/// `CF.periodic(initial, repeats)` (periodic, e.g. sqrt(2) = [1; 2, 2, ...])
/// or obtained from `eulers_constant()` (infinite).
///
/// Convergents are computed with the standard recurrence
/// p_{-2}=0, p_{-1}=1, q_{-2}=1, q_{-1}=0, p_n = a_n p_{n-1} + p_{n-2}.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct CF {
    kind: CfKind,
}

impl PartialEq for CF {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (CfKind::Finite(a), CfKind::Finite(b)) => a == b,
            (
                CfKind::Periodic {
                    initial: ai,
                    repeats: ar,
                },
                CfKind::Periodic {
                    initial: bi,
                    repeats: br,
                },
            ) => ai == bi && ar == br,
            _ => false,
        }
    }
}

impl Eq for CF {}

#[koto_impl]
impl CF {
    /// CF([a0, a1, ...]) builds a finite simple continued fraction. Coeffi
    /// cients after the first must be positive.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::List(list)] => {
                let mut coeffs = Vec::new();
                for value in list.data().iter() {
                    coeffs.push(integer_from_value(value)?);
                }
                Self::validate_coeffs(&coeffs)?;
                Ok(KValue::Object(KObject::from(Self {
                    kind: CfKind::Finite(coeffs),
                })))
            }
            unexpected => unexpected_args("|List|", unexpected),
        }
    }

    fn validate_coeffs(coeffs: &[Integer]) -> Result<()> {
        for (i, c) in coeffs.iter().enumerate().skip(1) {
            if *c < Integer::ONE {
                return runtime_error!(
                    "CF: coefficients after the first must be positive, got {} at index {}",
                    c,
                    i
                );
            }
        }
        Ok(())
    }

    /// CF.periodic(initial, repeats) builds a periodic continued fraction,
    /// e.g. CF.periodic([1], [2]) = [1; 2, 2, 2, ...] = sqrt(2). Coefficients
    /// after the first and all repeating coefficients must be positive.
    pub fn periodic(initial: &KList, repeats: &KList) -> Result<KValue> {
        let mut init = Vec::new();
        for value in initial.data().iter() {
            init.push(integer_from_value(value)?);
        }
        let mut rep = Vec::new();
        for value in repeats.data().iter() {
            rep.push(integer_from_value(value)?);
        }
        // Validate through algebraeon's constructor (rejects empty repeats and
        // non-positive coefficients after the first).
        match PeriodicSimpleContinuedFraction::new(init.clone(), rep.clone()) {
            Ok(_) => {}
            Err(()) => {
                return runtime_error!(
                    "CF.periodic: the repeating part must be non-empty and all coefficients \
                     after the first must be positive"
                )
            }
        }
        Ok(KValue::Object(KObject::from(Self {
            kind: CfKind::Periodic {
                initial: init,
                repeats: rep,
            },
        })))
    }

    /// Builds the (finite) continued fraction of a rational number using the
    /// Euclidean algorithm. `den` must be positive (as guaranteed by Q's
    /// reduced form). Used by Q.to_cf().
    pub fn from_rational(num: Integer, den: Natural) -> Self {
        let mut coeffs = Vec::new();
        let mut n = num;
        let mut d = Integer::from(den);
        loop {
            // d > 0, so div_mod returns q = floor(n/d) with 0 <= r < d.
            let (q, r) = n.div_mod(d.clone());
            coeffs.push(q);
            if r == Integer::ZERO {
                break;
            }
            n = d;
            d = r;
        }
        Self {
            kind: CfKind::Finite(coeffs),
        }
    }

    /// The coefficient a_n (None when a finite CF has fewer than n+1 terms).
    fn coeff(&self, n: usize) -> Option<Integer> {
        match &self.kind {
            CfKind::Finite(coeffs) => coeffs.get(n).cloned(),
            CfKind::Periodic { initial, repeats } => {
                if let Some(c) = initial.get(n) {
                    Some(c.clone())
                } else {
                    Some(repeats[(n - initial.len()) % repeats.len()].clone())
                }
            }
            CfKind::Irrational(scf) => scf.coeff(n).map(|c| c.into_owned()),
        }
    }

    /// The exact rational value of a finite continued fraction
    /// (a0 + 1/(a1 + 1/(... + 1/ak))). Errors for periodic or infinite CFs.
    #[koto_method]
    pub fn value(&self) -> Result<KValue> {
        match self.to_rational() {
            Some(rational) => Ok(KValue::Object(KObject::from(Q(rational)))),
            None => runtime_error!("CF.value() is only defined for finite continued fractions"),
        }
    }

    fn to_rational(&self) -> Option<Rational> {
        match &self.kind {
            CfKind::Finite(coeffs) => {
                let mut value = Rational::from(coeffs.last().unwrap().clone());
                for a in coeffs[..coeffs.len() - 1].iter().rev() {
                    value = Rational::from(a.clone()) + Rational::ONE / value;
                }
                Some(value)
            }
            _ => None,
        }
    }

    /// The n-th convergent p_n/q_n (an approximation of the value). For
    /// periodic and infinite CFs any n works; for finite CFs n must not
    /// exceed the number of coefficients minus one.
    #[koto_method]
    pub fn convergent(&self, args: &[KValue]) -> Result<KValue> {
        let n = match args {
            [KValue::Number(n)] if n.is_i64() && i64::from(*n) >= 0 => i64::from(*n) as usize,
            unexpected => return unexpected_args("|NN|", unexpected),
        };
        let mut p_prev2 = Integer::ZERO; // p_{-2}
        let mut p_prev1 = Integer::ONE; // p_{-1}
        let mut q_prev2 = Integer::ONE; // q_{-2}
        let mut q_prev1 = Integer::ZERO; // q_{-1}
        for i in 0..=n {
            let a = match self.coeff(i) {
                Some(a) => a,
                None => {
                    return runtime_error!(
                        "CF.convergent({}): the continued fraction has only {} coefficients",
                        n,
                        i
                    )
                }
            };
            let p = &a * &p_prev1 + &p_prev2;
            let q = &a * &q_prev1 + &q_prev2;
            p_prev2 = p_prev1;
            p_prev1 = p;
            q_prev2 = q_prev1;
            q_prev1 = q;
        }
        Ok(KValue::Object(KObject::from(Q(Rational::from_integers(
            p_prev1, q_prev1,
        )))))
    }

    /// The first n coefficients as a list of ZZ values (stops early for
    /// finite CFs).
    #[koto_method]
    pub fn take(&self, args: &[KValue]) -> Result<KValue> {
        let n = match args {
            [KValue::Number(n)] if n.is_i64() && i64::from(*n) >= 0 => i64::from(*n) as usize,
            unexpected => return unexpected_args("|NN|", unexpected),
        };
        let mut vals = Vec::new();
        for i in 0..n {
            match self.coeff(i) {
                Some(c) => vals.push(KValue::Object(KObject::from(ZZ::from_integer(c)))),
                None => break,
            }
        }
        Ok(KValue::List(KList::with_data(vals.into())))
    }

    /// A floating point approximation of the value: exact for finite CFs,
    /// the 30th convergent otherwise.
    #[koto_method]
    pub fn to_float(&self) -> Result<KValue> {
        let rational = match self.to_rational() {
            Some(rational) => rational,
            None => {
                let c = self.convergent(&[KValue::Number(30.0.into())])?;
                match c {
                    KValue::Object(object) => object.cast::<Q>().unwrap().0.clone(),
                    _ => unreachable!(),
                }
            }
        };
        Ok(KValue::from(f64::from(&rational)))
    }

    /// The infinite continued fraction of Euler's constant e, produced by
    /// algebraeon: e = [2; 1, 2, 1, 1, 4, 1, 1, 6, ...].
    pub fn eulers_constant() -> Self {
        Self {
            kind: CfKind::Irrational(algebraeon_rings::continued_fraction::eulers_constant()),
        }
    }
}

impl KotoObject for CF {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(*self == *other)
            }
            unexpected => unexpected_type("CF", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        match &self.kind {
            CfKind::Finite(coeffs) => {
                let parts: Vec<String> = coeffs.iter().map(|c| c.to_string()).collect();
                ctx.append(format!("[{}]", parts.join(", ")));
            }
            CfKind::Periodic { initial, repeats } => {
                let init: Vec<String> = initial.iter().map(|c| c.to_string()).collect();
                let rep: Vec<String> = repeats.iter().map(|c| c.to_string()).collect();
                ctx.append(format!("[{}; {}]", init.join(", "), rep.join(", ")));
            }
            CfKind::Irrational(_) => {
                let mut parts: Vec<String> = Vec::new();
                for i in 0..7 {
                    match self.coeff(i) {
                        Some(c) => parts.push(c.to_string()),
                        None => break,
                    }
                }
                ctx.append(format!("[{}, ...]", parts.join(", ")));
            }
        }
        Ok(())
    }
}
