use super::Schema;
use derivative::Derivative;
use displaydoc::Display;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{self, Debug};
use std::{borrow::Cow, convert::TryFrom, hash::Hash};

/// A set of marks
#[derive(Derivative, Deserialize)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Default(bound = "")
)]
#[serde(bound = "", try_from = "Vec<S::Mark>")]
pub struct MarkSet<S: Schema> {
    content: Vec<S::Mark>,
}

impl<S: Schema> MarkSet<S> {
    /// Check whether the set contains this exact mark
    pub fn contains(&self, mark: &S::Mark) -> bool {
        self.content.contains(mark)
    }

    /// Add a mark to the set
    pub fn add(&mut self, mark: &S::Mark) {
        match self
            .content
            .binary_search_by_key(&mark.r#type(), Mark::r#type)
        {
            Ok(index) => {
                if &self.content[index] != mark {
                    self.content[index] = mark.clone();
                }
            }
            Err(index) => {
                self.content.insert(index, mark.clone());
            }
        }
    }

    /// Remove a mark from the set
    pub fn remove(&mut self, mark: &S::Mark) {
        match self
            .content
            .binary_search_by_key(&mark.r#type(), Mark::r#type)
        {
            Ok(index) => {
                self.content.remove(index);
            }
            Err(_index) => {}
        }
    }
}

impl<'a, S: Schema> IntoIterator for &'a MarkSet<S> {
    type Item = &'a S::Mark;
    type IntoIter = std::slice::Iter<'a, S::Mark>;
    fn into_iter(self) -> Self::IntoIter {
        self.content.iter()
    }
}

impl<S: Schema> Serialize for MarkSet<S> {
    fn serialize<Sr>(&self, serializer: Sr) -> Result<Sr::Ok, Sr::Error>
    where
        Sr: Serializer,
    {
        self.content.serialize(serializer)
    }
}

#[derive(Display)]
pub enum MarkSetError {
    /// Duplicate mark types
    Duplicates,
}

impl<S: Schema> TryFrom<Vec<S::Mark>> for MarkSet<S> {
    type Error = MarkSetError;
    fn try_from(mut value: Vec<S::Mark>) -> Result<Self, Self::Error> {
        let len = value.len();
        value.sort_by_key(|m| m.r#type());
        value.dedup_by_key(|m| m.r#type());
        if len > value.len() {
            Err(MarkSetError::Duplicates)
        } else {
            Ok(MarkSet { content: value })
        }
    }
}

impl<S: Schema> fmt::Debug for MarkSet<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.fmt(f)
    }
}

/// The methods that
pub trait Mark<S: Schema<Mark = Self>>:
    Serialize + for<'de> Deserialize<'de> + Debug + Clone + PartialEq + Eq + Hash
{
    /// The type of this mark.
    fn r#type(&self) -> S::MarkType;

    /// Given a set of marks, create a new set which contains this one as well, in the right
    /// position. If this mark is already in the set, the set itself is returned. If any marks that
    /// are set to be exclusive with this mark are present, those are replaced by this one.
    fn add_to_set<'a>(&self, set: Cow<'a, MarkSet<S>>) -> Cow<'a, MarkSet<S>> {
        match set
            .content
            .binary_search_by_key(&self.r#type(), Mark::r#type)
        {
            Ok(index) => {
                if &set.content[index] == self {
                    set
                } else {
                    let mut owned_set = set.into_owned();
                    owned_set.content[index] = self.clone();
                    Cow::Owned(owned_set)
                }
            }
            Err(index) => {
                let mut owned_set = set.into_owned();
                owned_set.content.insert(index, self.clone());
                Cow::Owned(owned_set)
            }
        }
    }

    /// Remove this mark from the given set, returning a new set. If this mark is not in the set,
    /// the set itself is returned.
    fn remove_from_set<'a>(&self, set: Cow<'a, MarkSet<S>>) -> Cow<'a, MarkSet<S>> {
        match set
            .content
            .binary_search_by_key(&self.r#type(), Mark::r#type)
        {
            Ok(index) => {
                let mut owned_set = set.into_owned();
                owned_set.content.remove(index);
                Cow::Owned(owned_set)
            }
            Err(_index) => set,
        }
    }

    /// Create a set with just this mark
    fn into_set(self) -> MarkSet<S> {
        MarkSet {
            content: vec![self],
        }
    }
}
