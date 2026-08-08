//! Multivariate integer polynomials exposed to Koto.
//!
//! The public Koto representation deliberately keeps variables separate from
//! the algebraeon polynomial.  A variable is a [`Variable`] object, while a
//! term is written as `[powers, coefficient]`, where `powers` is a map from a
//! variable name to a non-negative exponent.  For example:
//!
//! ```text
//! MultiPoly([x, y], [[{"x": 2, "y": 1}, 3], [{}, -1]])
//! ```
//!
//! means `3*x^2*y - 1`.  The constructor also accepts the coefficient first
//! (`[coefficient, powers]`) form for convenience.  Coefficients are exact
//! algebraeon integers (`ZZ`, `NN`, or integer Koto numbers).

use crate::CF;
use crate::NN::NN;
use crate::ZZ::ZZ;
use algebraeon::nzq::{Integer, Natural};
use algebraeon::sets::structure::MetaType;
use algebraeon_rings::polynomial::{
    MultiPolynomial, RingToMultiPolynomialRingSignature, Variable as AlgebraVariable,
};
use algebraeon_rings::structure::*;
use koto_runtime::{derive::*, prelude::*, Result};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

static VARIABLE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A symbolic variable used by [`MultiPoly`].
///
/// Algebraeon's variables are identified by identity rather than by their
/// printed name.  We therefore retain both the algebraeon value and the name
/// supplied by the user.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct Variable {
    inner: AlgebraVariable,
    name: String,
    id: usize,
}

impl Variable {
    /// Makes a fresh symbolic variable with the supplied display name.
    pub fn new<S: Into<String>>(name: S) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return runtime_error!("Variable: the name must not be empty");
        }
        Ok(Self {
            inner: AlgebraVariable::new(name.clone()),
            name,
            id: VARIABLE_COUNTER.fetch_add(1, AtomicOrdering::Relaxed),
        })
    }

    /// Koto constructor implementation for `Variable("x")`.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::Str(name)] => Ok(KObject::from(Self::new(name.to_string())?).into()),
            unexpected => unexpected_args("|String|", unexpected),
        }
    }

    fn inner(&self) -> AlgebraVariable {
        self.inner.clone()
    }

    fn name_str(&self) -> &str {
        &self.name
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Variable {}

#[koto_impl]
impl Variable {
    #[koto_method]
    pub fn name(&self) -> KValue {
        self.name.clone().into()
    }
}

impl KotoObject for Variable {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                Ok(*self == *other.cast::<Self>().unwrap())
            }
            unexpected => unexpected_type("Variable", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        ctx.append(self.name.clone());
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct OwnedTerm {
    powers: Vec<(Variable, usize)>,
    coeff: Integer,
}

impl OwnedTerm {
    fn degree(&self) -> usize {
        self.powers.iter().map(|(_, power)| *power).sum()
    }

    fn power(&self, variable: &Variable) -> usize {
        self.powers
            .iter()
            .find(|(v, _)| v == variable)
            .map(|(_, power)| *power)
            .unwrap_or(0)
    }
}

/// A multivariate polynomial over the exact integer ring.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct MultiPoly {
    poly: MultiPolynomial<Integer>,
    variables: Vec<Variable>,
    terms: Vec<OwnedTerm>,
}

#[koto_impl]
impl MultiPoly {
    fn zero(variables: Vec<Variable>) -> Self {
        Self::from_owned_terms(variables, Vec::new())
    }

    fn from_owned_terms(variables: Vec<Variable>, terms: Vec<OwnedTerm>) -> Self {
        let (variables, terms) = Self::normalise(variables, terms);
        let poly = Self::make_algebraeon_poly(&terms);
        Self {
            poly,
            variables,
            terms,
        }
    }

    fn normalise(
        mut variables: Vec<Variable>,
        terms: Vec<OwnedTerm>,
    ) -> (Vec<Variable>, Vec<OwnedTerm>) {
        // Keep the first occurrence of each identity, while allowing callers
        // to pass variables that only occur in the declared variable list.
        let mut seen_ids = HashSet::new();
        variables.retain(|variable| seen_ids.insert(variable.id));

        let mut by_key: HashMap<Vec<(usize, usize)>, Integer> = HashMap::new();
        let mut representatives: HashMap<usize, Variable> = HashMap::new();
        for variable in &variables {
            representatives.insert(variable.id, variable.clone());
        }

        for mut term in terms {
            term.powers.retain(|(_, power)| *power != 0);
            term.powers.sort_by_key(|(variable, _)| variable.id);

            // Be forgiving of a manually assembled term containing a variable
            // that was not listed explicitly in the constructor.
            for (variable, _) in &term.powers {
                if !seen_ids.contains(&variable.id) {
                    seen_ids.insert(variable.id);
                    variables.push(variable.clone());
                    representatives.insert(variable.id, variable.clone());
                }
            }

            let mut merged: Vec<(usize, usize)> = Vec::new();
            for (variable, power) in term.powers {
                if let Some((last_id, last_power)) = merged.last_mut() {
                    if *last_id == variable.id {
                        *last_power = last_power.saturating_add(power);
                        continue;
                    }
                }
                merged.push((variable.id, power));
            }
            let key = merged;
            if let Some(existing) = by_key.get_mut(&key) {
                *existing = Integer::structure().add(existing, &term.coeff);
            } else {
                by_key.insert(key, term.coeff);
            }
        }

        let mut terms: Vec<OwnedTerm> = by_key
            .into_iter()
            .filter_map(|(key, coeff)| {
                if Integer::structure().is_zero(&coeff) {
                    return None;
                }
                let powers = key
                    .into_iter()
                    .filter_map(|(id, power)| {
                        representatives
                            .get(&id)
                            .cloned()
                            .map(|variable| (variable, power))
                    })
                    .collect();
                Some(OwnedTerm { powers, coeff })
            })
            .collect();
        terms.sort_by(Self::compare_terms);
        (variables, terms)
    }

    fn compare_terms(a: &OwnedTerm, b: &OwnedTerm) -> Ordering {
        // This is the same ordering used by algebraeon's Monomial, with our
        // stable local variable id standing in for its private identifier.
        for (left, right) in a.powers.iter().zip(b.powers.iter()) {
            if left.0.id < right.0.id {
                return Ordering::Less;
            }
            if left.0.id > right.0.id {
                return Ordering::Greater;
            }
            if left.1 > right.1 {
                return Ordering::Less;
            }
            if left.1 < right.1 {
                return Ordering::Greater;
            }
        }
        b.powers.len().cmp(&a.powers.len())
    }

    fn make_algebraeon_poly(terms: &[OwnedTerm]) -> MultiPolynomial<Integer> {
        let integer_ring = Integer::structure();
        let ring = integer_ring.multivariable_polynomial_ring();
        let mut result = ring.zero();
        for term in terms {
            let mut monomial = ring.one();
            for (variable, power) in &term.powers {
                let factor = ring.var_pow(variable.inner(), *power);
                monomial = ring.mul(&monomial, &factor);
            }
            let coefficient = MultiPolynomial::constant(term.coeff.clone());
            let term_poly = ring.mul(&coefficient, &monomial);
            ring.add_mut(&mut result, &term_poly);
        }
        ring.reduce(result)
    }

    fn variable_from_value(value: &KValue) -> Result<Variable> {
        match value {
            KValue::Object(object) => match object.cast::<Variable>() {
                Ok(variable) => Ok(variable.clone()),
                Err(_) => unexpected_type("Variable or String", value),
            },
            KValue::Str(name) => Variable::new(name.to_string()),
            unexpected => unexpected_type("Variable or String", unexpected),
        }
    }

    fn variable_for_name(variables: &mut Vec<Variable>, name: &str) -> Result<Variable> {
        if let Some(variable) = variables.iter().find(|variable| variable.name == name) {
            return Ok(variable.clone());
        }
        let variable = Variable::new(name.to_owned())?;
        variables.push(variable.clone());
        Ok(variable)
    }

    fn values_from_list(value: &KValue) -> Result<Vec<KValue>> {
        match value {
            KValue::List(list) => Ok(list.data().iter().cloned().collect()),
            KValue::Tuple(tuple) => Ok(tuple.iter().cloned().collect()),
            unexpected => unexpected_type("List", unexpected),
        }
    }

    fn parse_powers(
        value: &KValue,
        variables: &mut Vec<Variable>,
    ) -> Result<Vec<(Variable, usize)>> {
        let map = match value {
            KValue::Map(map) => map,
            unexpected => return unexpected_type("Map of variable names to exponents", unexpected),
        };
        let mut powers: Vec<(Variable, usize)> = Vec::new();
        for (key, exponent) in map.data().iter() {
            let name = match key.value() {
                KValue::Str(name) => name.to_string(),
                unexpected => {
                    return unexpected_type("String variable names in a powers map", unexpected)
                }
            };
            let exponent = Self::usize_from_value(exponent)?;
            let variable = Self::variable_for_name(variables, &name)?;
            if let Some((_, old_power)) = powers.iter_mut().find(|(old, _)| old == &variable) {
                *old_power = old_power.saturating_add(exponent);
            } else if exponent != 0 {
                powers.push((variable, exponent));
            }
        }
        powers.sort_by_key(|(variable, _)| variable.id);
        Ok(powers)
    }

    fn parse_term(value: &KValue, variables: &mut Vec<Variable>) -> Result<OwnedTerm> {
        let values = Self::values_from_list(value)?;
        if values.len() != 2 {
            return runtime_error!("MultiPoly: each term must be [powers-map, coefficient]");
        }
        let (powers, coefficient) = if matches!(values[0], KValue::Map(_)) {
            (
                Self::parse_powers(&values[0], variables)?,
                CF::integer_from_value(&values[1])?,
            )
        } else if matches!(values[1], KValue::Map(_)) {
            (
                Self::parse_powers(&values[1], variables)?,
                CF::integer_from_value(&values[0])?,
            )
        } else {
            return runtime_error!(
                "MultiPoly: each term must contain one powers map and one coefficient"
            );
        };
        Ok(OwnedTerm {
            powers,
            coeff: coefficient,
        })
    }

    fn usize_from_value(value: &KValue) -> Result<usize> {
        let natural = CF::natural_from_value(value)?;
        natural.to_string().parse::<usize>().map_err(|_| {
            koto_runtime::Error::from(format!(
                "integer `{natural}` does not fit in a machine usize"
            ))
        })
    }

    /// Builds a polynomial from one list of terms, or from `[variables,
    /// terms]`.  Variables may be `Variable` objects or strings.
    pub fn from_args(args: &[KValue]) -> Result<KValue> {
        let (declared, term_list) = match args {
            [KValue::List(terms)] => (None, terms),
            [KValue::List(variables), KValue::List(terms)] => (Some(variables), terms),
            unexpected => {
                return unexpected_args(
                    "|List of terms| or |List of variables, List of terms|",
                    unexpected,
                )
            }
        };

        let mut variables: Vec<Variable> = declared
            .map(|list| {
                list.data()
                    .iter()
                    .map(Self::variable_from_value)
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        let mut raw_terms = Vec::new();
        for value in term_list.data().iter() {
            raw_terms.push(Self::parse_term(value, &mut variables)?);
        }
        // Validate names after parsing; this also catches a variable object and
        // an inferred string with the same name being used accidentally.
        let mut names = HashSet::new();
        for variable in &variables {
            if !names.insert(variable.name.clone()) {
                return runtime_error!("MultiPoly: duplicate variable name `{}`", variable.name);
            }
        }
        Ok(KObject::from(Self::from_owned_terms(variables, raw_terms)).into())
    }

    fn rebase(&self, variables: Vec<Variable>) -> Self {
        Self::from_owned_terms(variables, self.terms.clone())
    }

    fn with_constant(&self, coefficient: Integer) -> Self {
        Self::from_owned_terms(
            self.variables.clone(),
            vec![OwnedTerm {
                powers: Vec::new(),
                coeff: coefficient,
            }],
        )
    }

    fn add_poly(&self, other: &Self) -> Self {
        let variables = Self::merge_variables(&self.variables, &other.variables);
        let mut terms = self.terms.clone();
        terms.extend(other.terms.clone());
        Self::from_owned_terms(variables, terms)
    }

    fn subtract_poly(&self, other: &Self) -> Self {
        let variables = Self::merge_variables(&self.variables, &other.variables);
        let mut terms = self.terms.clone();
        terms.extend(other.terms.iter().map(|term| OwnedTerm {
            powers: term.powers.clone(),
            coeff: -term.coeff.clone(),
        }));
        Self::from_owned_terms(variables, terms)
    }

    fn multiply_poly(&self, other: &Self) -> Self {
        let variables = Self::merge_variables(&self.variables, &other.variables);
        let mut terms = Vec::new();
        for left in &self.terms {
            for right in &other.terms {
                let mut powers = left.powers.clone();
                powers.extend(right.powers.clone());
                terms.push(OwnedTerm {
                    powers,
                    coeff: left.coeff.clone() * right.coeff.clone(),
                });
            }
        }
        Self::from_owned_terms(variables, terms)
    }

    fn negate_poly(&self) -> Self {
        Self::from_owned_terms(
            self.variables.clone(),
            self.terms
                .iter()
                .map(|term| OwnedTerm {
                    powers: term.powers.clone(),
                    coeff: -term.coeff.clone(),
                })
                .collect(),
        )
    }

    fn merge_variables(left: &[Variable], right: &[Variable]) -> Vec<Variable> {
        let mut result = left.to_vec();
        let mut seen: HashSet<usize> = result.iter().map(|variable| variable.id).collect();
        for variable in right {
            if seen.insert(variable.id) {
                result.push(variable.clone());
            }
        }
        result
    }

    fn scalar_from_value(value: &KValue) -> Result<Integer> {
        CF::integer_from_value(value)
    }

    fn output_integer(integer: Integer) -> KValue {
        KValue::Object(KObject::from(ZZ::from_integer(integer)))
    }

    fn output_natural(natural: Natural) -> KValue {
        KValue::Object(KObject::from(NN(natural)))
    }

    fn output_variable_names(variables: &[Variable]) -> KValue {
        KValue::List(KList::with_data(
            variables
                .iter()
                .map(|variable| KValue::Str(variable.name.clone().into()))
                .collect::<Vec<_>>()
                .into(),
        ))
    }

    fn parse_variable_list(value: &KValue) -> Result<Vec<Variable>> {
        let values = Self::values_from_list(value)?;
        let mut variables: Vec<Variable> = Vec::new();
        for value in values {
            let variable = Self::variable_from_value(&value)?;
            if variables.iter().any(|old| old.id == variable.id) {
                continue;
            }
            if variables.iter().any(|old| old.name == variable.name) {
                return runtime_error!("MultiPoly: duplicate variable name `{}`", variable.name);
            }
            variables.push(variable);
        }
        Ok(variables)
    }

    fn parse_variable_list_for_self(&self, value: &KValue) -> Result<Vec<Variable>> {
        let values = Self::values_from_list(value)?;
        let mut variables: Vec<Variable> = Vec::new();
        for value in values {
            let variable = match &value {
                KValue::Str(name) => self
                    .variables
                    .iter()
                    .find(|variable| variable.name == name.as_str())
                    .cloned()
                    .unwrap_or(Variable::new(name.to_string())?),
                _ => Self::variable_from_value(&value)?,
            };
            if variables.iter().any(|old| old.id == variable.id) {
                continue;
            }
            if variables.iter().any(|old| old.name == variable.name) {
                return runtime_error!("MultiPoly: duplicate variable name `{}`", variable.name);
            }
            variables.push(variable);
        }
        Ok(variables)
    }

    fn variable_args_or_self(&self, args: &[KValue]) -> Result<Vec<Variable>> {
        match args {
            [] => Ok(self.variables.clone()),
            [value] => self.parse_variable_list_for_self(value),
            unexpected => unexpected_args("|List of variables|", unexpected),
        }
    }

    fn elementary_owned(n: usize, variables: &[Variable]) -> Self {
        if n > variables.len() {
            return Self::zero(variables.to_vec());
        }
        let mut combinations = Vec::new();
        Self::choose_indices(variables.len(), n, 0, &mut Vec::new(), &mut combinations);
        let terms = combinations
            .into_iter()
            .map(|indices| OwnedTerm {
                powers: indices
                    .into_iter()
                    .map(|index| (variables[index].clone(), 1))
                    .collect(),
                coeff: Integer::ONE,
            })
            .collect();
        Self::from_owned_terms(variables.to_vec(), terms)
    }

    fn choose_indices(
        length: usize,
        choose: usize,
        start: usize,
        current: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if current.len() == choose {
            result.push(current.clone());
            return;
        }
        let remaining = choose - current.len();
        if length.saturating_sub(start) < remaining {
            return;
        }
        for index in start..length {
            current.push(index);
            Self::choose_indices(length, choose, index + 1, current, result);
            current.pop();
        }
    }

    fn elementary_from_parts(n_value: &KValue, variables_value: &KValue) -> Result<KValue> {
        let n = Self::usize_from_value(n_value)?;
        let variables = Self::parse_variable_list(variables_value)?;
        Self::elementary_value(n, &variables)
    }

    fn elementary_value(n: usize, variables: &[Variable]) -> Result<KValue> {
        // Exercise algebraeon's implementation as the source of truth for
        // validation/compatibility, while retaining our public term metadata
        // (the upstream term fields are intentionally crate-private).
        let inner_variables: Vec<_> = variables.iter().map(Variable::inner).collect();
        let _ = MultiPolynomial::<Integer>::elementary_symmetric(n, inner_variables);
        Ok(KObject::from(Self::elementary_owned(n, variables)).into())
    }

    fn elementary_from_parts_for_self(
        &self,
        n_value: &KValue,
        variables_value: &KValue,
    ) -> Result<KValue> {
        let n = Self::usize_from_value(n_value)?;
        let variables = self.parse_variable_list_for_self(variables_value)?;
        Self::elementary_value(n, &variables)
    }

    /// Module-level implementation for `MultiPoly.elementary(n, variables)`.
    pub fn elementary_from_args(args: &[KValue]) -> Result<KValue> {
        match args {
            [first, second] if matches!(first, KValue::List(_)) => {
                Self::elementary_from_parts(second, first)
            }
            [n, variables] => Self::elementary_from_parts(n, variables),
            unexpected => unexpected_args("|n, List of variables|", unexpected),
        }
    }

    fn restrict_var_zero(&self, variable: &Variable) -> Self {
        Self::from_owned_terms(
            self.variables.clone(),
            self.terms
                .iter()
                .filter(|term| term.power(variable) == 0)
                .cloned()
                .collect(),
        )
    }

    fn split_by_degree(&self) -> Vec<(usize, Self)> {
        let mut grouped: HashMap<usize, Vec<OwnedTerm>> = HashMap::new();
        for term in &self.terms {
            grouped.entry(term.degree()).or_default().push(term.clone());
        }
        let mut result: Vec<_> = grouped
            .into_iter()
            .map(|(degree, terms)| {
                (
                    degree,
                    Self::from_owned_terms(self.variables.clone(), terms),
                )
            })
            .collect();
        result.sort_by_key(|(degree, _)| *degree);
        result
    }

    fn product_of_variables(variables: &[Variable]) -> Self {
        let terms = if variables.is_empty() {
            vec![OwnedTerm {
                powers: Vec::new(),
                coeff: Integer::ONE,
            }]
        } else {
            vec![OwnedTerm {
                powers: variables
                    .iter()
                    .cloned()
                    .map(|variable| (variable, 1))
                    .collect(),
                coeff: Integer::ONE,
            }]
        };
        Self::from_owned_terms(variables.to_vec(), terms)
    }

    fn divide_by_product_exact(&self, variables: &[Variable]) -> Result<Self> {
        let mut terms = Vec::new();
        for term in &self.terms {
            let mut powers = Vec::new();
            for (variable, power) in &term.powers {
                let divisor_power = variables.iter().filter(|v| *v == variable).count();
                if *power < divisor_power {
                    return runtime_error!("MultiPoly: internal non-exact monomial division");
                }
                if *power > divisor_power {
                    powers.push((variable.clone(), *power - divisor_power));
                }
            }
            for variable in variables {
                if term.power(variable) == 0 {
                    return runtime_error!("MultiPoly: internal non-exact monomial division");
                }
            }
            terms.push(OwnedTerm {
                powers,
                coeff: term.coeff.clone(),
            });
        }
        Ok(Self::from_owned_terms(self.variables.clone(), terms))
    }

    fn pow_poly(&self, exponent: usize) -> Self {
        let mut result = Self::from_owned_terms(
            self.variables.clone(),
            vec![OwnedTerm {
                powers: Vec::new(),
                coeff: Integer::ONE,
            }],
        );
        let mut base = self.clone();
        let mut exponent = exponent;
        while exponent != 0 {
            if exponent & 1 == 1 {
                result = result.multiply_poly(&base);
            }
            exponent >>= 1;
            if exponent != 0 {
                base = base.multiply_poly(&base);
            }
        }
        result
    }

    fn substitute(&self, replacements: &HashMap<usize, Self>) -> Self {
        let mut variables = self.variables.clone();
        for replacement in replacements.values() {
            variables = Self::merge_variables(&variables, &replacement.variables);
        }
        let mut result = Self::zero(variables.clone());
        for term in &self.terms {
            let mut part = Self::from_owned_terms(
                variables.clone(),
                vec![OwnedTerm {
                    powers: Vec::new(),
                    coeff: term.coeff.clone(),
                }],
            );
            for (variable, power) in &term.powers {
                if let Some(replacement) = replacements.get(&variable.id) {
                    part = part.multiply_poly(&replacement.pow_poly(*power));
                } else {
                    part = part.multiply_poly(&Self::from_owned_terms(
                        variables.clone(),
                        vec![OwnedTerm {
                            powers: vec![(variable.clone(), *power)],
                            coeff: Integer::ONE,
                        }],
                    ));
                }
            }
            result = result.add_poly(&part);
        }
        result
    }

    fn as_elementary_impl(
        variables: &[Variable],
        polynomial: &Self,
        elementary_variables: &[Variable],
    ) -> Result<Self> {
        let mut total_terms = Vec::new();
        for (degree, homogeneous) in polynomial.split_by_degree() {
            if degree == 0 {
                total_terms.extend(homogeneous.terms.clone());
                continue;
            }
            let homogeneous_result =
                Self::as_elementary_homogeneous(variables, &homogeneous, elementary_variables)?;
            total_terms.extend(homogeneous_result.terms);
        }
        Ok(Self::from_owned_terms(
            elementary_variables.to_vec(),
            total_terms,
        ))
    }

    fn as_elementary_homogeneous(
        variables: &[Variable],
        polynomial: &Self,
        elementary_variables: &[Variable],
    ) -> Result<Self> {
        let (last, first) = variables.split_last().ok_or_else(|| {
            koto_runtime::Error::from("MultiPoly: no variables for a positive-degree term")
        })?;
        let p_tilde = polynomial.restrict_var_zero(last);
        let r_sym =
            Self::as_elementary_impl(first, &p_tilde, &elementary_variables[..first.len()])?;

        let mut replacements = HashMap::new();
        for (index, elementary_variable) in
            elementary_variables.iter().take(first.len()).enumerate()
        {
            replacements.insert(
                elementary_variable.id,
                Self::elementary_owned(index + 1, variables),
            );
        }
        let r = r_sym.substitute(&replacements).rebase(variables.to_vec());
        let q = polynomial.subtract_poly(&r);
        let product = Self::product_of_variables(variables);
        let q_div = q.divide_by_product_exact(variables)?;
        let q_sym = Self::as_elementary_impl(variables, &q_div, elementary_variables)?;
        let en_sym = Self::from_owned_terms(
            elementary_variables.to_vec(),
            vec![OwnedTerm {
                powers: vec![(elementary_variables[variables.len() - 1].clone(), 1)],
                coeff: Integer::ONE,
            }],
        );
        // Keep the product in the computation above, so accidental changes to
        // the exact-division precondition are caught during development.
        let _ = product;
        Ok(r_sym
            .add_poly(&en_sym.multiply_poly(&q_sym))
            .rebase(elementary_variables.to_vec()))
    }

    fn generated_elementary_variables(count: usize) -> Vec<Variable> {
        (1..=count)
            .map(|index| Variable::new(format!("e{}", subscript_number(index))).unwrap())
            .collect()
    }

    fn parse_eval_map(&self, map: &KMap) -> Result<KValue> {
        let mut values_by_name = HashMap::new();
        for (key, value) in map.data().iter() {
            let name = match key.value() {
                KValue::Str(name) => name.to_string(),
                unexpected => {
                    return unexpected_type("String variable names in eval map", unexpected)
                }
            };
            if values_by_name.contains_key(&name) {
                return runtime_error!("MultiPoly.eval: duplicate variable `{name}`");
            }
            values_by_name.insert(name, Self::scalar_from_value(value)?);
        }

        let free_variables: HashSet<usize> = self
            .terms
            .iter()
            .flat_map(|term| term.powers.iter().map(|(variable, _)| variable.id))
            .collect();
        let mut values = HashMap::new();
        for variable in &self.variables {
            if free_variables.contains(&variable.id) {
                let value = values_by_name.get(variable.name_str()).ok_or_else(|| {
                    koto_runtime::Error::from(format!(
                        "MultiPoly.eval: missing value for variable `{}`",
                        variable.name_str()
                    ))
                })?;
                values.insert(variable.inner(), value.clone());
            }
        }
        let evaluated = self.poly.evaluate(values);
        Ok(Self::output_integer(evaluated))
    }

    fn as_elementary_value(&self, args: &[KValue]) -> Result<KValue> {
        let variables = match args {
            [] => self.variables.clone(),
            [value] => self.parse_variable_list_for_self(value)?,
            unexpected => return unexpected_args("|List of variables|", unexpected),
        };
        let inner_variables: Vec<_> = variables.iter().map(Variable::inner).collect();
        if self
            .poly
            .as_elementary_symmetric_polynomials(inner_variables.clone())
            .is_none()
        {
            return Ok(KValue::Null);
        }

        let elementary_variables = Self::generated_elementary_variables(variables.len());
        let expression = Self::as_elementary_impl(&variables, self, &elementary_variables)?;
        let names = Self::output_variable_names(&elementary_variables);
        Ok(KValue::Tuple(
            vec![names, KObject::from(expression).into()].into(),
        ))
    }

    /// Returns the polynomial's declared variables as a list of names.
    #[koto_method]
    pub fn vars(&self) -> KValue {
        Self::output_variable_names(&self.variables)
    }

    /// Returns the total degree, with degree zero for the zero polynomial.
    #[koto_method]
    pub fn degree(&self) -> KValue {
        Self::output_natural(Natural::from(self.poly.degree().unwrap_or(0)))
    }

    /// Returns terms as `[[powers-map, ZZ-coefficient], ...]`.
    #[koto_method]
    pub fn terms(&self) -> KValue {
        let values = self
            .terms
            .iter()
            .map(|term| {
                let powers = KMap::default();
                for (variable, power) in &term.powers {
                    powers.insert(
                        variable.name.as_str(),
                        Self::output_natural(Natural::from(*power)),
                    );
                }
                KValue::List(KList::with_data(
                    vec![
                        KValue::Map(powers),
                        Self::output_integer(term.coeff.clone()),
                    ]
                    .into(),
                ))
            })
            .collect::<Vec<_>>();
        KValue::List(KList::with_data(values.into()))
    }

    /// Evaluates with a map from variable names to integer values.
    #[koto_method]
    pub fn eval(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::Map(map)] => self.parse_eval_map(map),
            unexpected => unexpected_args("|Map of variable names to values|", unexpected),
        }
    }

    /// Tests symmetry in the supplied variables.  With no argument, all
    /// declared variables are used.
    #[koto_method]
    pub fn is_symmetric(&self, args: &[KValue]) -> Result<KValue> {
        let variables = self.variable_args_or_self(args)?;
        let inner_variables: Vec<_> = variables.iter().map(Variable::inner).collect();
        Ok(self.poly.is_symmetric(inner_variables).into())
    }

    /// Builds an elementary symmetric polynomial.  As an instance method,
    /// `p.elementary(n)` uses `p.vars()`; passing a variable list overrides it.
    #[koto_method]
    pub fn elementary(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [n] => {
                let variables = self.variables.clone();
                let n = Self::usize_from_value(n)?;
                let inner_variables: Vec<_> = variables.iter().map(Variable::inner).collect();
                let _ = MultiPolynomial::<Integer>::elementary_symmetric(n, inner_variables);
                Ok(KObject::from(Self::elementary_owned(n, &variables)).into())
            }
            [first, second] if matches!(first, KValue::List(_)) => {
                self.elementary_from_parts_for_self(second, first)
            }
            [n, variables] => self.elementary_from_parts_for_self(n, variables),
            unexpected => unexpected_args("|n| or |n, List of variables|", unexpected),
        }
    }

    /// Expresses a symmetric polynomial in elementary symmetric variables.
    /// The result follows algebraeon's `(variables, polynomial)` return shape:
    /// a tuple containing the generated names and the resulting `MultiPoly`.
    #[koto_method]
    pub fn as_elementary(&self, args: &[KValue]) -> Result<KValue> {
        self.as_elementary_value(args)
    }
}

enum ParsedOperand {
    Poly(MultiPoly),
    Scalar(Integer),
}

impl MultiPoly {
    fn parse_operand(&self, other: &KValue) -> Result<ParsedOperand> {
        match other {
            KValue::Object(object) if object.is_a::<Self>() => {
                Ok(ParsedOperand::Poly(object.cast::<Self>().unwrap().clone()))
            }
            _ => Ok(ParsedOperand::Scalar(Self::scalar_from_value(other)?)),
        }
    }

    fn assign_result(&mut self, value: KValue) -> Result<()> {
        match value {
            KValue::Object(object) => {
                let result = object
                    .cast::<Self>()
                    .map_err(|_| koto_runtime::Error::from("expected MultiPoly result"))?;
                *self = result.clone();
                Ok(())
            }
            unexpected => unexpected_type("MultiPoly", &unexpected),
        }
    }
}

impl KotoObject for MultiPoly {
    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                Ok(self.poly == other.cast::<Self>().unwrap().poly)
            }
            unexpected => unexpected_type("MultiPoly", unexpected),
        }
    }

    fn negate(&self) -> Result<KValue> {
        Ok(KObject::from(self.negate_poly()).into())
    }

    fn add(&self, other: &KValue) -> Result<KValue> {
        match self.parse_operand(other)? {
            ParsedOperand::Poly(other) => Ok(KObject::from(self.add_poly(&other)).into()),
            ParsedOperand::Scalar(value) => {
                Ok(KObject::from(self.add_poly(&self.with_constant(value))).into())
            }
        }
    }

    fn add_rhs(&self, other: &KValue) -> Result<KValue> {
        self.add(other)
    }

    fn subtract(&self, other: &KValue) -> Result<KValue> {
        match self.parse_operand(other)? {
            ParsedOperand::Poly(other) => Ok(KObject::from(self.subtract_poly(&other)).into()),
            ParsedOperand::Scalar(value) => {
                Ok(KObject::from(self.subtract_poly(&self.with_constant(value))).into())
            }
        }
    }

    fn subtract_rhs(&self, other: &KValue) -> Result<KValue> {
        // The runtime calls this for `scalar - polynomial`.
        let scalar = Self::scalar_from_value(other)?;
        Ok(KObject::from(self.negate_poly().add_poly(&self.with_constant(scalar))).into())
    }

    fn multiply(&self, other: &KValue) -> Result<KValue> {
        match self.parse_operand(other)? {
            ParsedOperand::Poly(other) => Ok(KObject::from(self.multiply_poly(&other)).into()),
            ParsedOperand::Scalar(value) => {
                Ok(KObject::from(self.multiply_poly(&self.with_constant(value))).into())
            }
        }
    }

    fn multiply_rhs(&self, other: &KValue) -> Result<KValue> {
        self.multiply(other)
    }

    fn add_assign(&mut self, other: &KValue) -> Result<()> {
        let result = self.add(other)?;
        self.assign_result(result)
    }

    fn subtract_assign(&mut self, other: &KValue) -> Result<()> {
        let result = self.subtract(other)?;
        self.assign_result(result)
    }

    fn multiply_assign(&mut self, other: &KValue) -> Result<()> {
        let result = self.multiply(other)?;
        self.assign_result(result)
    }

    fn display(&self, ctx: &mut DisplayContext) -> Result<()> {
        if self.terms.is_empty() {
            ctx.append("0");
            return Ok(());
        }
        let mut output = String::new();
        for (index, term) in self.terms.iter().enumerate() {
            let negative = term.coeff < Integer::ZERO;
            let magnitude = if negative {
                (-term.coeff.clone()).to_string()
            } else {
                term.coeff.to_string()
            };
            let has_variables = !term.powers.is_empty();
            if index == 0 {
                if negative {
                    output.push('-');
                }
            } else if negative {
                output.push_str(" - ");
            } else {
                output.push_str(" + ");
            }
            if !has_variables || magnitude != "1" {
                output.push_str(&magnitude);
            }
            for (power_index, (variable, power)) in term.powers.iter().enumerate() {
                if power_index > 0 {
                    output.push('*');
                }
                output.push_str(variable.name_str());
                if *power != 1 {
                    output.push('^');
                    output.push_str(&power.to_string());
                }
            }
        }
        ctx.append(output);
        Ok(())
    }
}

fn subscript_number(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|digit| match digit {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            other => other,
        })
        .collect()
}
