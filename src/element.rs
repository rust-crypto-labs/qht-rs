pub use std::collections::hash_map::DefaultHasher;
pub use std::hash::{Hash, Hasher};

// --------------------------------------------------------------------------------
// Elements

/// This struct defines which elements are processed as stream elements
#[derive(Clone, Copy, Hash)]
pub struct Element {
    /// Value held by the element
    pub value: u64,
}

impl PartialEq for Element {
    /// Elements can be compared to check if their values are equal
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// Elements can be compared to check if their values are equal
impl Eq for Element {}
