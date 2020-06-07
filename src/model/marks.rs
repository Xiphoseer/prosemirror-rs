use super::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// A set of marks
#[allow(type_alias_bounds)]
pub type MarkSet<S: Schema> = HashSet<S::Mark>;

/// The methods that
pub trait Mark:
    Serialize + for<'de> Deserialize<'de> + Debug + Clone + PartialEq + Eq + Hash
{
}
