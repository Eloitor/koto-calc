use crate::NN::NN;
use crate::Q::Q;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::{Abs, Fraction};
use algebraeon::sets::structure::{EqSignature, MetaType};
use algebraeon_rings::polynomial::{Polynomial, ToPolynomialSignature};
use algebraeon_rings::structure::{
    AdditiveGroupSignature, AdditionSignature, GreatestCommonDivisorSignature,
    MetaFactoringMonoid, MultiplicationSignature,
};

/// A coefficient field for a univariate polynomial: either ZZ (all
/// coefficients integers) or Q (rational coefficients). The field is chosen
/// automatically at construction: ZZ if every coefficient is an integer,
/// Q if any coefficient is a fraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolyCoeff {
    ZZ(Polynomial<Integer>),
    QQ(Polynomial<Rational>),
}

/// A univariate polynomial over ZZ or Q.
///
/// Coefficients are given from degree 0 upwards: `Poly([6, -5, 1])` is
/// `6 - 5x + x^2` (the independent term first).
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct Poly {
    pub poly: PolyCoeff,
}

/// A scalar that can be used as a coefficient: either an integer or a rational.
enum Scalar {
    Int(Integer),
    Rat(Rational),
}

fn scalar_from_value(value: &KValue) -> Result<Scalar> {
    match value {
        KValue::Number(n) => {
            if n.is_i64() {
                Ok(Scalar::Int(Integer::from(i64::from(*n))))
            } else {
                match Rational::try_from_float_simplest(f64::from(*n)) {
                    Ok(r) => Ok(Scalar::Rat(r)),
                    Err(()) => runtime_error!("cannot convert {} to a polynomial coefficient", n),
                }
            }
        }
        KValue::Object(object) => {
            if let Ok(nn) = object.cast::<NN>() {
                Ok(Scalar::Int(Integer::from(nn.0.clone())))
            } else if let Ok(zz) = object.cast::<ZZ>() {
                Ok(Scalar::Int(zz.to_integer()))
            } else if let Ok(q) = object.cast::<Q>() {
                Ok(Scalar::Rat(q.0.clone()))
            } else {
                unexpected_type("Number, NN, ZZ, or Q", value)
            }
        }
        unexpected => unexpected_type("Number, NN, ZZ, or Q", unexpected),
    }
}

fn to_qq(p: &Polynomial<Integer>) -> Polynomial<Rational> {
    p.apply_map(|c| Rational::from(c.clone()))
}

/// Brings two polynomials to a common coefficient field (ZZ is promoted to Q
/// when the two fields differ).
fn promote_pair(a: &PolyCoeff, b: &PolyCoeff) -> (PolyCoeff, PolyCoeff) {
    match (a, b) {
        (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => (PolyCoeff::ZZ(a.clone()), PolyCoeff::ZZ(b.clone())),
        (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => (PolyCoeff::QQ(a.clone()), PolyCoeff::QQ(b.clone())),
        (PolyCoeff::ZZ(a), PolyCoeff::QQ(b)) => (PolyCoeff::QQ(to_qq(a)), PolyCoeff::QQ(b.clone())),
        (PolyCoeff::QQ(a), PolyCoeff::ZZ(b)) => (PolyCoeff::QQ(a.clone()), PolyCoeff::QQ(to_qq(b))),
    }
}

/// Makes a rational polynomial monic (divides by the leading coefficient).
/// The zero polynomial is returned unchanged.
fn monic_qq(p: Polynomial<Rational>) -> Polynomial<Rational> {
    let structure = Rational::structure();
    let s = structure.polynomials();
    match s.leading_coeff(&p) {
        Some(lc) if lc == &Rational::ONE => p,
        Some(lc) => {
            let inv = Rational::ONE / lc.clone();
            s.mul_scalar(&p, &inv)
        }
        None => p,
    }
}

/// Formats a list of (power, magnitude, negative) terms in ascending degree
/// (constant term first), matching the coefficient convention of `Poly`.
/// The magnitude is the positive coefficient string ("1" is omitted for
/// non-constant terms); the variable is `x`.
fn format_terms(terms: Vec<(usize, String, bool)>) -> String {
    if terms.is_empty() {
        return "0".into();
    }
    let mut out = String::new();
    for (i, (power, mag, negative)) in terms.iter().enumerate() {
        if i > 0 {
            out.push_str(if *negative { " - " } else { " + " });
        } else if *negative {
            out.push('-');
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

fn display_zz(p: &Polynomial<Integer>) -> String {
    let mut terms: Vec<(usize, String, bool)> = Vec::new();
    for (power, coeff) in (0..p.num_coeffs()).map(|i| p.coeff(i)).enumerate() {
        if coeff.as_ref() != &Integer::ZERO {
            let coeff = coeff.into_owned();
            let coeff_abs = coeff.clone().abs();
            terms.push((power, coeff_abs.to_string(), coeff < Integer::ZERO));
        }
    }
    format_terms(terms)
}

fn display_qq(p: &Polynomial<Rational>) -> String {
    let mut terms: Vec<(usize, String, bool)> = Vec::new();
    for (power, coeff) in (0..p.num_coeffs()).map(|i| p.coeff(i)).enumerate() {
        if coeff.as_ref() != &Rational::ZERO {
            let (num, den) = coeff.as_ref().numerator_and_denominator();
            let negative = num < Integer::ZERO;
            let num_abs = num.abs();
            let mag = if den == Natural::ONE {
                num_abs.to_string()
            } else {
                format!("({}/{})", num_abs, den)
            };
            terms.push((power, mag, negative));
        }
    }
    format_terms(terms)
}

#[koto_impl]
impl Poly {
    /// Builds a Poly from a Koto list of coefficients (degree 0 upwards).
    /// Chooses ZZ if all coefficients are integers, Q otherwise.
    pub fn from_koto_list(list: &KList) -> Result<KObject> {
        let mut coeffs: Vec<Scalar> = Vec::new();
        let mut any_rational = false;
        for value in list.data().iter() {
            let scalar = scalar_from_value(value)?;
            if matches!(scalar, Scalar::Rat(_)) {
                any_rational = true;
            }
            coeffs.push(scalar);
        }
        let poly = if any_rational {
            let rats: Vec<Rational> = coeffs
                .into_iter()
                .map(|s| match s {
                    Scalar::Int(i) => Rational::from(i),
                    Scalar::Rat(r) => r,
                })
                .collect();
            PolyCoeff::QQ(
                Rational::structure()
                    .polynomials()
                    .reduce_poly(Polynomial::from_coeffs(rats)),
            )
        } else {
            let ints: Vec<Integer> = coeffs
                .into_iter()
                .map(|s| match s {
                    Scalar::Int(i) => i,
                    Scalar::Rat(_) => unreachable!(),
                })
                .collect();
            PolyCoeff::ZZ(
                Integer::structure()
                    .polynomials()
                    .reduce_poly(Polynomial::from_coeffs(ints)),
            )
        };
        Ok(KObject::from(Self { poly }))
    }

    #[koto_method]
    pub fn degree(&self) -> KValue {
        let degree = match &self.poly {
            PolyCoeff::ZZ(p) => p.degree(),
            PolyCoeff::QQ(p) => p.degree(),
        };
        // The zero polynomial has degree 0 by convention.
        KValue::Object(KObject::from(NN(Natural::from(degree.unwrap_or(0)))))
    }

    #[koto_method]
    pub fn coeffs(&self) -> KValue {
        let values: Vec<KValue> = match &self.poly {
            PolyCoeff::ZZ(p) => (0..p.num_coeffs())
                .map(|i| {
                    KValue::Object(KObject::from(ZZ::from_integer(
                        p.coeff(i).into_owned(),
                    )))
                })
                .collect(),
            PolyCoeff::QQ(p) => (0..p.num_coeffs())
                .map(|i| KValue::Object(KObject::from(Q(p.coeff(i).into_owned()))))
                .collect(),
        };
        KValue::List(KList::with_data(values.into()))
    }

    #[koto_method]
    pub fn eval(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [x] => {
                let scalar = scalar_from_value(x)?;
                match (&self.poly, &scalar) {
                    (PolyCoeff::ZZ(p), Scalar::Int(v)) => {
                        let result = Integer::structure().polynomials().evaluate(p, v);
                        Ok(KValue::Object(KObject::from(ZZ::from_integer(result))))
                    }
                    (PolyCoeff::ZZ(p), Scalar::Rat(v)) => {
                        let qp = to_qq(p);
                        let result = Rational::structure().polynomials().evaluate(&qp, v);
                        Ok(KValue::Object(KObject::from(Q(result))))
                    }
                    (PolyCoeff::QQ(p), Scalar::Int(v)) => {
                        let rv = Rational::from(v.clone());
                        let result = Rational::structure().polynomials().evaluate(p, &rv);
                        Ok(KValue::Object(KObject::from(Q(result))))
                    }
                    (PolyCoeff::QQ(p), Scalar::Rat(v)) => {
                        let result = Rational::structure().polynomials().evaluate(p, v);
                        Ok(KValue::Object(KObject::from(Q(result))))
                    }
                }
            }
            unexpected => unexpected_args("|x|", unexpected),
        }
    }

    #[koto_method]
    pub fn derivative(&self) -> KValue {
        let poly = match &self.poly {
            PolyCoeff::ZZ(p) => PolyCoeff::ZZ(
                Integer::structure()
                    .polynomials()
                    .derivative(p.clone()),
            ),
            PolyCoeff::QQ(p) => PolyCoeff::QQ(
                Rational::structure()
                    .polynomials()
                    .derivative(p.clone()),
            ),
        };
        KValue::Object(KObject::from(Self { poly }))
    }

    #[koto_method]
    pub fn gcd(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::Object(other)] if other.is_a::<Poly>() => {
                let other = other.cast::<Poly>().unwrap();
                match (&self.poly, &other.poly) {
                    (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => {
                        let g = Integer::structure().polynomials().gcd(a, b);
                        match g.leading_coeff() {
                            Some(lc) if lc == Integer::ONE => {
                                Ok(KValue::Object(KObject::from(Self {
                                    poly: PolyCoeff::ZZ(g),
                                })))
                            }
                            Some(lc) if lc == -Integer::ONE => {
                                let structure = Integer::structure();
                                let s = structure.polynomials();
                                Ok(KValue::Object(KObject::from(Self {
                                    poly: PolyCoeff::ZZ(s.neg(&g)),
                                })))
                            }
                            _ => {
                                // Not monic over ZZ: promote to Q and normalize.
                                let qg = monic_qq(to_qq(&g));
                                Ok(KValue::Object(KObject::from(Self {
                                    poly: PolyCoeff::QQ(qg),
                                })))
                            }
                        }
                    }
                    (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => {
                        let g = Rational::structure().polynomials().gcd(a, b);
                        Ok(KValue::Object(KObject::from(Self {
                            poly: PolyCoeff::QQ(monic_qq(g)),
                        })))
                    }
                    (PolyCoeff::ZZ(a), PolyCoeff::QQ(b)) => {
                        let qa = to_qq(a);
                        let g = Rational::structure().polynomials().gcd(&qa, b);
                        Ok(KValue::Object(KObject::from(Self {
                            poly: PolyCoeff::QQ(monic_qq(g)),
                        })))
                    }
                    (PolyCoeff::QQ(a), PolyCoeff::ZZ(b)) => {
                        let qb = to_qq(b);
                        let g = Rational::structure().polynomials().gcd(a, &qb);
                        Ok(KValue::Object(KObject::from(Self {
                            poly: PolyCoeff::QQ(monic_qq(g)),
                        })))
                    }
                }
            }
            unexpected => unexpected_args("|Poly|", unexpected),
        }
    }

    #[koto_method]
    pub fn factor(&self) -> KValue {
        let values: Vec<KValue> = match &self.poly {
            PolyCoeff::ZZ(p) => match p.factor().into_powers() {
                Some(factors) => factors
                    .into_iter()
                    .map(|(factor, exp)| {
                        KValue::Tuple(
                            vec![
                                KValue::Object(KObject::from(Self {
                                    poly: PolyCoeff::ZZ(factor),
                                })),
                                KValue::Object(KObject::from(NN(exp))),
                            ]
                            .into(),
                        )
                    })
                    .collect(),
                None => return KValue::Null,
            },
            PolyCoeff::QQ(p) => match p.factor().into_powers() {
                Some(factors) => factors
                    .into_iter()
                    .map(|(factor, exp)| {
                        KValue::Tuple(
                            vec![
                                KValue::Object(KObject::from(Self {
                                    poly: PolyCoeff::QQ(factor),
                                })),
                                KValue::Object(KObject::from(NN(exp))),
                            ]
                            .into(),
                        )
                    })
                    .collect(),
                None => return KValue::Null,
            },
        };
        KValue::List(KList::with_data(values.into()))
    }
}

impl KotoObject for Poly {
    fn add(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let (a, b) = promote_pair(&self.poly, &other.poly);
                let poly = match (&a, &b) {
                    (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => PolyCoeff::ZZ(
                        Integer::structure()
                            .polynomials()
                            .reduce_poly(Integer::structure().polynomials().add(a, b)),
                    ),
                    (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => PolyCoeff::QQ(
                        Rational::structure()
                            .polynomials()
                            .reduce_poly(Rational::structure().polynomials().add(a, b)),
                    ),
                    _ => unreachable!(),
                };
                Ok(KValue::Object(KObject::from(Self { poly })))
            }
            unexpected => unexpected_type("Poly", unexpected),
        }
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let (a, b) = promote_pair(&self.poly, &other.poly);
                let poly = match (&a, &b) {
                    (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => PolyCoeff::ZZ(
                        Integer::structure()
                            .polynomials()
                            .reduce_poly(Integer::structure().polynomials().sub(a, b)),
                    ),
                    (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => PolyCoeff::QQ(
                        Rational::structure()
                            .polynomials()
                            .reduce_poly(Rational::structure().polynomials().sub(a, b)),
                    ),
                    _ => unreachable!(),
                };
                Ok(KValue::Object(KObject::from(Self { poly })))
            }
            unexpected => unexpected_type("Poly", unexpected),
        }
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let (a, b) = promote_pair(&self.poly, &other.poly);
                let poly = match (&a, &b) {
                    (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => PolyCoeff::ZZ(
                        Integer::structure().polynomials().mul(a, b),
                    ),
                    (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => PolyCoeff::QQ(
                        Rational::structure().polynomials().mul(a, b),
                    ),
                    _ => unreachable!(),
                };
                Ok(KValue::Object(KObject::from(Self { poly })))
            }
            // Scalar multiplication: Number / NN / ZZ / Q
            other => {
                let scalar = scalar_from_value(other)?;
                let poly = match (&self.poly, &scalar) {
                    (PolyCoeff::ZZ(p), Scalar::Int(v)) => PolyCoeff::ZZ(
                        Integer::structure().polynomials().mul_scalar(p, v),
                    ),
                    (PolyCoeff::ZZ(p), Scalar::Rat(v)) => {
                        let qp = to_qq(p);
                        PolyCoeff::QQ(Rational::structure().polynomials().mul_scalar(&qp, v))
                    }
                    (PolyCoeff::QQ(p), Scalar::Int(v)) => {
                        let rv = Rational::from(v.clone());
                        PolyCoeff::QQ(Rational::structure().polynomials().mul_scalar(p, &rv))
                    }
                    (PolyCoeff::QQ(p), Scalar::Rat(v)) => {
                        PolyCoeff::QQ(Rational::structure().polynomials().mul_scalar(p, v))
                    }
                };
                Ok(KValue::Object(KObject::from(Self { poly })))
            }
        }
    }

    fn multiply_rhs(&self, other: &KValue) -> Result<KValue> {
        // Allows `2 * p` as well as `p * 2`.
        self.multiply(other)
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let (a, b) = promote_pair(&self.poly, &other.poly);
                match (&a, &b) {
                    (PolyCoeff::ZZ(a), PolyCoeff::ZZ(b)) => Ok(
                        Integer::structure().polynomials().equal(a, b),
                    ),
                    (PolyCoeff::QQ(a), PolyCoeff::QQ(b)) => Ok(
                        Rational::structure().polynomials().equal(a, b),
                    ),
                    _ => unreachable!(),
                }
            }
            unexpected => unexpected_type("Poly", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        let s = match &self.poly {
            PolyCoeff::ZZ(p) => display_zz(p),
            PolyCoeff::QQ(p) => display_qq(p),
        };
        ctx.append(s);
        Ok(())
    }
}
