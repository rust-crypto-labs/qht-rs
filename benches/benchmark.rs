#[macro_use]
extern crate criterion;
extern crate qht;
extern crate rand;

mod benchmarks {

    const MAX_ELEMENT_VALUE: u64 = 1_000_000;
    const MEMORY_SIZE: usize = 100_000;
    const NUM_BUCKETS: usize = 5;
    const FINGERPRINT_SIZE: usize = 3;

    use criterion::Criterion;
    use qht::{DQQuotientHashTable, DQuotientHashTable, Element, Filter, QuotientHashTable};

    use rand::{rngs::StdRng, FromEntropy, RngCore};

    pub fn bench_rng(c: &mut Criterion) {
        let mut rng = StdRng::from_entropy();

        c.bench_function("RNG", move |b| {
            b.iter(|| {
                let e = Element {
                    value: rng.next_u64() % MAX_ELEMENT_VALUE,
                };
                e
            })
        });
    }

    pub fn bench_new_qht(c: &mut Criterion) {
        c.bench_function("QuotientHashTable::new", |b| {
            b.iter(|| QuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE))
        });
    }

    pub fn bench_new_dqht(c: &mut Criterion) {
        c.bench_function("DQuotientHashTable::new", |b| {
            b.iter(|| DQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE))
        });
    }

    pub fn bench_new_dqqht(c: &mut Criterion) {
        c.bench_function("DQQuotientHashTable::new", |b| {
            b.iter(|| DQQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE))
        });
    }

    pub fn bench_insert_qht(c: &mut Criterion) {
        let mut f = QuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("QuotientHashTable::insert", move |b| b.iter(|| f.insert(e)));
    }

    pub fn bench_insert_dqht(c: &mut Criterion) {
        let mut f = DQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("DQuotientHashTable::insert", move |b| {
            b.iter(|| f.insert(e))
        });
    }

    pub fn bench_insert_dqqht(c: &mut Criterion) {
        let mut f = DQQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("DQQuotientHashTable::insert", move |b| {
            b.iter(|| f.insert(e))
        });
    }

    pub fn bench_lookup_qht(c: &mut Criterion) {
        let f = QuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("QuotientHashTable::lookup", move |b| b.iter(|| f.lookup(e)));
    }

    pub fn bench_lookup_dqht(c: &mut Criterion) {
        let f = DQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("DQuotientHashTable::lookup", move |b| {
            b.iter(|| f.lookup(e))
        });
    }

    pub fn bench_lookup_dqqht(c: &mut Criterion) {
        let f = DQQuotientHashTable::new(MEMORY_SIZE, NUM_BUCKETS, FINGERPRINT_SIZE);
        let e = Element { value: 1234 };
        c.bench_function("DQQuotientHashTable::lookup", move |b| {
            b.iter(|| f.lookup(e))
        });
    }

}

/// General tests
criterion_group!(general, benchmarks::bench_rng,);

/// Tests for QHT
criterion_group!(
    bench_qht,
    benchmarks::bench_new_qht,
    benchmarks::bench_insert_qht,
    benchmarks::bench_lookup_qht,
);

/// Tests for DQHT
criterion_group!(
    bench_dqht,
    benchmarks::bench_new_dqht,
    benchmarks::bench_insert_dqht,
    benchmarks::bench_lookup_dqht,
);

/// Tests for DQQHT
criterion_group!(
    bench_dqqht,
    benchmarks::bench_new_dqqht,
    benchmarks::bench_insert_dqqht,
    benchmarks::bench_lookup_dqqht,
);

/// Run tests
criterion_main!(general, bench_qht, bench_dqht, bench_dqqht);
