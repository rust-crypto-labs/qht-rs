pub use std::collections::hash_map::DefaultHasher;
pub use std::hash::{Hash, Hasher};

// --------------------------------------------------------------------------------
// Elements

/// This struct defines which elements are processed as stream elements
#[derive(Clone, Copy)]
pub struct Element {
    /// Value held by the element
    pub value: u64,
}

impl Element {
    /// Returns a hash of the element's value
    ///
    /// It takes an argument seed that provides access to independent hash functions
    pub fn get_hash(self, seed: u64) -> u64 {
        let mut s = DefaultHasher::new();
        self.value.hash(&mut s);
        seed.hash(&mut s);
        s.finish()
    }
}

impl PartialEq for Element {
    /// Elements can be compared to check if their values are equal
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// Elements can be compared to check if their values are equal
impl Eq for Element {}