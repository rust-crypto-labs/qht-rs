extern crate rand;
extern crate rust_dense_bitset;

mod filter;
mod basicqht;
mod element;
mod qht;
mod qqht;
mod qqhtd;

pub use crate::filter::Filter;
pub use crate::basicqht::BasicQHT;
pub use crate::element::Element;
pub use crate::qht::QuotientHashTable;
pub use crate::qqht::DQuotientHashTable;
pub use crate::qqhtd::DQQuotientHashTable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_qht(){
        let _qht = QuotientHashTable::new(1025,1,3);
    }
}
