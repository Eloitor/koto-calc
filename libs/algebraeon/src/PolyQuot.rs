use crate::NN::NN;
use crate::Poly::{Poly, PolyCoeff};
use crate::Q::Q;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Natural, Rational, RationalCanonicalStructure};
use algebraeon::sets::structure::{EqSignature, MetaType};
use algebraeon_rings::polynomial::{
    Polynomial, PolynomialQuotientRingStructure, PolynomialStructure, ToPolynomialSignature,
};
use algebraeon_rings::structure::{
    AdditionSignature, AdditiveGroupSignature, MultiplicationSignature,
    RingToQuotientFieldSignature, TryReciprocalSignature,
};

/// The owned algebraeon structure used for Q[x].
type RationalPolynomialStructure =
    PolynomialStructure<RationalCanonicalStructure, RationalCanonicalStructure>;

/// The algebraeon structure used for the number field Q[x]/(f).
type RationalPolynomialQuotient = PolynomialQuotientRingStructure<
    RationalCanonicalStructure,
    RationalCanonicalStructure,
    RationalPolynomialStructure,
    true,
>;

/// Promotes a `Poly` over ZZ or Q to a polynomial over Q.
fn rational_polynomial(poly: &Poly) -> Polynomial<Rational> {
    match &poly.poly {
        PolyCoeff::ZZ(poly) => poly.apply_map(|coefficient| Rational::from(coefficient.clone())),
        PolyCoeff::QQ(poly) => poly.clone(),
    }
}

/// Extracts a rational polynomial from either a `Poly` or a Koto coefficient
/// list. Coefficients in lists use the same ascending-degree convention as
/// `Poly`: `[c0, c1, ...]` represents c0 + c1*x + ... .
fn rational_polynomial_from_value(value: &KValue) -> Result<Polynomial<Rational>> {
    match value {
        KValue::Object(object) if object.is_a::<Poly>() => {
            let poly = object.cast::<Poly>()?;
            Ok(rational_polynomial(&poly))
        }
        KValue::List(list) => {
            let object = Poly::from_koto_list(list)?;
            let poly = object.cast::<Poly>()?;
            Ok(rational_polynomial(&poly))
        }
        unexpected => unexpected_type("Poly or List", unexpected),
    }
}

/// Reconstructs the quotient-field structure on demand. Construction is cheap,
/// while retaining the modulus in every Koto value makes cross-field checks
/// explicit and keeps elements self-contained.
fn quotient_structure(modulus: &Polynomial<Rational>) -> RationalPolynomialQuotient {
    Rational::structure()
        .into_polynomials()
        .into_quotient_field_unchecked(modulus.clone())
}

/// Appends a rational polynomial using `Poly`'s established display format.
fn display_polynomial(
    polynomial: &Polynomial<Rational>,
    ctx: &mut DisplayContext,
) -> koto_runtime::Result<()> {
    <Poly as KotoObject>::display(
        &Poly {
            poly: PolyCoeff::QQ(polynomial.clone()),
        },
        ctx,
    )
}

/// A polynomial quotient field Q[x]/(f), where f is monic and irreducible over
/// Q. Elements are constructed with `.of(Poly(...))` or `.of([...])`.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct PolyQuot {
    modulus: Polynomial<Rational>,
}

#[koto_impl]
impl PolyQuot {
    /// `PolyQuot(f)` constructs Q[x]/(f). The modulus must be a monic,
    /// irreducible `Poly` over Q.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::Object(object)] if object.is_a::<Poly>() => {
                let poly = object.cast::<Poly>()?;
                let modulus = rational_polynomial(&poly);
                if modulus.leading_coeff() != Some(Rational::ONE) {
                    return runtime_error!("PolyQuot: the modulus must be monic over Q");
                }

                // Use algebraeon's checked quotient-field constructor for the
                // exact irreducibility test over Q.
                if Rational::structure()
                    .into_polynomials()
                    .into_quotient_field(modulus.clone())
                    .is_none()
                {
                    return runtime_error!("PolyQuot: the modulus must be irreducible over Q");
                }

                Ok(KValue::Object(KObject::from(Self { modulus })))
            }
            unexpected => unexpected_args("|Poly|", unexpected),
        }
    }

    /// Creates an element and reduces its polynomial representative modulo f.
    /// The argument can be a `Poly` or a coefficient list in ascending degree.
    #[koto_method]
    pub fn of(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [value] => {
                let polynomial = rational_polynomial_from_value(value)?;
                let value = quotient_structure(&self.modulus).reduce(polynomial);
                Ok(KValue::Object(KObject::from(PolyQuotElement {
                    modulus: self.modulus.clone(),
                    value,
                })))
            }
            unexpected => unexpected_args("|Poly or List|", unexpected),
        }
    }

    /// The distinguished class of x in Q[x]/(f).
    #[koto_method]
    pub fn generator(&self) -> KValue {
        let structure = quotient_structure(&self.modulus);
        KValue::Object(KObject::from(PolyQuotElement {
            modulus: self.modulus.clone(),
            value: structure.reduce(structure.generator()),
        }))
    }

    /// The degree [Q[x]/(f) : Q] = degree(f).
    #[koto_method]
    pub fn degree(&self) -> KValue {
        KValue::Object(KObject::from(NN(Natural::from(
            quotient_structure(&self.modulus).degree(),
        ))))
    }
}

impl KotoObject for PolyQuot {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                Ok(self.modulus == other.cast::<Self>()?.modulus)
            }
            unexpected => unexpected_type("PolyQuot", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append("Q[x]/(");
        display_polynomial(&self.modulus, ctx)?;
        ctx.append(")");
        Ok(())
    }
}

/// An exact element of a particular polynomial quotient field. `value` is
/// always the representative reduced modulo `modulus`.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct PolyQuotElement {
    modulus: Polynomial<Rational>,
    value: Polynomial<Rational>,
}

impl PartialEq for PolyQuotElement {
    fn eq(&self, other: &Self) -> bool {
        self.modulus == other.modulus
            && quotient_structure(&self.modulus).equal(&self.value, &other.value)
    }
}

impl Eq for PolyQuotElement {}

#[koto_impl]
impl PolyQuotElement {
    /// The degree of the ambient extension Q[x]/(f) over Q.
    #[koto_method]
    pub fn degree(&self) -> KValue {
        KValue::Object(KObject::from(NN(Natural::from(
            quotient_structure(&self.modulus).degree(),
        ))))
    }

    /// The monic minimal polynomial of this element over Q.
    #[koto_method]
    pub fn min_poly(&self) -> KValue {
        let polynomial = quotient_structure(&self.modulus).min_poly(&self.value);
        KValue::Object(KObject::from(Poly {
            poly: PolyCoeff::QQ(polynomial),
        }))
    }

    /// The field norm of this element down to Q.
    #[koto_method]
    pub fn norm(&self) -> KValue {
        KValue::Object(KObject::from(Q(
            quotient_structure(&self.modulus).norm(&self.value)
        )))
    }

    /// The field trace of this element down to Q.
    #[koto_method]
    pub fn trace(&self) -> KValue {
        KValue::Object(KObject::from(Q(
            quotient_structure(&self.modulus).trace(&self.value)
        )))
    }

    /// The canonical polynomial representative of degree less than degree(f).
    #[koto_method]
    pub fn to_poly(&self) -> KValue {
        KValue::Object(KObject::from(Poly {
            poly: PolyCoeff::QQ(self.value.clone()),
        }))
    }

    /// The multiplicative inverse, computed by algebraeon's extended Euclidean
    /// algorithm. Zero (or any non-unit, should a structure invariant be
    /// violated) produces a Koto error.
    #[koto_method]
    pub fn inverse(&self) -> Result<KValue> {
        match quotient_structure(&self.modulus).try_reciprocal(&self.value) {
            Some(value) => Ok(KValue::Object(KObject::from(Self {
                modulus: self.modulus.clone(),
                value,
            }))),
            None => runtime_error!("PolyQuot: element has no multiplicative inverse"),
        }
    }

    fn structure(&self) -> RationalPolynomialQuotient {
        quotient_structure(&self.modulus)
    }

    fn with_value(&self, value: Polynomial<Rational>) -> Self {
        Self {
            modulus: self.modulus.clone(),
            value,
        }
    }

    fn check_same_ring(&self, other: &Self, operation: &str) -> Result<()> {
        if self.modulus != other.modulus {
            return runtime_error!(
                "PolyQuot: cannot {} elements from different quotient rings",
                operation
            );
        }
        Ok(())
    }
}

impl KotoObject for PolyQuotElement {
    fn add(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_ring(&other, "add")?;
        Ok(KValue::Object(KObject::from(self.with_value(
            self.structure().add(&self.value, &other.value),
        ))))
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_ring(&other, "subtract")?;
        Ok(KValue::Object(KObject::from(self.with_value(
            self.structure().sub(&self.value, &other.value),
        ))))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        let other = cast_element(other)?;
        self.check_same_ring(&other, "multiply")?;
        Ok(KValue::Object(KObject::from(self.with_value(
            self.structure().mul(&self.value, &other.value),
        ))))
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(
            self.with_value(self.structure().neg(&self.value)),
        )))
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>()?;
                if self.modulus != other.modulus {
                    Ok(false)
                } else {
                    Ok(self.structure().equal(&self.value, &other.value))
                }
            }
            unexpected => unexpected_type("PolyQuotElement", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        display_polynomial(&self.value, ctx)
    }
}

fn cast_element(value: &KValue) -> Result<PolyQuotElement> {
    match value {
        KValue::Object(object) if object.is_a::<PolyQuotElement>() => {
            Ok(object.cast::<PolyQuotElement>()?.clone())
        }
        unexpected => unexpected_type("PolyQuotElement", unexpected),
    }
}
