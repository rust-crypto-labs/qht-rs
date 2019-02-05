use crate::element::Element;
use crate::filter::Filter;

pub type Fingerprint = u64;

pub trait BasicQHT: Filter {
    fn get_fingerprint_from_bucket(&self, address: usize, bucket_number: usize) -> Fingerprint;

    fn insert_fingerprint_in_bucket(
        &mut self,
        address: usize,
        bucket_number: usize,
        fingerprint: Fingerprint,
    );

    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool;

    fn get_fingerprint(&self, e: Element) -> Fingerprint;
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
    };
}
