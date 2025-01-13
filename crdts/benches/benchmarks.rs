use criterion::{criterion_group, criterion_main, Criterion};

mod bft_crdt_benchmarks;
mod crdt_benchmarks;

fn benchmark_orset(c: &mut Criterion) {
    let mut group = c.benchmark_group("ORSet Comparison");
    
    // BFT-CRDT ORSet benchmarks
    bft_crdt_benchmarks::orset::bench_bft_orset_add(&mut group);
    bft_crdt_benchmarks::orset::bench_bft_orset_remove(&mut group);
    
    // Traditional CRDT ORSet benchmarks
    crdt_benchmarks::orset::bench_orset_add(&mut group);
    crdt_benchmarks::orset::bench_orset_remove(&mut group);
    
    group.finish();
}

fn benchmark_rga(c: &mut Criterion) {
    let mut group = c.benchmark_group("RGA Comparison");
    
    // BFT-CRDT RGA benchmarks
    bft_crdt_benchmarks::rga::bench_bft_rga_insert(&mut group);
    bft_crdt_benchmarks::rga::bench_bft_rga_delete(&mut group);
    
    // Traditional CRDT RGA benchmarks
    crdt_benchmarks::rga::bench_rga_insert(&mut group);
    crdt_benchmarks::rga::bench_rga_delete(&mut group);
    
    group.finish();
}

criterion_group!(benches, benchmark_orset, benchmark_rga);
criterion_main!(benches);