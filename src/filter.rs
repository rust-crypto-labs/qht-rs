pub use std::hash::Hash;
// --------------------------------------------------------------------------------
// Filter

/// A `Filter` provides duplicate detection capabilities
pub trait Filter {
    /// Performs a lookup for the provided element
    fn lookup(&self, e: impl Hash) -> bool;

    /// Performs a lookup for the provided element and inserts it
    fn insert(&mut self, e: impl Hash) -> bool;
}
