use crate::element::Element;
// --------------------------------------------------------------------------------
// Filter

/// A `Filter` provides duplicate detection capabilities
pub trait Filter {
    /// Performs a lookup for the provided element
    fn lookup(&self, e: Element) -> bool;

    /// Performs a lookup for the provided element and inserts it
    fn insert(&mut self, e: Element) -> bool;
}
