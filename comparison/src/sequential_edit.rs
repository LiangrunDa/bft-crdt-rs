use std::io::Write;
use std::time::Instant;
use crdts::bft_crdts::bft_crdt::BFTCRDTHandler;
use crdts::bft_crdts::bft_rga::BFTRGA;
use crdts::crdts::crdt::CRDT;
use crdts::crdts::rga::RGA;
use crate::common::*;

pub fn sequential_editing(dataset_name: &str) {
    let data = load_testing_data(format!("comparison/editing_trace/{}.json.gz", dataset_name).as_str());
    let mut out = get_output_file(format!("sequential_editing-{dataset_name}").as_str());
    
    let mut rga = RGA::new();
    
    // start timer
    let start_time = Instant::now();
    let mut i = 0;
    let len = data.patches().count();
    let mut count = 0;
    let mut total_op_count = 0;
    for patch in data.patches() {
        println!("rga: {}/{}", count, len);
        count = count + 1;
        // for each character in the patch, we delete a node with the id i
        for _ in 0..patch.1 {
            let delete_op = rga.delete(patch.0);
            total_op_count += 1; 
            rga.interpret_op(&delete_op.unwrap());
        }
        // it would be enough to guarantee the uniqueness of id using increment integer
        // for each character in the patch, we insert a new node with the id i
        let chars_count = patch.2.chars().clone().count();
        for j in 0..chars_count {
            let s = patch.2.chars().nth(j);
            let insert_op = rga.insert(patch.0 + j, s.unwrap(), i);
            total_op_count += 1; 
            rga.interpret_op(&insert_op.unwrap());
            i = i + 1;
        }
    }
    let elapsed = start_time.elapsed();
    writeln!(out, "RGA time for {} is {:?}", dataset_name, elapsed).unwrap();

    let char_list = rga.get_list();
    let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
    let expected = data.end_content.clone();
    assert_eq!(result.len(), data.end_content.len());
    assert_eq!(result, expected);

    let bft_rga = BFTRGA::new();
    let mut handler = BFTCRDTHandler::new(bft_rga);
    let start_time = Instant::now();
    count = 0;
    let mut i = 0;
    for patch in data.patches() {
        println!("bft-rga: {}/{}", count, len);
        count = count + 1;
        for _ in 0..patch.1 {
            let delete_op = handler.crdt.delete(patch.0);
            handler.handle_local_op(delete_op.unwrap());
        }
        
        let chars_count = patch.2.chars().clone().count();
        for j in 0..chars_count {
            let s = patch.2.chars().nth(j).unwrap();
            let insert_op = handler.crdt.insert(patch.0 + j, s, i);
            i = i + 1;
            handler.handle_local_op(insert_op.unwrap());
        }
    }
    let elapsed = start_time.elapsed();
    writeln!(out, "BFT-RGA time for {} is {:?}", dataset_name, elapsed).unwrap();

    writeln!(out, "the total number of operation is {:?}", total_op_count).unwrap();

    let char_list = handler.crdt.get_list();
    let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
    
    assert_eq!(result.len(), data.end_content.len());
    assert_eq!(result, data.end_content);
    
}