use criterion::BenchmarkGroup;
use criterion::measurement::WallTime;
use crdts::bft_crdts::bft_rga::BFTRGA;
use crdts::bft_crdts::bft_crdt::BFTCRDTHandler;

pub fn bench_bft_rga_insert(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("BFT-RGA-Insert", |b| {
        let rga = BFTRGA::new();
        let mut handler = BFTCRDTHandler::new(rga);
        
        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let insert_op = handler.crdt.insert(iteration - 1, format!("test_element_{}", iteration), "test").unwrap();
                handler.handle_local_op(insert_op);
            }
        )
    });
}

pub fn bench_bft_rga_delete(group: &mut BenchmarkGroup<WallTime>) {
    group.bench_function("BFT-RGA-Delete", |b| {
        let rga = BFTRGA::new();
        let mut handler = BFTCRDTHandler::new(rga);

        // Setup: Insert 1000 elements first
        for i in 0..1000 {
            let insert_op = handler.crdt.insert(i, format!("test_element_{}", i), "test").unwrap();
            handler.handle_local_op(insert_op);
        }
        
        let current_list = handler.crdt.get_list();
        assert_eq!(current_list.len(), 1000);


        let mut i = 0;
        b.iter_with_setup(
            || { i += 1; i },
            |iteration| {
                let delete_op = handler.crdt.raw_delete(iteration % 1000).unwrap();
                handler.handle_local_op(delete_op);
            }
        )
    });
} 