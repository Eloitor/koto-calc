use crate::NN::NN;
use crate::ZZ::ZZ;
use algebraeon::groups::examples::c2::C2;
use algebraeon::groups::permutation::Permutation;
use algebraeon::nzq::Integer;
use koto_runtime::{Result, derive::*, prelude::*};

/// Converts a koto value into a `usize` (accepts non-negative i64 Number,
/// NN and ZZ; negative values are rejected).
pub(crate) fn usize_from_value(value: &KValue) -> Result<usize> {
    match value {
        KValue::Number(n) if n.is_i64() => {
            let v = i64::from(*n);
            if v < 0 {
                return runtime_error!("expected a non-negative integer, got {}", v);
            }
            Ok(v as usize)
        }
        KValue::Object(object) => {
            if let Ok(nn) = object.cast::<NN>() {
                nn.0.clone()
                    .try_into()
                    .map_err(|_| koto_runtime::Error::from("number too large"))
            } else if let Ok(zz) = object.cast::<ZZ>() {
                let i = zz.to_integer();
                if i < Integer::ZERO {
                    runtime_error!("expected a non-negative integer, got {}", i)
                } else {
                    i.try_into()
                        .map_err(|_| koto_runtime::Error::from("number too large"))
                }
            } else {
                unexpected_type("Number, N natural, or Z integer", value)
            }
        }
        unexpected => unexpected_type("Number, N natural, or Z integer", unexpected),
    }
}

/// A permutation of {0, ..., n-1}, given by the list of images (indexed from
/// 0). `Perm([1, 2, 0])` sends 0 -> 1, 1 -> 2 and 2 -> 0.
///
/// Composition `p.compose(q)` (or `p * q`) applies `q` first and then `p`,
/// matching the algebraeon `PermutationCanonicalStructure::compose`.
#[derive(PartialEq, Clone, KotoCopy, KotoType, Eq, Debug)]
pub struct Perm {
    pub perm: Permutation,
}

/// Decomposes the permutation into disjoint cycles; fixed points (1-cycles)
/// are omitted, matching `Permutation::disjoint_cycles`.
fn disjoint_cycles(perm: &Permutation) -> Vec<Vec<usize>> {
    let n = perm.n();
    let mut visited = vec![false; n];
    let mut cycles = Vec::new();
    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut cycle = Vec::new();
        let mut x = start;
        while !visited[x] {
            visited[x] = true;
            cycle.push(x);
            x = perm.call(x);
        }
        if cycle.len() >= 2 {
            cycles.push(cycle);
        }
    }
    cycles
}

#[koto_impl]
impl Perm {
    /// Builds a Perm from a Koto list of images (indexed from 0). The list
    /// must contain every value of {0, ..., n-1} exactly once.
    pub fn from_koto_list(list: &KList) -> Result<KObject> {
        let mut images = Vec::new();
        for value in list.data().iter() {
            images.push(usize_from_value(value)?);
        }
        match Permutation::new(images) {
            Ok(perm) => Ok(KObject::from(Perm { perm })),
            Err(msg) => runtime_error!("Perm: {}", msg),
        }
    }

    /// Image of `x` under the permutation.
    #[koto_method]
    pub fn call(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [x] => {
                let x = usize_from_value(x)?;
                Ok(KValue::Number((self.perm.call(x) as i64).into()))
            }
            unexpected => unexpected_args("|x|", unexpected),
        }
    }

    /// Sign of the permutation: 1 for even permutations, -1 for odd ones.
    #[koto_method]
    pub fn sign(&self) -> KValue {
        match self.perm.sign() {
            C2::Identity => KValue::Number(1i64.into()),
            C2::Flip => KValue::Number((-1i64).into()),
        }
    }

    /// List of the disjoint cycles (each a list of elements); fixed points
    /// are omitted. E.g. `Perm([2, 0, 1, 4, 3]).cycles()` is
    /// `[[0, 2, 1], [3, 4]]`.
    #[koto_method]
    pub fn cycles(&self) -> KValue {
        let cycles = disjoint_cycles(&self.perm);
        let list: Vec<KValue> = cycles
            .into_iter()
            .map(|cycle| {
                KValue::List(KList::with_data(
                    cycle.into_iter().map(|x| KValue::Number((x as i64).into())).collect(),
                ))
            })
            .collect();
        KValue::List(KList::with_data(list.into()))
    }

    /// Cycle type: sorted list of the lengths of the disjoint cycles.
    #[koto_method]
    pub fn cycle_shape(&self) -> KValue {
        let mut shape = disjoint_cycles(&self.perm)
            .iter()
            .map(|c| c.len())
            .collect::<Vec<usize>>();
        shape.sort_unstable();
        KValue::List(KList::with_data(
            shape.into_iter().map(|x| KValue::Number((x as i64).into())).collect(),
        ))
    }

    /// Inverse permutation.
    #[koto_method]
    pub fn inverse(&self) -> KValue {
        let n = self.perm.n();
        let mut inv = vec![0usize; n];
        for i in 0..n {
            inv[self.perm.call(i)] = i;
        }
        KValue::Object(KObject::from(Perm {
            perm: Permutation::new_unchecked(inv),
        }))
    }

    /// Composition `self.compose(other)` = `self * other`: applies `other`
    /// first, then `self`.
    #[koto_method]
    pub fn compose(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [KValue::Object(other)] if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                let n = std::cmp::max(self.perm.n(), other.perm.n());
                let images = (0..n)
                    .map(|i| self.perm.call(other.perm.call(i)))
                    .collect();
                Ok(KValue::Object(KObject::from(Perm {
                    perm: Permutation::new_unchecked(images),
                })))
            }
            unexpected => unexpected_args("|Perm|", unexpected),
        }
    }
}

impl KotoObject for Perm {
    fn multiply(&self, other: &KValue) -> Result<KValue> {
        self.compose(&[other.clone()])
    }

    fn equal(&self, other: &KValue) -> Result<bool> {
        match other {
            KValue::Object(other) if other.is_a::<Self>() => {
                let other = other.cast::<Self>().unwrap();
                Ok(self.perm == other.perm)
            }
            unexpected => unexpected_type("Perm", unexpected),
        }
    }

    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        let n = self.perm.n();
        let mut s = String::from("[");
        for i in 0..n {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&self.perm.call(i).to_string());
        }
        s.push(']');
        ctx.append(s);
        Ok(())
    }
}
