use crate::CF::integer_from_value;
use crate::Q::Q;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Rational};
use algebraeon::sets::structure::{EqSignature, SetSignature};
use algebraeon_rings::algebraic_number_field::{
    QuadraticNumberFieldElement, QuadraticNumberFieldStructure, QuadraticRingOfIntegersStructure,
};
use algebraeon_rings::structure::{
    AdditionSignature, AdditiveGroupSignature, MultiplicationSignature,
};

/// The quadratic number field Q(sqrt(d)), where d is a squarefree integer
/// other than 1.
///
/// `QSqrt(d)` constructs the field. Its `.of(a, b)` method constructs the
/// exact element a + b sqrt(d). As a shorthand, `QSqrt(d, a, b)` constructs
/// the same element directly.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct QSqrt {
    d: Integer,
}

#[koto_impl]
impl QSqrt {
    /// QSqrt(d) creates Q(sqrt(d)); QSqrt(d, a, b) creates a + b sqrt(d).
    /// QSqrt(d, x), for an existing QSqrt element x of the same field, returns
    /// a copy of x.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [d] => Ok(KValue::Object(KObject::from(Self::from_d_value(d)?))),
            [d, a, b] => {
                let field = Self::from_d_value(d)?;
                let a = Q::rational_from_value(a)?;
                let b = Q::rational_from_value(b)?;
                Ok(KValue::Object(KObject::from(field.element(a, b))))
            }
            [d, KValue::Object(object)] if object.is_a::<QSqrtElement>() => {
                let field = Self::from_d_value(d)?;
                let element = object.cast::<QSqrtElement>()?.clone();
                field.check_element_field(&element)?;
                Ok(KValue::Object(KObject::from(element)))
            }
            unexpected => unexpected_args("|d|, |d, a, b|, or |d, QSqrt element|", unexpected),
        }
    }

    /// Creates an element of this field. `.of(a, b)` is a + b sqrt(d), and
    /// `.of(a)` embeds the rational a into the field. An existing element can
    /// be copied only when it belongs to this same field.
    #[koto_method]
    pub fn of(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [a, b] => Ok(KValue::Object(KObject::from(
                self.element(Q::rational_from_value(a)?, Q::rational_from_value(b)?),
            ))),
            [KValue::Object(object)] if object.is_a::<QSqrtElement>() => {
                let element = object.cast::<QSqrtElement>()?.clone();
                self.check_element_field(&element)?;
                Ok(KValue::Object(KObject::from(element)))
            }
            [a] => Ok(KValue::Object(KObject::from(
                self.element(Q::rational_from_value(a)?, Rational::ZERO),
            ))),
            unexpected => unexpected_args("|a|, |a, b|, or |QSqrt element|", unexpected),
        }
    }

    /// The distinguished generator sqrt(d).
    #[koto_method]
    pub fn generator(&self) -> KValue {
        KValue::Object(KObject::from(self.element(Rational::ZERO, Rational::ONE)))
    }

    /// The squarefree radicand d as a ZZ.
    #[koto_method]
    pub fn d(&self) -> KValue {
        KValue::Object(KObject::from(ZZ::from_integer(self.d.clone())))
    }

    /// The full ring of algebraic integers of Q(sqrt(d)).
    #[koto_method]
    pub fn ring_of_integers(&self) -> Result<KValue> {
        // Construct through algebraeon's checked API rather than merely
        // reusing d: this keeps the wrapper tied to its exact ROI structure.
        let ring = QuadraticRingOfIntegersStructure::new(self.d.clone())
            .expect("a QSqrt field always has a valid squarefree radicand");
        Ok(KValue::Object(KObject::from(QSqrtRingOfIntegers {
            d: ring.d().clone(),
        })))
    }

    fn from_d_value(value: &KValue) -> Result<Self> {
        let d = integer_from_value(value)?;
        match QuadraticNumberFieldStructure::new(d.clone()) {
            Ok(_) => Ok(Self { d }),
            Err(()) => runtime_error!(
                "QSqrt: d must be a squarefree integer other than 1, got {}",
                d
            ),
        }
    }

    fn element(&self, a: Rational, b: Rational) -> QSqrtElement {
        QSqrtElement {
            d: self.d.clone(),
            value: QuadraticNumberFieldElement {
                rational_part: a,
                algebraic_part: b,
            },
        }
    }

    fn check_element_field(&self, element: &QSqrtElement) -> Result<()> {
        if self.d != element.d {
            return runtime_error!(
                "QSqrt: element belongs to Q(sqrt({})), not Q(sqrt({}))",
                element.d,
                self.d
            );
        }
        Ok(())
    }
}

impl KotoObject for QSqrt {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => Ok(self.d == other.cast::<Self>()?.d),
            unexpected => unexpected_type("QSqrt", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format!("Q(sqrt({}))", self.d));
        Ok(())
    }
}

/// An exact element a + b sqrt(d) of a particular [`QSqrt`] field.
///
/// The field is retained alongside the algebraeon element because two
/// `QuadraticNumberFieldElement` values alone do not carry their radicand.
/// All arithmetic therefore verifies that the radicands agree before asking
/// algebraeon's `QuadraticNumberFieldStructure` to add, multiply, or negate.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct QSqrtElement {
    d: Integer,
    value: QuadraticNumberFieldElement,
}

impl PartialEq for QSqrtElement {
    fn eq(&self, other: &Self) -> bool {
        self.d == other.d && self.structure().equal(&self.value, &other.value)
    }
}

impl Eq for QSqrtElement {}

#[koto_impl]
impl QSqrtElement {
    /// The rational coefficient a in a + b sqrt(d).
    #[koto_method]
    pub fn a(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.value.rational_part.clone())))
    }

    /// The sqrt(d) coefficient b in a + b sqrt(d).
    #[koto_method]
    pub fn b(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.value.algebraic_part.clone())))
    }

    /// The radicand d as a ZZ.
    #[koto_method]
    pub fn d(&self) -> KValue {
        KValue::Object(KObject::from(ZZ::from_integer(self.d.clone())))
    }

    /// The field Q(sqrt(d)) containing this element.
    #[koto_method]
    pub fn field(&self) -> KValue {
        KValue::Object(KObject::from(QSqrt { d: self.d.clone() }))
    }

    /// The conjugate a - b sqrt(d).
    #[koto_method]
    pub fn conjugate(&self) -> KValue {
        KValue::Object(KObject::from(self.conjugate_element()))
    }

    /// The field norm (a + b sqrt(d))(a - b sqrt(d)) = a^2 - d*b^2.
    #[koto_method]
    pub fn norm(&self) -> KValue {
        KValue::Object(KObject::from(Q(self.norm_rational())))
    }

    /// The field trace 2*a.
    #[koto_method]
    pub fn trace(&self) -> KValue {
        KValue::Object(KObject::from(Q(
            Rational::TWO * self.value.rational_part.clone()
        )))
    }

    /// The multiplicative inverse, or an error for zero.
    #[koto_method]
    pub fn inverse(&self) -> Result<KValue> {
        match self.reciprocal_element() {
            Some(inverse) => Ok(KValue::Object(KObject::from(inverse))),
            None => runtime_error!("QSqrt: 0 has no multiplicative inverse"),
        }
    }

    /// A floating-point approximation. For real quadratic fields this is a
    /// Number. For d < 0 it is the list [real, imaginary], because Koto has
    /// no native complex Number type.
    #[koto_method]
    pub fn to_float(&self) -> KValue {
        let a = f64::from(&self.value.rational_part);
        let b = f64::from(&self.value.algebraic_part);
        let d = f64::from(&Rational::from(self.d.clone()));
        if d > 0.0 {
            KValue::from(a + b * d.sqrt())
        } else {
            KValue::List(KList::with_data(
                vec![KValue::from(a), KValue::from(b * (-d).sqrt())].into(),
            ))
        }
    }

    /// The full ring of algebraic integers of this element's field.
    #[koto_method]
    pub fn ring_of_integers(&self) -> Result<KValue> {
        QSqrt { d: self.d.clone() }.ring_of_integers()
    }

    fn structure(&self) -> QuadraticNumberFieldStructure<Integer> {
        QuadraticNumberFieldStructure::new(self.d.clone())
            .expect("a QSqrt element always has a valid squarefree radicand")
    }

    fn new_element(&self, value: QuadraticNumberFieldElement) -> Self {
        Self {
            d: self.d.clone(),
            value,
        }
    }

    fn check_same_field(&self, other: &Self, operation: &str) -> Result<()> {
        if self.d != other.d {
            return runtime_error!(
                "QSqrt: cannot {} elements from different fields Q(sqrt({})) and Q(sqrt({}))",
                operation,
                self.d,
                other.d
            );
        }
        Ok(())
    }

    /// Coerces a rational scalar into this field, or checks a QSqrt element's
    /// field. This permits the canonical Q inclusion while still rejecting a
    /// QSqrt element from a different quadratic field.
    fn element_from_value(&self, value: &KValue, operation: &str) -> Result<Self> {
        match value {
            KValue::Object(object) if object.is_a::<Self>() => {
                let other = object.cast::<Self>()?.clone();
                self.check_same_field(&other, operation)?;
                Ok(other)
            }
            scalar => Ok(Self {
                d: self.d.clone(),
                value: QuadraticNumberFieldElement {
                    rational_part: Q::rational_from_value(scalar)?,
                    algebraic_part: Rational::ZERO,
                },
            }),
        }
    }

    fn conjugate_element(&self) -> Self {
        Self {
            d: self.d.clone(),
            value: QuadraticNumberFieldElement {
                rational_part: self.value.rational_part.clone(),
                algebraic_part: -self.value.algebraic_part.clone(),
            },
        }
    }

    fn norm_rational(&self) -> Rational {
        let a_squared = self.value.rational_part.clone() * self.value.rational_part.clone();
        let b_squared = self.value.algebraic_part.clone() * self.value.algebraic_part.clone();
        a_squared - Rational::from(self.d.clone()) * b_squared
    }

    fn reciprocal_element(&self) -> Option<Self> {
        let norm = self.norm_rational();
        if norm == Rational::ZERO {
            return None;
        }

        // algebraeon-rings 0.0.17's quadratic `try_reciprocal` uses
        // a^2 + d*b^2 as its denominator. For a + b*sqrt(d), the exact field
        // norm is a^2 - d*b^2; use it directly so inverse/division retain the
        // field identity x * x^-1 = 1 for both positive and negative d.
        Some(Self {
            d: self.d.clone(),
            value: QuadraticNumberFieldElement {
                rational_part: self.value.rational_part.clone() / norm.clone(),
                algebraic_part: -self.value.algebraic_part.clone() / norm,
            },
        })
    }
}

impl KotoObject for QSqrtElement {
    fn add(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "add")?;
        Ok(KValue::Object(KObject::from(self.new_element(
            self.structure().add(&self.value, &other.value),
        ))))
    }

    fn add_rhs(&self, other: &KValue) -> Result<KValue> {
        // Addition is commutative, and element_from_value checks cross-field
        // operands before embedding rational scalars.
        self.add(other)
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "subtract")?;
        Ok(KValue::Object(KObject::from(self.new_element(
            self.structure().sub(&self.value, &other.value),
        ))))
    }

    fn subtract_rhs(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "subtract")?;
        Ok(KValue::Object(KObject::from(self.new_element(
            self.structure().sub(&other.value, &self.value),
        ))))
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "multiply")?;
        Ok(KValue::Object(KObject::from(self.new_element(
            self.structure().mul(&self.value, &other.value),
        ))))
    }

    fn multiply_rhs(&self, other: &KValue) -> Result<KValue> {
        // Multiplication is commutative in a quadratic number field.
        self.multiply(other)
    }

    fn divide(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "divide")?;
        match other.reciprocal_element() {
            Some(inverse) => Ok(KValue::Object(KObject::from(
                self.new_element(self.structure().mul(&self.value, &inverse.value)),
            ))),
            None => runtime_error!("QSqrt: division by zero"),
        }
    }

    fn divide_rhs(&self, other: &KValue) -> Result<KValue> {
        let other = self.element_from_value(other, "divide")?;
        match self.reciprocal_element() {
            Some(inverse) => Ok(KValue::Object(KObject::from(
                self.new_element(self.structure().mul(&other.value, &inverse.value)),
            ))),
            None => runtime_error!("QSqrt: division by zero"),
        }
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KValue::Object(KObject::from(
            self.new_element(self.structure().neg(&self.value)),
        )))
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(object) if object.is_a::<Self>() => {
                let other = object.cast::<Self>()?;
                if self.d != other.d {
                    Ok(false)
                } else {
                    Ok(self.structure().equal(&self.value, &other.value))
                }
            }
            scalar => {
                let scalar = Q::rational_from_value(scalar)?;
                Ok(self.value.algebraic_part == Rational::ZERO
                    && self.value.rational_part == scalar)
            }
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format_qsqrt(
            &self.d,
            &self.value.rational_part,
            &self.value.algebraic_part,
        ));
        Ok(())
    }
}

/// The ring of algebraic integers O_Q(sqrt(d)). It is kept as a separate Koto
/// object so a field can expose its ring of integers without pretending that
/// every field element is integral.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct QSqrtRingOfIntegers {
    d: Integer,
}

#[koto_impl]
impl QSqrtRingOfIntegers {
    /// Constructs an algebraic integer in this ring. The one-argument form
    /// embeds a rational integer (or validates an existing QSqrt element); the
    /// two-argument form is a + b sqrt(d), with exact rational coefficients
    /// that must satisfy the ring-of-integers integrality condition.
    #[koto_method]
    pub fn of(&self, args: &[KValue]) -> Result<KValue> {
        let element = match args {
            [a, b] => QSqrtElement {
                d: self.d.clone(),
                value: QuadraticNumberFieldElement {
                    rational_part: Q::rational_from_value(a)?,
                    algebraic_part: Q::rational_from_value(b)?,
                },
            },
            [KValue::Object(object)] if object.is_a::<QSqrtElement>() => {
                let element = object.cast::<QSqrtElement>()?.clone();
                if element.d != self.d {
                    return runtime_error!(
                        "QSqrt ring of integers: element belongs to Q(sqrt({})), not Q(sqrt({}))",
                        element.d,
                        self.d
                    );
                }
                element
            }
            [a] => QSqrtElement {
                d: self.d.clone(),
                value: QuadraticNumberFieldElement {
                    rational_part: Q::rational_from_value(a)?,
                    algebraic_part: Rational::ZERO,
                },
            },
            unexpected => return unexpected_args("|a|, |a, b|, or |QSqrt element|", unexpected),
        };

        let ring = QuadraticRingOfIntegersStructure::new(self.d.clone())
            .expect("a QSqrt ring always has a valid squarefree radicand");
        match ring.validate_element(&element.value) {
            Ok(()) => Ok(KValue::Object(KObject::from(element))),
            Err(_) => runtime_error!(
                "QSqrt ring of integers: {} is not an algebraic integer in O_Q(sqrt({}))",
                format_qsqrt(
                    &self.d,
                    &element.value.rational_part,
                    &element.value.algebraic_part,
                ),
                self.d
            ),
        }
    }

    /// The radicand d as a ZZ.
    #[koto_method]
    pub fn d(&self) -> KValue {
        KValue::Object(KObject::from(ZZ::from_integer(self.d.clone())))
    }

    /// Its fraction field Q(sqrt(d)).
    #[koto_method]
    pub fn field(&self) -> KValue {
        KValue::Object(KObject::from(QSqrt { d: self.d.clone() }))
    }
}

impl KotoObject for QSqrtRingOfIntegers {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => Ok(self.d == other.cast::<Self>()?.d),
            unexpected => unexpected_type("QSqrtRingOfIntegers", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format!("O_Q(sqrt({}))", self.d));
        Ok(())
    }
}

/// Formats a rational coefficient the same way as Q's display implementation.
fn format_rational(rational: &Rational) -> String {
    rational.to_string()
}

/// Formats a + b sqrt(d), omitting zero terms and an unambiguous coefficient
/// one on sqrt(d). Fractional sqrt coefficients are parenthesized, matching
/// the established Quat display convention.
fn format_qsqrt(d: &Integer, a: &Rational, b: &Rational) -> String {
    if *a == Rational::ZERO && *b == Rational::ZERO {
        return "0".into();
    }

    let mut result = String::new();
    if *a != Rational::ZERO {
        result.push_str(&format_rational(a));
    }

    if *b != Rational::ZERO {
        let negative = *b < Rational::ZERO;
        let magnitude = if negative { -b.clone() } else { b.clone() };
        let coefficient = format_rational(&magnitude);
        let sqrt = format!("sqrt({})", d);
        let term = if magnitude == Rational::ONE {
            sqrt
        } else if coefficient.contains('/') {
            format!("({}){}", coefficient, sqrt)
        } else {
            format!("{}{}", coefficient, sqrt)
        };

        if result.is_empty() {
            if negative {
                result.push('-');
            }
            result.push_str(&term);
        } else {
            result.push_str(if negative { " - " } else { " + " });
            result.push_str(&term);
        }
    }

    result
}
