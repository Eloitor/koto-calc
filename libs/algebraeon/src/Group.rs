use crate::Perm::usize_from_value;
use algebraeon::groups::composition_table::group::{FiniteGroupMultiplicationTable, examples};
use koto_runtime::{Result, derive::*, prelude::*};
use std::sync::Arc;

/// A finite group given by its multiplication table, together with a
/// human-readable name (`C4`, `D3`, `S3`, `A4`, `K4`, `Q8`, ...).
///
/// Elements are the indices `0 .. size-1` of the table; the identity is `0`.
#[derive(Clone, KotoCopy, KotoType, Debug)]
pub struct Group {
    pub table: Arc<FiniteGroupMultiplicationTable>,
    pub name: String,
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.table, &other.table) && self.name == other.name
    }
}

impl Eq for Group {}

fn group_object(table: FiniteGroupMultiplicationTable, name: String) -> KObject {
    KObject::from(Group {
        table: Arc::new(table),
        name,
    })
}

#[koto_impl]
impl Group {
    /// Cyclic group C_n.
    pub fn cyclic(n: usize) -> Result<KObject> {
        if n < 1 {
            return runtime_error!("Group.cyclic: n must be at least 1, got {}", n);
        }
        Ok(group_object(examples::cyclic_group_structure(n), format!("C{}", n)))
    }

    /// Dihedral group D_n (symmetries of a regular n-gon, size 2n).
    pub fn dihedral(n: usize) -> Result<KObject> {
        if n < 1 {
            return runtime_error!("Group.dihedral: n must be at least 1, got {}", n);
        }
        Ok(group_object(
            examples::dihedral_group_structure(n),
            format!("D{}", n),
        ))
    }

    /// Symmetric group S_n (permutations of n elements, size n!).
    pub fn symmetric(n: usize) -> Result<KObject> {
        if n < 1 {
            return runtime_error!("Group.symmetric: n must be at least 1, got {}", n);
        }
        Ok(group_object(
            examples::symmetric_group_structure(n),
            format!("S{}", n),
        ))
    }

    /// Alternating group A_n (even permutations of n elements, size n!/2).
    pub fn alternating(n: usize) -> Result<KObject> {
        if n < 1 {
            return runtime_error!("Group.alternating: n must be at least 1, got {}", n);
        }
        Ok(group_object(
            examples::alternating_group_structure(n),
            format!("A{}", n),
        ))
    }

    /// Klein four-group V4 (size 4, abelian).
    pub fn klein4() -> Result<KObject> {
        Ok(group_object(
            examples::klein_four_structure(),
            "K4".to_string(),
        ))
    }

    /// Quaternion group Q8 (size 8, non-abelian).
    pub fn quaternion() -> Result<KObject> {
        Ok(group_object(
            examples::quaternion_group_structure(),
            "Q8".to_string(),
        ))
    }

    /// Trivial group (size 1).
    pub fn trivial() -> Result<KObject> {
        Ok(group_object(
            examples::trivial_group_structure(),
            "1".to_string(),
        ))
    }

    /// Number of elements of the group.
    #[koto_method]
    pub fn size(&self) -> KValue {
        KValue::Number((self.table.size() as i64).into())
    }

    /// Order of the element with the given index.
    #[koto_method]
    pub fn order(&self, args: &[KValue]) -> Result<KValue> {
        match args {
            [x] => {
                let x = usize_from_value(x)?;
                match self.table.order(x) {
                    Ok(ord) => Ok(KValue::Number((ord as i64).into())),
                    Err(()) => runtime_error!(
                        "Group.order: element index {} is out of range (group size {})",
                        x,
                        self.table.size()
                    ),
                }
            }
            unexpected => unexpected_args("|element|", unexpected),
        }
    }

    /// Whether the group is abelian (commutative).
    #[koto_method]
    pub fn is_abelian(&self) -> KValue {
        KValue::Bool(self.table.is_abelian())
    }

    /// Conjugacy classes of the group, as a list of lists of element
    /// indices. Classes are sorted by their smallest element (and each class
    /// internally), so the result is deterministic.
    #[koto_method]
    pub fn conjugacy_classes(&self) -> KValue {
        let partition = self.table.conjugacy_classes();
        let mut classes: Vec<Vec<usize>> = Vec::new();
        for i in 0..partition.size() {
            let mut class: Vec<usize> =
                partition.partition.get_class(i).iter().copied().collect();
            class.sort_unstable();
            classes.push(class);
        }
        classes.sort_unstable_by_key(|class| class[0]);
        let classes: Vec<KValue> = classes
            .into_iter()
            .map(|class| {
                KValue::List(KList::with_data(
                    class.into_iter().map(|x| KValue::Number((x as i64).into())).collect(),
                ))
            })
            .collect();
        KValue::List(KList::with_data(classes.into()))
    }
}

impl KotoObject for Group {
    fn display(&self, ctx: &mut DisplayContext) -> koto_runtime::Result<()> {
        ctx.append(format!("{} (size {})", self.name, self.table.size()));
        Ok(())
    }
}
