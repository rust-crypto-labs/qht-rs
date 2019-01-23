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

/// Quotient Hash Table ("compact")
///
/// This implements QHTc, using a dense bitset as the underlying data structure
pub struct QuotientHashTable {
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

    /// Random number generator
    rng: StdRng,
}

/// QHTc implementation
impl QuotientHashTable {
    /// Returns a newly created `QuotientHashTable` or panics
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
    /// use qht::QuotientHashTable;
    /// let f = QuotientHashTable::new(1024, 1, 3);
    /// ```
    ///
    pub fn new(memory_size: usize, n_buckets: usize, fingerprint_size: usize) -> Self {
        // Fingerprint size is limited
        if fingerprint_size > FINGERPRINT_SIZE_LIMIT {
            panic!("[QHTc Filter] Incorrect parameters, fingerprint_size cannot exceed 8.");
        } else if fingerprint_size == 0 {
            panic!("[QHTc Filter] Incorrect parameters, fingerprint_size cannot be zero.");
        }

        // At least one bucket is required
        if n_buckets == 0 {
            panic!("[QHTc Filter] Incorrect parameters, n_buckets cannot be zero.");
        }

        let rng = StdRng::from_entropy();
        let pow_fingerprint_size = 2u64.pow(fingerprint_size as u32);
        let n_cells = memory_size / (n_buckets * fingerprint_size);

        // There should be at least one cell
        if n_cells == 0 {
            panic!("[QHT Filter] Incorrect parameters, memory size should be at least n_buckets * fingerprint_size");
        }

        // Initialise the vector with the appropriate length
        let qht = DenseBitSetExtended::with_capacity(n_cells * n_buckets * fingerprint_size);

        Self {
            n_cells,
            n_buckets,
            fingerprint_size,
            pow_fingerprint_size,
            qht,
            rng,
        }
    }

    /// Retrieves a fingerprint from a given bucket (provided as an `address` and `bucket_number`
    fn get_fingerprint_from_bucket(&self, address: usize, bucket_number: usize) -> Fingerprint {
        let offset = (address * self.n_buckets + bucket_number) * self.fingerprint_size;

        self.qht.extract_u64(offset, self.fingerprint_size)
    }

    /// Inserts a fingerprint in a given buffer (provided as an `address` and `bucket_number`
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

    /// Checks whether a given fingerprint belongs to a given bucket
    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool {
        for idx in 0..self.n_buckets {
            if self.get_fingerprint_from_bucket(address, idx) == fingerprint {
                return true;
            }
        }
        false
    }

    /// Returns a randomly chosen bucket
    fn get_random_bucket(&mut self) -> usize {
        self.rng.gen_range(0, self.n_buckets)
    }

    /// Inserts the fingerprint in the first empty bucket
    fn insert_empty(&mut self, address: usize, fingerprint: Fingerprint) -> bool {
        for idx in 0..self.n_buckets {
            if self.get_fingerprint_from_bucket(address, idx) == 0 {
                self.insert_fingerprint_in_bucket(address, idx, fingerprint);
                return true;
            }
        }
        false
    }

    /// Obtains an element's fingerprint
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
}

impl Filter for QuotientHashTable {
    /// Performs a lookup for the provided element
    ///
    /// # Example
    /// ```rust
    /// use qht::{Element, Filter, QuotientHashTable};
    /// let f = QuotientHashTable::new(1024, 1, 3);
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
    /// Note: In this implementation, if the element is already present, it is not inserted
    /// # Example
    /// ```rust
    /// use qht::{Element,Filter, QuotientHashTable};
    /// let mut f = QuotientHashTable::new(1024, 1, 3);
    /// let e = Element { value: 1234 };
    /// let was_present = f.insert(e);
    /// assert!( f.lookup(e) ); // The filter now contains e
    /// assert!( !was_present ); // The filter did not previously contain e
    /// ```
    fn insert(&mut self, e: Element) -> bool {
        let fingerprint = self.get_fingerprint(e);
        let address = (e.get_hash(1) as usize) % self.n_cells;

        if self.in_cell(address, fingerprint) {
            return true;
        }

        if !self.insert_empty(address, fingerprint) {
            let bucket = self.get_random_bucket();
            self.insert_fingerprint_in_bucket(address, bucket, fingerprint);
        }

        false
    }
}
