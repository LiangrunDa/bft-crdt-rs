use criterion::BenchmarkGroup;
use criterion::measurement::WallTime;
use crdts::crdts::rga::{RGA, RGAOp};
use crdts::crdts::crdt::CRDT;

pub fn bench_rga_insert(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("RGA-Insert", |b| {
        let mut rga = RGA::new();
        
        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let insert_op = rga.insert(iteration - 1, format!("test_element_{}", iteration), iteration).unwrap();
                rga.interpret_op(&insert_op);
            }
        )
    });
}

pub fn bench_rga_delete(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("RGA-Delete", |b| {
        let mut rga = RGA::new();

        // Setup: Insert 1000 elements first
        for i in 0..1000 {
            let insert_op = rga.insert(i, format!("test_element_{}", i), i).unwrap();
            rga.interpret_op(&insert_op);
        }
        
        let current_list = rga.get_list();
        assert_eq!(current_list.len(), 1000);

        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let delete_op = rga.raw_delete(iteration % 1000).unwrap();
                rga.interpret_op(&delete_op);
            }
        )
    });
}
