use criterion::BenchmarkGroup;
use criterion::measurement::WallTime;

use crdts::bft_crdts::bft_orset::BFTORSet;
use crdts::bft_crdts::bft_crdt::BFTCRDTHandler;

pub fn bench_bft_orset_add(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("BFT-ORSet-Add", |b| {
        let orset = BFTORSet::new();
        let mut handler = BFTCRDTHandler::new(orset);
        
        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let add_op = handler.crdt.add(format!("test_element_{}", iteration));
                handler.handle_local_op(add_op);
            }
        );
    });
}

pub fn bench_bft_orset_remove(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("BFT-ORSet-Remove", |b| {
        let orset = BFTORSet::new();
        let mut handler = BFTCRDTHandler::new(orset);
        
        // Setup: Add elements first
        let add_op = handler.crdt.add("test_element");
        handler.handle_local_op(add_op);
        
        b.iter(|| {
            let remove_op = handler.crdt.remove_elem("test_element");
            handler.handle_local_op(remove_op);
        });
    });
} 