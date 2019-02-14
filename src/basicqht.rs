use crate::filter::Filter;
pub use std::collections::hash_map::DefaultHasher;
pub use std::hash::{Hash, Hasher};

pub type Fingerprint = u64;

/// The `BasicQHT` trait collects common functionality between the different QHT flavours
pub trait BasicQHT: Filter {
    /// Obtains the fingerprint stored in a given bucket
    fn get_fingerprint_from_bucket(&self, address: usize, bucket_number: usize) -> Fingerprint;

    /// Inserts a fingerprint in a bucket
    fn insert_fingerprint_in_bucket(
        &mut self,
        address: usize,
        bucket_number: usize,
        fingerprint: Fingerprint,
    );

    /// Checks whether a fingerprint is in a cell
    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool;

    /// Obtains the fingerprint of an object
    fn get_fingerprint(&self, e: impl Hash) -> Fingerprint;
}

/// Returns the hash of (e, base, counter)
pub fn get_hash(e: impl Hash, base: u64, counter: u64) -> u64 {
    let mut s = DefaultHasher::new();
    e.hash(&mut s);
    base.hash(&mut s);
    counter.hash(&mut s);
    s.finish()
}

#[macro_export]
macro_rules! impl_basicqht {
    ($struct_type:ty) => {
        impl BasicQHT for $struct_type {
            /// Retrieves a fingerprint from a given bucket (provided as an `address` and `bucket_number`
            fn get_fingerprint_from_bucket(
                &self,
                address: usize,
                bucket_number: usize,
            ) -> Fingerprint {
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

            /// Obtains an element's fingerprint
            fn get_fingerprint(&self, e: impl Hash) -> Fingerprint {
                //let mut f = e;
                let mut fingerprint = 0;
                let mut counter = 0;

                while fingerprint == 0 {
                    let v = get_hash(&e, 2, counter);
                    fingerprint = (v % self.pow_fingerprint_size) as Fingerprint;
                    counter += 1;
                }
                fingerprint
            }
        }
    };
}
