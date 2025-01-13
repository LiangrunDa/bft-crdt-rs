use criterion::BenchmarkGroup;
use criterion::measurement::WallTime;
use crdts::crdts::crdt::CRDT;
use crdts::crdts::orset::{ORSet, ORSetOp};

pub fn bench_orset_add(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("ORSet-Add", |b| {
        let mut orset = ORSet::new();
        
        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let add_op = orset.add(
                    format!("test_element_{}", iteration),
                    format!("id_{}", iteration)
                );
                orset.interpret_op(&add_op);
            }
        );
    });
}

pub fn bench_orset_remove(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("ORSet-Remove", |b| {
        let mut orset = ORSet::new();
        
        // Setup: Add element first
        let add_op = orset.add("test_element", "id_1");
        orset.interpret_op(&add_op);
        
        b.iter(|| {
            let ids = orset.get_ids("test_element")
                         .into_iter()
                         .collect();
            let remove_op = orset.remove("test_element", ids);
            orset.interpret_op(&remove_op);
            
            // Re-add the element for next iteration
            let add_op = orset.add("test_element", "id_1");
            orset.interpret_op(&add_op);
        });
    });
}
