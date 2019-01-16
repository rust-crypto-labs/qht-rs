extern crate rand;
extern crate rust_dense_bitset;

use self::rand::rngs::StdRng;
use self::rand::{FromEntropy, Rng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rust_dense_bitset::DenseBitSetExtended;

// --------------------------------------------------------------------------------
// Configuration

type Fingerprint = u64;
const FINGERPRINT_SIZE_LIMIT: usize = 8;

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
    fn get_hash(self, seed: u64) -> u64 {
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

// --------------------------------------------------------------------------------
// Filter

/// A `Filter` provides duplicate detection capabilities
pub trait Filter {
    /// Performs a lookup for the provided element
    fn lookup(&self, e: Element) -> bool;

    /// Performs a lookup for the provided element and inserts it
    fn insert(&mut self, e: Element) -> bool;
}

// --------------------------------------------------------------------------------
// QHTc, QHTcd, qQHTcd : fields

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

/// Quotient Hash Table D ("compact")
///
/// This implements dQHTc, using a dense bitset as the underlying data structure
pub struct DQuotientHashTable {
    /// Number of cells (automatically computed)
    n_cells: usize,

    /// Number of buckets
    n_buckets: usize,

    /// Size of the fingerprint (in bits)
    fingerprint_size: usize,

    /// Size of the fingerprint (positional, automatically computed)
    pow_fingerprint_size: u64,

    /// Underlying data structure
    //    qht: Vec<bool>,
    qht: DenseBitSetExtended,

    /// Random number generator
    rng: StdRng,
}

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

// --------------------------------------------------------------------------------
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

// --------------------------------------------------------------------------------
/// QHTcd implementation
impl DQuotientHashTable {
    /// Returns a a newly created `DQuotientHashTable` or panics
    ///
    /// This function takes as arguments:
    /// * `memory_size`: allocated memory for the filter, in bits
    /// * `n_buckets`: number of buckets
    /// * `fingerprint_size`: size of each fingerprint, in bits. Cannot exceed `FINGERPRINT_SIZE_LIMIT`.
    ///
    /// Parameters should be chosen in a consistent way, namely so that `memory_size` >= `n_buckets` * `fingerprint_size`
    /// # Example
    /// ```rust
    /// use qht::DQuotientHashTable;
    /// let f = DQuotientHashTable::new(1024, 1, 3);
    /// ```
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

    /// Checks whether a fingerprint belongs to a given cell
    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool {
        for idx in 0..self.n_buckets {
            if self.get_fingerprint_from_bucket(address, idx) == fingerprint {
                return true;
            }
        }
        false
    }

    // Returns a random bucket
    fn get_random_bucket(&mut self) -> usize {
        self.rng.gen_range(0, self.n_buckets)
    }

    /// Inserts the fingerprirnt in the first empty bucket
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

impl Filter for DQuotientHashTable {
    /// Performs a lookup for the provided element
    ///
    /// # Example
    /// ```rust
    /// use qht::{Element, Filter, DQuotientHashTable};
    /// let f = DQuotientHashTable::new(1024, 1, 3);
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
    ///
    /// # Example
    /// ```rust
    /// use qht::{Element,Filter, DQuotientHashTable};
    /// let mut f = DQuotientHashTable::new(1024, 1, 3);
    /// let e = Element { value: 1234 };
    /// let was_present = f.insert(e);
    /// assert!( f.lookup(e) ); // The filter now contains e
    /// assert!( !was_present ); // The filter did not previously contain e
    /// ```

    fn insert(&mut self, e: Element) -> bool {
        let fingerprint = self.get_fingerprint(e);
        let address = (e.get_hash(1) as usize) % self.n_cells;

        let detected = self.in_cell(address, fingerprint);

        if !self.insert_empty(address, fingerprint) {
            let bucket = self.get_random_bucket();
            self.insert_fingerprint_in_bucket(address, bucket, fingerprint);
        }

        detected
    }
}

// --------------------------------------------------------------------------------
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
