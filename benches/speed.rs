use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

fn tiny_keccak(input: &[u8]) {
    let mut hasher = Keccak::v256();
    hasher.update(input);
    let mut output: [u8; 32] = [0u8; 32];
    hasher.finalize(&mut output);
}

fn mine(input: &[u8]) {
    keccak_rs::keccak_256(input);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Keccak");
    for i in [16u64, 32u64, 128u64, 512u64, 2048u64, 4096u64, 32000u64].iter() {
        let rand_input: Vec<u8> = (0..*i).map(|_| rand::random::<u8>()).collect();
        group.bench_with_input(BenchmarkId::new("keccak_rs", i), i, |b, i| {
            b.iter(|| mine(rand_input.as_slice()))
        });
        group.bench_with_input(BenchmarkId::new("tiny_keccak", i), i, |b, i| {
            b.iter(|| tiny_keccak(rand_input.as_slice()))
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
