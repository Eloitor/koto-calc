use crate::NN::NN;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural};
use algebraeon::nzq::traits::DivMod;
use algebraeon::rings::natural::factorization::primes::{PrimalityTestResult, primality_test};
use algebraeon::sets::structure::EqSignature;
use algebraeon_rings::finite_fields::conway_finite_fields::ConwayFiniteFieldStructure;
use algebraeon_rings::polynomial::Polynomial;
use algebraeon_rings::structure::{
    AdditiveGroupSignature, AdditionSignature, CancellativeMultiplicationSignature,
    MultiplicativeMonoidSignature, MultiplicationSignature, OneSignature, TryReciprocalSignature,
    ZeroSignature,
};

/// Converts a koto value into an Integer (accepts i64 Number, NN and ZZ).
fn integer_from_value(value: &KValue) -> Result<Integer> {
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
fn natural_from_value(value: &KValue) -> Result<Natural> {
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

/// Human-readable name of the field: GF(7) or GF(2^3).
fn field_name(p: &Natural, k: &Natural) -> String {
    if *k == Natural::ONE {
        format!("GF({})", p)
    } else {
        format!("GF({}^{})", p, k)
    }
}

/// The algebraeon structure of GF(p^k) via the Conway polynomial, or a clear
/// error when the Conway polynomial database has no entry for (p, k).
fn conway_structure(p: &Natural, k: &Natural) -> Result<ConwayFiniteFieldStructure> {
    match ConwayFiniteFieldStructure::new(
        Integer::from(p.clone()).try_into().unwrap_or(usize::MAX),
        Integer::from(k.clone()).try_into().unwrap_or(usize::MAX),
    ) {
        Ok(s) => Ok(s),
        Err(()) => runtime_error!(
            "FF: no Conway polynomial for GF({}^{}) in the algebraeon database",
            p,
            k
        ),
    }
}

/// Extended Euclidean algorithm: the inverse of `a` modulo `p` (normalized
/// into [0, p)) or None when gcd(a, p) != 1. Assumes p > 0 and 0 <= a < p;
/// same algorithm as the ZZn inverse.
fn mod_inv(a: &Natural, p: &Natural) -> Option<Natural> {
    let mut r0 = Integer::from(a.clone());
    let m = Integer::from(p.clone());
    let mut r1 = m.clone();
    let mut s0 = Integer::ONE;
    let mut s1 = Integer::ZERO;
    while r1 != Integer::ZERO {
        let (q, r) = r0.clone().div_mod(r1.clone());
        r0 = r1;
        r1 = r;
        let s = s0.clone() - q * s1.clone();
        s0 = s1;
        s1 = s;
    }
    if r0 == Integer::ONE {
        // s0 is a Bezout coefficient of gcd(a, p) = 1: s0*a ≡ 1 (mod p).
        let (_q, r) = s0.div_mod(m);
        Some(r.try_into().unwrap())
    } else {
        None
    }
}

/// Modular exponentiation a^e mod p by binary exponentiation.
fn mod_pow(mut base: Natural, mut exp: Natural, p: &Natural) -> Natural {
    let mut result = Natural::ONE;
    base = base % p.clone();
    while exp != Natural::ZERO {
        if &exp % Natural::TWO == Natural::ONE {
            result = (result * &base) % p;
        }
        base = (&base * &base) % p;
        exp = exp / Natural::TWO;
    }
    result
}

/// Formats an element of GF(p^k) as a polynomial in descending degree order,
/// e.g. x^2 + x + 1, 2x + 1, x, 2 (coefficients are in [0, p)).
fn display_ext(p: &Polynomial<Integer>, p_char: &Natural) -> String {
    let mut terms: Vec<(usize, String)> = Vec::new();
    for (power, coeff) in (0..p.num_coeffs()).map(|i| p.coeff(i)).enumerate() {
        let c = coeff.into_owned();
        if c != Integer::ZERO {
            // Canonical representative in [0, p): algebraeon pot deixar
            // coeficients negatius (ex: -1) que han de mostrar-se com p-1.
            let (_q, r) = c.div_mod(Integer::from(p_char.clone()));
            terms.push((power, r.to_string()));
        }
    }
    if terms.is_empty() {
        return "0".into();
    }
    let mut out = String::new();
    for (i, (power, mag)) in terms.iter().rev().enumerate() {
        if i > 0 {
            out.push_str(" + ");
        }
        let omit_coeff = *power > 0 && mag == "1";
        if !omit_coeff {
            out.push_str(mag);
        }
        match power {
            0 => {}
            1 => out.push('x'),
            n => {
                out.push('x');
                out.push('^');
                out.push_str(&n.to_string());
            }
        }
    }
    out
}

/// A finite field GF(p) (prime field, k = 1) or GF(p^k) (extension via the
/// Conway polynomial, k > 1). FF(p) builds the prime field, FF(p, k) the
/// extension. Elements are obtained with `FF(p).of(x)` (x reduced mod p) or
/// `FF(p, k).of([c0, c1, ...])` (coefficients of the polynomial c0 + c1*x +
/// ..., each reduced mod p).
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct FF {
    p: Natural,
    k: Natural,
}

#[koto_impl]
impl FF {
    /// FF(p) -> GF(p) with p prime; FF(p, k) -> GF(p^k) via the Conway
    /// polynomial (requires an entry in algebraeon's Conway database).
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [p] => {
                let p = natural_from_value(p)?;
                Self::new_field(p, Natural::ONE)
            }
            [p, k] => {
                let p = natural_from_value(p)?;
                let k = natural_from_value(k)?;
                Self::new_field(p, k)
            }
            unexpected => unexpected_args("|Number| or |Number, Number|", unexpected),
        }
    }

    fn new_field(p: Natural, k: Natural) -> Result<KValue> {
        if k == Natural::ZERO {
            return runtime_error!("FF: the degree k must be positive");
        }
        match primality_test(&p) {
            PrimalityTestResult::Prime => {}
            _ => {
                return runtime_error!(
                    "FF: {} is not a prime number (GF(p^k) requires p prime)",
                    p
                )
            }
        }
        if k > Natural::ONE {
            // Validate now that the extension is available in the database.
            conway_structure(&p, &k)?;
        }
        Ok(KValue::Object(KObject::from(Self { p, k })))
    }

    /// The element x of the field. For GF(p), x can be Number/NN/ZZ and is
    /// reduced mod p. For GF(p^k), x can be a scalar (constant element) or a
    /// list of coefficients c0 + c1*x + ... (ascending degree), each reduced
    /// mod p; the polynomial is then reduced modulo the Conway polynomial.
    #[koto_method]
    pub fn of(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [x] => {
                if self.k == Natural::ONE {
                    let v = integer_from_value(x)?;
                    let (_q, r) = v.div_mod(Integer::from(self.p.clone()));
                    Ok(KValue::Object(KObject::from(FFEl {
                        p: self.p.clone(),
                        k: self.k.clone(),
                        value: FFValue::Prime(r.try_into().unwrap()),
                    })))
                } else {
                    let structure = conway_structure(&self.p, &self.k)?;
                    let poly = match x {
                        KValue::List(list) => {
                            let mut coeffs: Vec<Integer> = Vec::new();
                            for value in list.data().iter() {
                                let c = integer_from_value(value)?;
                                let (_q, r) = c.div_mod(Integer::from(self.p.clone()));
                                coeffs.push(r);
                            }
                            structure.reduce(Polynomial::from_coeffs(coeffs))
                        }
                        scalar => {
                            let c = integer_from_value(scalar)?;
                            let (_q, r) = c.div_mod(Integer::from(self.p.clone()));
                            Polynomial::from_coeffs(vec![r])
                        }
                    };
                    Ok(KValue::Object(KObject::from(FFEl {
                        p: self.p.clone(),
                        k: self.k.clone(),
                        value: FFValue::Ext(poly),
                    })))
                }
            }
            unexpected => unexpected_args("|Number| or |List|", unexpected),
        }
    }

    /// The characteristic p of the field (an NN).
    #[koto_method]
    pub fn char(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.p.clone())))
    }

    /// The degree k of the field over its prime field (1 for GF(p)).
    #[koto_method]
    pub fn degree(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.k.clone())))
    }
}

impl KotoObject for FF {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.p == other.p && self.k == other.k)
            }
            unexpected => unexpected_type("FF", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(field_name(&self.p, &self.k));
        Ok(())
    }
}

/// The value of a finite field element: an integer 0..p-1 in GF(p), or a
/// polynomial of degree < k (reduced mod the Conway polynomial) in GF(p^k).
#[derive(PartialEq, Clone, Eq, Debug)]
enum FFValue {
    Prime(Natural),
    Ext(Polynomial<Integer>),
}

/// An element of a finite field FF(p) / FF(p, k), created with
/// `FF(p).of(x)` or `FF(p, k).of([c0, c1, ...])`.
///
/// Display: for GF(p) the canonical representative 0..p-1; for GF(p^k) the
/// polynomial in descending degree order (e.g. x^2 + x + 1).
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct FFEl {
    p: Natural,
    k: Natural,
    value: FFValue,
}

#[koto_impl]
impl FFEl {
    /// The characteristic p of the field (an NN).
    #[koto_method]
    pub fn char(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.p.clone())))
    }

    /// The coefficients of the element (ascending degree, each in [0, p)).
    /// For GF(p) this is the singleton [x].
    #[koto_method]
    pub fn coeffs(&self) -> KValue {
        let values: Vec<KValue> = match &self.value {
            FFValue::Prime(a) => vec![KValue::Object(KObject::from(NN(a.clone())))],
            FFValue::Ext(p) => (0..p.num_coeffs())
                .map(|i| {
                    // Reduir mod p: els coeficients poden ser negatius.
                    let c = p.coeff(i).into_owned();
                    let (_q, r) = c.div_mod(Integer::from(self.p.clone()));
                    KValue::Object(KObject::from(NN(r.try_into().unwrap())))
                })
                .collect(),
        };
        KValue::List(KList::with_data(values.into()))
    }

    /// The multiplicative inverse of the element (error when it is zero).
    #[koto_method]
    pub fn inverse(&self) -> Result<KValue> {
        let value = match &self.value {
            FFValue::Prime(a) => {
                if a == &Natural::ZERO {
                    return runtime_error!("FF: 0 has no multiplicative inverse");
                }
                FFValue::Prime(mod_inv(a, &self.p).unwrap())
            }
            FFValue::Ext(a) => {
                let structure = conway_structure(&self.p, &self.k)?;
                match structure.try_reciprocal(a) {
                    Some(inv) => FFValue::Ext(inv),
                    None => return runtime_error!("FF: 0 has no multiplicative inverse"),
                }
            }
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    /// The multiplicative order of the element (error when it is zero):
    /// the smallest positive n with self^n = 1.
    #[koto_method]
    pub fn order(&self) -> Result<KValue> {
        let ord = match &self.value {
            FFValue::Prime(a) => {
                if a == &Natural::ZERO {
                    return runtime_error!("FF: 0 has no multiplicative order");
                }
                let mut cur = a.clone();
                let mut ord = Natural::ONE;
                while cur != Natural::ONE {
                    cur = (cur * a) % &self.p;
                    ord += Natural::ONE;
                }
                ord
            }
            FFValue::Ext(a) => {
                let structure = conway_structure(&self.p, &self.k)?;
                if structure.equal(a, &structure.zero()) {
                    return runtime_error!("FF: 0 has no multiplicative order");
                }
                let mut cur = a.clone();
                let mut ord = Natural::ONE;
                while !structure.equal(&cur, &structure.one()) {
                    cur = structure.mul(&cur, a);
                    ord += Natural::ONE;
                }
                ord
            }
        };
        Ok(KValue::Object(KObject::from(NN(ord))))
    }

    /// self^n: n can be any integer (Number/NN/ZZ). Negative exponents use
    /// the multiplicative inverse (error when the element is zero).
    #[koto_method]
    pub fn pow(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [n] => {
                let exp = integer_from_value(n)?;
                Ok(KValue::Object(KObject::from(self.pow_el(&exp)?)))
            }
            unexpected => unexpected_args("|Number|", unexpected),
        }
    }

    fn pow_el(&self, exp: &Integer) -> Result<FFEl> {
        if *exp >= Integer::ZERO {
            let exp_nat = Natural::try_from(exp.clone()).unwrap();
            let value = match &self.value {
                FFValue::Prime(a) => FFValue::Prime(mod_pow(a.clone(), exp_nat, &self.p)),
                FFValue::Ext(a) => FFValue::Ext(conway_structure(&self.p, &self.k)?.nat_pow(a, &exp_nat)),
            };
            Ok(FFEl {
                p: self.p.clone(),
                k: self.k.clone(),
                value,
            })
        } else {
            // Negative exponent: invert first, then raise to |exp|.
            let inv = self.inverse()?;
            let inv = match inv {
                KValue::Object(o) => o.cast::<Self>().unwrap().clone(),
                _ => unreachable!(),
            };
            inv.pow_el(&(-exp.clone()))
        }
    }

    /// The algebraeon structure of the field, or an error when the Conway
    /// polynomial is missing from the database.
    fn conway(&self) -> Result<ConwayFiniteFieldStructure> {
        conway_structure(&self.p, &self.k)
    }

    /// Errors unless both elements belong to the same field.
    fn check_same_field(&self, other: &FFEl) -> Result<()> {
        if self.p != other.p || self.k != other.k {
            return runtime_error!(
                "FF: cannot operate on elements from different fields ({} and {})",
                field_name(&self.p, &self.k),
                field_name(&other.p, &other.k)
            );
        }
        Ok(())
    }
}

impl KotoObject for FFEl {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                if self.p != other.p || self.k != other.k {
                    return Ok(false);
                }
                Ok(match (&self.value, &other.value) {
                    // Igualtat canonica mod p: les representacions internes
                    // poden diferir (ex: -1 vs 1 a GF(2)) tot i ser iguals.
                    (FFValue::Prime(a), FFValue::Prime(b)) => a == b,
                    (FFValue::Ext(a), FFValue::Ext(b)) => {
                        self.conway()?.equal(a, b)
                    }
                    _ => unreachable!(
                        "elements of the same field share the same representation"
                    ),
                })
            }
            unexpected => unexpected_type("FFEl", unexpected),
        }
    }

    fn add(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_field(&other)?;
        let value = match (&self.value, &other.value) {
            (FFValue::Prime(a), FFValue::Prime(b)) => FFValue::Prime((a + b) % &self.p),
            (FFValue::Ext(a), FFValue::Ext(b)) => FFValue::Ext(self.conway()?.add(a, b)),
            _ => unreachable!("elements of the same field share the same representation"),
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_field(&other)?;
        let value = match (&self.value, &other.value) {
            (FFValue::Prime(a), FFValue::Prime(b)) => {
                // a, b in [0, p) so p + a - b is positive.
                FFValue::Prime((&self.p + a - b) % &self.p)
            }
            (FFValue::Ext(a), FFValue::Ext(b)) => FFValue::Ext(self.conway()?.sub(a, b)),
            _ => unreachable!("elements of the same field share the same representation"),
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_field(&other)?;
        let value = match (&self.value, &other.value) {
            (FFValue::Prime(a), FFValue::Prime(b)) => FFValue::Prime((a * b) % &self.p),
            (FFValue::Ext(a), FFValue::Ext(b)) => FFValue::Ext(self.conway()?.mul(a, b)),
            _ => unreachable!("elements of the same field share the same representation"),
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    fn divide(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_field(&other)?;
        let value = match (&self.value, &other.value) {
            (FFValue::Prime(a), FFValue::Prime(b)) => {
                if b == &Natural::ZERO {
                    return runtime_error!("FF: division by zero");
                }
                FFValue::Prime((a * mod_inv(b, &self.p).unwrap()) % &self.p)
            }
            (FFValue::Ext(a), FFValue::Ext(b)) => {
                let structure = self.conway()?;
                match structure.try_divide(a, b) {
                    Some(q) => FFValue::Ext(q),
                    None => return runtime_error!("FF: division by zero"),
                }
            }
            _ => unreachable!("elements of the same field share the same representation"),
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    fn negate(&self) -> Result<KValue> {
        let value = match &self.value {
            FFValue::Prime(a) => {
                if a == &Natural::ZERO {
                    FFValue::Prime(Natural::ZERO)
                } else {
                    FFValue::Prime(&self.p - a)
                }
            }
            FFValue::Ext(a) => FFValue::Ext(self.conway()?.neg(a)),
        };
        Ok(KValue::Object(KObject::from(FFEl {
            p: self.p.clone(),
            k: self.k.clone(),
            value,
        })))
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        let s = match &self.value {
            FFValue::Prime(a) => a.to_string(),
            FFValue::Ext(p) => display_ext(p, &self.p),
        };
        ctx.append(s);
        Ok(())
    }
}

/// Extracts an FFEl from a KValue, or errors with a type error.
fn cast_element(other: &KValue) -> Result<FFEl> {
    match other {
        KValue::Object(o) if o.is_a::<FFEl>() => Ok(o.cast::<FFEl>().unwrap().clone()),
        unexpected => unexpected_type("FFEl", unexpected),
    }
}
