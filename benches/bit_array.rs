use mad_bench::bit_array::BitArray;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const ELEMS: usize = 1434;
pub fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("BitArray");
    for (r_size, v) in (1..8).map(|w| {
        (w, {
            let num_max = 1 << w;
            let mut v = BitArray::new(w, ELEMS);
            for n in 0..ELEMS {
                v.set(n as usize, (n as u8) % num_max);
            }
            v
        })
    }) {
        group.throughput(Throughput::Bytes(r_size as u64));
        group.bench_with_input(
            BenchmarkId::new("iterator", format!("register_size_{}", r_size)),
            &v,
            |b, v| {
                b.iter(|| {
                    v.iter().for_each(|i| {
                        black_box(i);
                    })
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("iterator2", format!("register_size_{}", r_size)),
            &v,
            |b, v| {
                b.iter(|| {
                    v.iter2().for_each(|i| {
                        black_box(i);
                    })
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("iterator3", format!("register_size_{}", r_size)),
            &v,
            |b, v| {
                b.iter(|| {
                    v.iter3().for_each(|i| {
                        black_box(i);
                    })
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("loop", format!("register_size_{}", r_size)),
            &v,
            |b, v| {
                b.iter(|| {
                    for i in 0..black_box(ELEMS) {
                        black_box(v.get(i));
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = iteration
}
criterion_main!(benches);
