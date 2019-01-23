use crate::filter::Filter;
use crate::element::Element;

pub use rand::rngs::StdRng;
pub use rand::{FromEntropy, Rng};
pub use std::collections::hash_map::DefaultHasher;
pub use std::hash::{Hash, Hasher};

pub use rust_dense_bitset::DenseBitSetExtended;

// --------------------------------------------------------------------------------
// Configuration

type Fingerprint = u64;
const FINGERPRINT_SIZE_LIMIT: usize = 8;

// --------------------------------------------------------------------------------

/// qQuotient Hash Table D ("compact")
///
/// This implements dqQHTc, using a dense bitset as the underlying data structure
pub struct DQQuotientHashTable {
    /// Number of cells (automatically computed)
    n_cells: usize,

    /// Number of buckets
    n_buckets: usize,

    /// Size of the fingerprint (in bits)
    fingerprint_size: usize,

    /// Size of the fingerprint (positional, automatically computed)
    pow_fingerprint_size: u64,

    /// Underlying data structure
    //qht: Vec<bool>,
    qht: DenseBitSetExtended,
}

/// qQHTcd implementation
impl DQQuotientHashTable {
    /// Returns a a newly created `DQQuotientHashTable` or panics
    ///
    /// This function takes as arguments:
    /// * `memory_size`: allocated memory for the filter, in bits
    /// * `n_buckets`: number of buckets
    /// * `fingerprint_size`: size of each fingerprint, in bits. Cannot exceed `FINGERPRINT_SIZE_LIMIT`.
    ///
    /// Parameters should be chosen in a consistent way, namely so that `memory_size` >= `n_buckets` * `fingerprint_size`
    ///
    /// # Example
    /// ```rust
    /// use qht::DQQuotientHashTable;
    /// let f = DQQuotientHashTable::new(1024, 1, 3);
    /// ```

    pub fn new(memory_size: usize, n_buckets: usize, fingerprint_size: usize) -> Self {
        if fingerprint_size > FINGERPRINT_SIZE_LIMIT {
            panic!("[qQHTcd Filter] Incorrect parameters, fingerprint_size cannot exceed 8.");
        } else if fingerprint_size == 0 {
            panic!("[QHTc Filter] Incorrect parameters, fingerprint_size cannot be zero.");
        }

        // At least one bucket is required
        if n_buckets == 0 {
            panic!("[QHTc Filter] Incorrect parameters, n_buckets cannot be zero.");
        }

        let pow_fingerprint_size = 2u64.pow(fingerprint_size as u32);
        let n_cells = memory_size / (n_buckets * fingerprint_size);

        let qht = DenseBitSetExtended::with_capacity(n_cells * n_buckets * fingerprint_size);

        if n_cells == 0 {
            panic!("[QHT Filter] Incorrect parameters, memory size should be at least n_buckets * fingerprint_size");
        }

        Self {
            n_cells,
            n_buckets,
            fingerprint_size,
            pow_fingerprint_size,
            qht,
        }
    }

    fn get_fingerprint_from_bucket(&self, address: usize, bucket_number: usize) -> Fingerprint {
        let offset = (address * self.n_buckets + bucket_number) * self.fingerprint_size;

        self.qht.extract_u64(offset, self.fingerprint_size)
    }

    fn insert_fingerprint_in_bucket(
        &mut self,
        address: usize,
        bucket_number: usize,
        fingerprint: Fingerprint,
    ) {
        let offset = (address * self.n_buckets + bucket_number) * self.fingerprint_size;
        self.qht
            .insert_u64(fingerprint, offset, self.fingerprint_size);
    }

    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool {
        for idx in 0..self.n_buckets {
            if self.get_fingerprint_from_bucket(address, idx) == fingerprint {
                return true;
            }
        }
        false
    }

    fn get_fingerprint(&self, e: Element) -> Fingerprint {
        let mut f = e;
        let mut fingerprint = 0;

        while fingerprint == 0 {
            let v = f.get_hash(2);
            fingerprint = (v % self.pow_fingerprint_size) as Fingerprint;
            f.value += 1;
        }
        fingerprint
    }

    fn insert_fingerprint_in_last_bucket(&mut self, address: usize, fingerprint: Fingerprint) {
        for prev in 0..(self.n_buckets - 1) {
            let idx = prev + 1;
            let fg = self.get_fingerprint_from_bucket(address, idx);
            self.insert_fingerprint_in_bucket(address, prev, fg);
        }
        let last_bucket = self.n_buckets - 1;
        self.insert_fingerprint_in_bucket(address, last_bucket, fingerprint)
    }
}

impl Filter for DQQuotientHashTable {
    /// Performs a lookup for the provided element
    ///
    /// # Example
    /// ```rust
    /// use qht::{Element, Filter, DQQuotientHashTable};
    /// let f = DQQuotientHashTable::new(1024, 1, 3);
    /// let e = Element { value: 1234 };
    /// assert!( !f.lookup(e) ); // The filter is empty
    /// ```
    fn lookup(&self, e: Element) -> bool {
        let fingerprint = self.get_fingerprint(e);
        let address = (e.get_hash(1) as usize) % self.n_cells;
        self.in_cell(address, fingerprint)
    }

    /// Performs a lookup for an element and inserts it
    ///
    /// Note: In this implementation, if the element is already present, it is re-inserted.
    ///
    /// Note: The new element is inserted in the last bucket (not a random bucket)
    /// # Example
    /// ```rust
    /// use qht::{Element,Filter, DQQuotientHashTable};
    /// let mut f = DQQuotientHashTable::new(1024, 1, 3);
    /// let e = Element { value: 1234 };
    /// let was_present = f.insert(e);
    /// assert!( f.lookup(e) ); // The filter now contains e
    /// assert!( !was_present ); // The filter did not previously contain e
    /// ```

    fn insert(&mut self, e: Element) -> bool {
        let fingerprint = self.get_fingerprint(e);
        let address = (e.get_hash(1) as usize) % self.n_cells;

        let detected = self.in_cell(address, fingerprint);

        self.insert_fingerprint_in_last_bucket(address, fingerprint);

        detected
    }
}