use crate::element::Element;
use crate::filter::Filter;

type Fingerprint = u64;

pub trait BasicQHT : Filter {

    fn get_fingerprint_from_bucket(&self, address: usize, bucket_number: usize) -> Fingerprint;

    fn insert_fingerprint_in_bucket(&mut self, address: usize, bucket_number: usize, fingerprint: Fingerprint);

    fn in_cell(&self, address: usize, fingerprint: Fingerprint) -> bool;

    fn get_fingerprint(&self, e: Element) -> Fingerprint; 

}