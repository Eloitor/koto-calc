use crate::NN::NN;
use crate::Q::Q;
use crate::ZZ::ZZ;
use koto_runtime::{Result, derive::*, prelude::*};

use algebraeon::nzq::{Integer, Natural, Rational};
use algebraeon::nzq::traits::Fraction;
use algebraeon::rings::matrix::{Matrix, StandardInnerProduct};
use algebraeon::sets::structure::MetaType;
use std::str::FromStr;

/// A matrix over the integers (Mat::ZZ) or over the rationals (Mat::Q).
///
/// `Mat([[..], [..]])` auto-selects the coefficient ring: ZZ when every entry
/// is an integer (Number/NN/ZZ/Q integer), Q as soon as a fraction appears.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub enum Mat {
    /// Matrix with integer entries (lattice basis for `.lll()`).
    ZZ(Matrix<Integer>),
    /// Matrix with rational entries.
    Q(Matrix<Rational>),
}

fn mat_opp_err(e: &algebraeon::rings::matrix::MatOppErr) -> &'static str {
    match e {
        algebraeon::rings::matrix::MatOppErr::DimMismatch => "dimension mismatch",
        algebraeon::rings::matrix::MatOppErr::InvalidIndex => "invalid index",
        algebraeon::rings::matrix::MatOppErr::NotSquare => "matrix is not square",
        algebraeon::rings::matrix::MatOppErr::Singular => "matrix is singular",
    }
}

fn mat_error(msg: &str, e: &algebraeon::rings::matrix::MatOppErr) -> koto_runtime::Error {
    koto_runtime::Error::from(format!("{}: {}", msg, mat_opp_err(e)))
}

fn is_mat(value: &KValue) -> bool {
    matches!(value, KValue::Object(object) if object.is_a::<Mat>())
}

fn index_from_value(value: &KValue) -> Result<usize> {
    match value {
        KValue::Number(n) if n.is_i64() => {
            let i = i64::from(*n);
            if i < 0 {
                runtime_error!("index must be non-negative, got {}", i)
            } else {
                Ok(i as usize)
            }
        }
        KValue::Object(object) if object.is_a::<NN>() => {
            let nn = object.cast::<NN>().unwrap();
            let u: usize = (&nn.0)
                .try_into()
                .map_err(|_| koto_runtime::Error::from(format!("index too large")))?;
            Ok(u)
        }
        unexpected => unexpected_type("Number or NN (index)", unexpected),
    }
}

#[koto_impl]
impl Mat {
    /// True when `value` denotes an integer (Number i64, NN, ZZ, or integer Q).
    pub fn is_integer_value(value: &KValue) -> Result<bool> {
        match value {
            KValue::Number(n) => Ok(n.is_i64()),
            KValue::Object(object) => {
                if let Ok(q) = object.cast::<Q>() {
                    Ok(q.0.is_integer())
                } else if object.is_a::<NN>() || object.is_a::<ZZ>() {
                    Ok(true)
                } else {
                    unexpected_type("Number, NN, ZZ, or Q", value)
                }
            }
            unexpected => unexpected_type("Number, NN, ZZ, or Q", unexpected),
        }
    }

    /// Converts a koto value (Number/NN/ZZ/Q) into an algebraeon Integer.
    /// Errors when the value is a non-integer rational.
    pub fn integer_from_value(value: &KValue) -> Result<Integer> {
        match value {
            KValue::Number(n) if n.is_i64() => Ok(Integer::from(i64::from(*n))),
            KValue::Object(object) => {
                if let Ok(nn) = object.cast::<NN>() {
                    Ok(Integer::from(nn.0.clone()))
                } else if let Ok(zz) = object.cast::<ZZ>() {
                    Ok(zz.to_integer())
                } else if let Ok(q) = object.cast::<Q>() {
                    match Integer::try_from(&q.0) {
                        Ok(i) => Ok(i),
                        Err(()) => runtime_error!("expected an integer entry, got {}", q.0),
                    }
                } else {
                    unexpected_type("Number, NN, ZZ, or Q", value)
                }
            }
            unexpected => unexpected_type("Number, NN, ZZ, or Q", unexpected),
        }
    }

    /// Converts a koto value (Number/NN/ZZ/Q) into an algebraeon Rational.
    pub fn rational_from_value(value: &KValue) -> Result<Rational> {
        Q::rational_from_value(value)
    }

    /// Builds a Mat from a koto list of lists (rows).
    pub fn from_value(value: &KValue) -> Result<Self> {
        let rows_vals = match value {
            KValue::List(list) => {
                let mut rows = Vec::with_capacity(list.len());
                for row in list.data().iter() {
                    match row {
                        KValue::List(r) => rows.push(r.data().iter().cloned().collect::<Vec<_>>()),
                        unexpected => {
                            return unexpected_type(
                                "List of lists (each row must be a List)",
                                unexpected,
                            )
                        }
                    }
                }
                rows
            }
            unexpected => return unexpected_type("List of lists", unexpected),
        };
        if rows_vals.is_empty() {
            return runtime_error!("Mat: empty matrix is not supported");
        }
        let cols = rows_vals[0].len();
        if cols == 0 {
            return runtime_error!("Mat: rows must not be empty");
        }
        for (i, row) in rows_vals.iter().enumerate() {
            if row.len() != cols {
                return runtime_error!(
                    "Mat: all rows must have the same length (row {} has {}, expected {})",
                    i,
                    row.len(),
                    cols
                );
            }
        }

        // Auto-select the coefficient ring: ZZ when all entries are integers,
        // Q as soon as any fraction appears.
        let mut has_fraction = false;
        for row in &rows_vals {
            for entry in row {
                if !Self::is_integer_value(entry)? {
                    has_fraction = true;
                }
            }
        }

        if has_fraction {
            let mut entries = Vec::with_capacity(rows_vals.len() * cols);
            for row in &rows_vals {
                for entry in row {
                    entries.push(Self::rational_from_value(entry)?);
                }
            }
            Ok(Mat::Q(Matrix::from_rows(
                entries
                    .chunks(cols)
                    .map(|r| r.to_vec())
                    .collect::<Vec<Vec<Rational>>>(),
            )))
        } else {
            let mut entries = Vec::with_capacity(rows_vals.len() * cols);
            for row in &rows_vals {
                for entry in row {
                    entries.push(Self::integer_from_value(entry)?);
                }
            }
            Ok(Mat::ZZ(Matrix::from_rows(
                entries
                    .chunks(cols)
                    .map(|r| r.to_vec())
                    .collect::<Vec<Vec<Integer>>>(),
            )))
        }
    }

    /// Converts a koto value back into a Mat (for method arguments).
    pub fn mat_from_value(value: &KValue) -> Result<Self> {
        match value {
            KValue::Object(object) if object.is_a::<Self>() => {
                Ok(object.cast::<Self>().unwrap().clone())
            }
            unexpected => unexpected_type("Mat", unexpected),
        }
    }

    /// The same matrix with rational entries (ZZ matrices are promoted).
    fn as_rational(&self) -> Matrix<Rational> {
        match self {
            Mat::ZZ(m) => m.apply_map(|x| Rational::from(x.clone())),
            Mat::Q(m) => m.clone(),
        }
    }

    fn row_count(&self) -> usize {
        match self {
            Mat::ZZ(m) => m.rows(),
            Mat::Q(m) => m.rows(),
        }
    }

    fn col_count(&self) -> usize {
        match self {
            Mat::ZZ(m) => m.cols(),
            Mat::Q(m) => m.cols(),
        }
    }

    /// Number of rows as NN.
    #[koto_method]
    pub fn rows(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.row_count().into())))
    }

    /// Number of columns as NN.
    #[koto_method]
    pub fn cols(&self) -> KValue {
        KValue::Object(KObject::from(NN(self.col_count().into())))
    }

    /// Entry at row `r`, column `c` (0-indexed) as ZZ or Q.
    #[koto_method]
    pub fn at(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [r, c] => {
                let r = index_from_value(r)?;
                let c = index_from_value(c)?;
                match self {
                    Mat::ZZ(m) => m
                        .at(r, c)
                        .map(|e| KValue::Object(KObject::from(ZZ::from_integer(e.clone()))))
                        .map_err(|e| mat_error("Mat.at", &e)),
                    Mat::Q(m) => m
                        .at(r, c)
                        .map(|e| KValue::Object(KObject::from(Q(e.clone()))))
                        .map_err(|e| mat_error("Mat.at", &e)),
                }
            }
            unexpected => unexpected_args("|NN, NN|", unexpected),
        }
    }

    /// Transposed matrix.
    #[koto_method]
    pub fn transpose(&self) -> KValue {
        match self {
            Mat::ZZ(m) => KValue::Object(KObject::from(Mat::ZZ(m.transpose_ref()))),
            Mat::Q(m) => KValue::Object(KObject::from(Mat::Q(m.transpose_ref()))),
        }
    }

    /// Matrix product with another Mat (or scalar multiplication with a
    /// Number/NN/ZZ/Q).
    #[koto_method]
    pub fn mul(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [other] => self.multiply(other),
            unexpected => unexpected_args("|Mat| or |Number/NN/ZZ/Q|", unexpected),
        }
    }

    /// Determinant (ZZ for integer matrices, Q for rational matrices).
    #[koto_method]
    pub fn det(&self) -> Result<KValue> {
        match self {
            Mat::ZZ(m) => m
                .det()
                .map(|d| KValue::Object(KObject::from(ZZ::from_integer(d))))
                .map_err(|e| mat_error("Mat.det", &e)),
            Mat::Q(m) => m
                .det()
                .map(|d| KValue::Object(KObject::from(Q(d))))
                .map_err(|e| mat_error("Mat.det", &e)),
        }
    }

    /// Inverse matrix over Q (integer matrices are converted to Q first).
    #[koto_method]
    pub fn inverse(&self) -> Result<KValue> {
        match self {
            Mat::ZZ(m) => {
                let mq: Matrix<Rational> = m.apply_map(|x| Rational::from(x.clone()));
                mq.inv()
                    .map(|i| KValue::Object(KObject::from(Mat::Q(i))))
                    .map_err(|e| mat_error("Mat.inverse", &e))
            }
            Mat::Q(m) => m
                .inv()
                .map(|i| KValue::Object(KObject::from(Mat::Q(i))))
                .map_err(|e| mat_error("Mat.inverse", &e)),
        }
    }

    /// Integer LLL reduction (new in algebraeon 0.0.16/0.0.17).
    ///
    /// Takes the rows of the matrix as the basis of a lattice and returns a
    /// new Mat whose rows are an LLL-reduced basis (delta = 3/4, standard
    /// inner product). Only defined for integer (ZZ) matrices.
    #[koto_method]
    pub fn lll(&self) -> Result<KValue> {
        match self {
            Mat::ZZ(m) => {
                let delta = Rational::from_str("3/4").unwrap();
                let (_h, reduced) = m.clone().lll_integral_row_reduction_algorithm(
                    &StandardInnerProduct::new(Integer::structure()),
                    &delta,
                );
                Ok(KValue::Object(KObject::from(Mat::ZZ(reduced))))
            }
            Mat::Q(_) => runtime_error!(
                "Mat.lll: LLL reduction is only defined for integer (ZZ) matrices"
            ),
        }
    }
}

impl KotoObject for Mat {
    fn add(&self, other: &KValue) -> Result<KValue> {
        if is_mat(other) {
            let b = Self::mat_from_value(other)?;
            match (self, &b) {
                (Mat::ZZ(a), Mat::ZZ(b)) => Matrix::add(a, b)
                    .map(|m| KValue::Object(KObject::from(Mat::ZZ(m))))
                    .map_err(|e| mat_error("Mat + Mat", &e)),
                _ => {
                    let a = self.as_rational();
                    let b = b.as_rational();
                    Matrix::add(&a, &b)
                        .map(|m| KValue::Object(KObject::from(Mat::Q(m))))
                        .map_err(|e| mat_error("Mat + Mat", &e))
                }
            }
        } else {
            unexpected_type("Mat", other)
        }
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        if is_mat(other) {
            let b = Self::mat_from_value(other)?;
            match (self, &b) {
                (Mat::ZZ(a), Mat::ZZ(b)) => Matrix::add(a, &b.neg())
                    .map(|m| KValue::Object(KObject::from(Mat::ZZ(m))))
                    .map_err(|e| mat_error("Mat - Mat", &e)),
                _ => {
                    let a = self.as_rational();
                    let b = b.as_rational().neg();
                    Matrix::add(&a, &b)
                        .map(|m| KValue::Object(KObject::from(Mat::Q(m))))
                        .map_err(|e| mat_error("Mat - Mat", &e))
                }
            }
        } else {
            unexpected_type("Mat", other)
        }
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        if is_mat(other) {
            let b = Self::mat_from_value(other)?;
            match (self, &b) {
                (Mat::ZZ(a), Mat::ZZ(b)) => Matrix::mul(a, b)
                    .map(|m| KValue::Object(KObject::from(Mat::ZZ(m))))
                    .map_err(|e| mat_error("Mat * Mat", &e)),
                (Mat::ZZ(a), Mat::Q(b)) => {
                    let aq: Matrix<Rational> = a.apply_map(|x| Rational::from(x.clone()));
                    Matrix::mul(&aq, b)
                        .map(|m| KValue::Object(KObject::from(Mat::Q(m))))
                        .map_err(|e| mat_error("Mat * Mat", &e))
                }
                (Mat::Q(a), Mat::ZZ(b)) => {
                    let bq: Matrix<Rational> = b.apply_map(|x| Rational::from(x.clone()));
                    Matrix::mul(a, &bq)
                        .map(|m| KValue::Object(KObject::from(Mat::Q(m))))
                        .map_err(|e| mat_error("Mat * Mat", &e))
                }
                (Mat::Q(a), Mat::Q(b)) => Matrix::mul(a, b)
                    .map(|m| KValue::Object(KObject::from(Mat::Q(m))))
                    .map_err(|e| mat_error("Mat * Mat", &e)),
            }
        } else {
            // Scalar multiplication (Number/NN/ZZ/Q).
            match self {
                Mat::ZZ(m) => {
                    if Self::is_integer_value(other)? {
                        let scalar = Self::integer_from_value(other)?;
                        Ok(KValue::Object(KObject::from(Mat::ZZ(
                            m.mul_scalar_ref(&scalar),
                        ))))
                    } else {
                        // Fractional scalar: promote the matrix to Q first.
                        let scalar = Self::rational_from_value(other)?;
                        let mq: Matrix<Rational> = m.apply_map(|x| Rational::from(x.clone()));
                        Ok(KValue::Object(KObject::from(Mat::Q(
                            mq.mul_scalar_ref(&scalar),
                        ))))
                    }
                }
                Mat::Q(m) => {
                    let scalar = Self::rational_from_value(other)?;
                    Ok(KValue::Object(KObject::from(Mat::Q(
                        m.mul_scalar_ref(&scalar),
                    ))))
                }
            }
        }
    }

    fn negate(&self) -> Result<KValue> {
        match self {
            Mat::ZZ(m) => Ok(KValue::Object(KObject::from(Mat::ZZ(m.neg())))),
            Mat::Q(m) => Ok(KValue::Object(KObject::from(Mat::Q(m.neg())))),
        }
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        if is_mat(other) {
            let b = Self::mat_from_value(other)?;
            Ok(match (self, &b) {
                (Mat::ZZ(a), Mat::ZZ(b)) => a == b,
                (Mat::Q(a), Mat::Q(b)) => a == b,
                // Mixed ZZ/Q: promote both sides to Q and compare.
                _ => self.as_rational() == b.as_rational(),
            })
        } else {
            unexpected_type("Mat", other)
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append("[");
        match self {
            Mat::ZZ(m) => {
                for r in 0..m.rows() {
                    if r > 0 {
                        ctx.append(", ");
                    }
                    ctx.append("[");
                    for c in 0..m.cols() {
                        if c > 0 {
                            ctx.append(", ");
                        }
                        ctx.append(m.at(r, c).unwrap().to_string());
                    }
                    ctx.append("]");
                }
            }
            Mat::Q(m) => {
                for r in 0..m.rows() {
                    if r > 0 {
                        ctx.append(", ");
                    }
                    ctx.append("[");
                    for c in 0..m.cols() {
                        if c > 0 {
                            ctx.append(", ");
                        }
                        let e = m.at(r, c).unwrap();
                        let (num, den) = (&e).numerator_and_denominator();
                        if den == Natural::ONE {
                            ctx.append(num.to_string());
                        } else {
                            ctx.append(format!("{}/{}", num, den));
                        }
                    }
                    ctx.append("]");
                }
            }
        }
        ctx.append("]");
        Ok(())
    }
}

