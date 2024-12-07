//! An ordered set of [`XpathItem`]s.

use std::ops::Index;

use indexmap::{self, IndexSet};

use super::grammar::data_model::{AnyAtomicType, XpathItem};

/// An ordered set of [`XpathItem`]s.
#[derive(Debug)]
pub struct XpathItemSet<'tree> {
    index_set: IndexSet<XpathItem<'tree>>,
}

impl PartialEq for XpathItemSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.index_set == other.index_set
    }
}

// impl PartialOrd for XpathItemSet<'_> {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.index_set.iter().partial_cmp(&other.index_set)
//     }
// }

impl<'a, 'tree> IntoIterator for &'a XpathItemSet<'tree> {
    type Item = &'a XpathItem<'tree>;

    type IntoIter = indexmap::set::Iter<'a, XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.iter()
    }
}

impl<'tree> IntoIterator for XpathItemSet<'tree> {
    type Item = XpathItem<'tree>;

    type IntoIter = indexmap::set::IntoIter<XpathItem<'tree>>;

    fn into_iter(self) -> Self::IntoIter {
        self.index_set.into_iter()
    }
}

impl<'tree> FromIterator<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn from_iter<T: IntoIterator<Item = XpathItem<'tree>>>(iter: T) -> Self {
        let index_set = IndexSet::from_iter(iter);
        XpathItemSet { index_set }
    }
}

impl<'tree> Extend<XpathItem<'tree>> for XpathItemSet<'tree> {
    fn extend<T: IntoIterator<Item = XpathItem<'tree>>>(&mut self, iter: T) {
        self.index_set.extend(iter)
    }
}

impl<'tree> XpathItemSet<'tree> {
    /// Create a new empty [`XpathItemSet`].
    pub fn new() -> Self {
        XpathItemSet {
            index_set: IndexSet::new(),
        }
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.index_set.is_empty()
    }

    /// The number of items in the set.
    pub fn len(&self) -> usize {
        self.index_set.len()
    }

    /// Inserts a new item into the set.
    ///
    /// Returns true if the item was inserted, false if it was already present.
    pub fn insertb(&mut self, item: XpathItem<'tree>) -> bool {
        self.index_set.insert(item)
    }

    /// Inserts a new item into the set.
    ///
    /// Drops the bool returned by [`XpathItemSet::insertb`] so that it can be used in match arms
    /// without causing incompatible types with [`XpathItemSet::extend`].
    pub fn insert(&mut self, item: XpathItem<'tree>) {
        self.insertb(item);
    }

    /// Return an iterator over the items in the set.
    pub fn iter(&self) -> indexmap::set::Iter<'_, XpathItem<'tree>> {
        self.index_set.iter()
    }

    /// Return the effective boolean value of the result.
    ///
    /// <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-ebv>
    pub fn boolean(&self) -> bool {
        // If this is a singleton value, check for the effective boolean value of that value.
        if self.index_set.len() == 1 {
            match &self.index_set[0] {
                XpathItem::Node(_) => true,
                XpathItem::Function(_) => true,
                XpathItem::AnyAtomicType(atomic_type) => match atomic_type {
                    AnyAtomicType::Boolean(b) => *b,
                    AnyAtomicType::Integer(n) => *n != 0,
                    AnyAtomicType::Float(n) => *n != 0.0,
                    AnyAtomicType::Double(n) => *n != 0.0,
                    AnyAtomicType::String(s) => !s.is_empty(),
                },
            }
        }
        // Otherwise, the effective boolean value is true if the sequence contains any items.
        else {
            !self.index_set.is_empty()
        }
    }

    // pub(crate) fn sort(&mut self) {
    //     self.index_set.sort();
    // }
}

impl<'tree> From<IndexSet<XpathItem<'tree>>> for XpathItemSet<'tree> {
    fn from(value: IndexSet<XpathItem<'tree>>) -> Self {
        XpathItemSet { index_set: value }
    }
}

impl<'tree> Index<usize> for XpathItemSet<'tree> {
    type Output = XpathItem<'tree>;

    fn index(&self, index: usize) -> &Self::Output {
        self.index_set.index(index)
    }
}

/// Create an [XpathItemSet] from a list of values
#[macro_export]
macro_rules! xpath_item_set {
    ($($value:expr,)+) => { $crate::xpath::xpath_item_set::xpath_item_set!($($value),+) };
    ($($value:expr),*) => {
        {
            let set = indexmap::indexset![$($value,)*];

            crate::xpath::XpathItemSet::from(set)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::xpath::grammar::data_model::AnyAtomicType;

    use super::*;

    #[test]
    fn macro_works_with_one() {
        // arrange
        let node1 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("1")));

        // act
        let item_set = xpath_item_set![node1.clone()];

        // assert
        let mut expected = XpathItemSet::new();
        expected.insert(node1);

        assert_eq!(item_set, expected);
    }

    #[test]
    fn macro_works_with_multiple() {
        // arrange
        let node1 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("1")));
        let node2 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("2")));
        let node3 = XpathItem::AnyAtomicType(AnyAtomicType::String(String::from("3")));

        // act
        let item_set = xpath_item_set![node1.clone(), node2.clone(), node3.clone()];

        // assert
        let mut expected = XpathItemSet::new();
        expected.insert(node1);
        expected.insert(node2);
        expected.insert(node3);

        assert_eq!(item_set, expected);
    }
}
