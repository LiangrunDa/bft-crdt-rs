use crate::common::*;
use std::sync::{mpsc::{self, Sender, Receiver}};
use std::thread;
use std::collections::{HashSet};
use std::thread::JoinHandle;
use std::time::{Instant};
use std::io::Write;
use crdts::crdts::crdt::CRDT;
use crdts::crdts::rga::{RGAOp, RGA};
use crdts::bft_crdts::bft_crdt::BFTCRDTHandler;
use crdts::bft_crdts::bft_rga::{BFTRGAOp, BFTRGA};
use crdts::bft_crdts::hash_graph::Node;

#[derive(Clone)]
pub struct Message {
    pub deps: HashSet<usize>,
    pub ops: Vec<RGAOp<i32, char>>,
    pub txnid: i32,
}

#[derive(Clone)]
pub struct BFTMessage {
    pub deps: HashSet<usize>,
    pub ops: Vec<Node<BFTRGAOp<i32, char>>>,
    pub txnid: i32,
}

pub struct Peer {
    pub id: usize,
    pub receiver: Receiver<Message>,
    pub sender: Sender<Message>,
    pub data: TestData,
}

pub struct BFTPeer {
    pub id: usize,
    pub receiver: Receiver<BFTMessage>,
    pub sender: Sender<BFTMessage>,
    pub data: TestData,
}

impl Peer {
    pub fn new(
        id: usize,
        receiver: Receiver<Message>,
        sender: Sender<Message>,
        data: TestData,
    ) -> Self {
        Peer {
            id,
            receiver,
            sender,
            data,
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        let id = self.id;
        let receiver = self.receiver;
        let sender = self.sender;
        let end_content = self.data.end_content.clone();
        let mut txns = self.data.txns.clone();
        let num_of_txns = txns.len();

        return thread::spawn(move || {
            let mut graph = HashSet::new();
            let mut heads = HashSet::new();
            let mut rga = RGA::new();
            let mut txnid = -1;
            let mut buffer = Vec::new();
            let mut wait_for_parents = HashSet::new();
            let mut done = false;
            let mut num_of_txns_applied = 0;
            loop {
                // we first try to apply all the txns locally until we can't apply any more (missing nodes from remote peers)
                loop {
                    if txns.is_empty() {
                        break;
                    }
                    let txn = txns.remove(0);
                    txnid += 1;
                    if txn.agent.unwrap() == id {
                        println!("applying txn {} [{}]", txnid, id);
                        // convert txn.parents to a HashSet and check if it's equal to heads
                        let parents = txn.clone().parents.unwrap().iter().cloned().collect::<HashSet<_>>();
                        if parents == heads {
                            num_of_txns_applied += 1;
                            // apply the txn
                            let mut operations = Vec::new();
                            for patch in txn.patches {
                                for _ in 0..patch.1 {
                                    let delete_op = rga.delete(patch.0).unwrap();
                                    rga.interpret_op(&delete_op);
                                    operations.push(delete_op)
                                }
                                // it would be enough to guarantee the uniqueness of id using increment integer
                                // for each character in the patch, we insert a new node with the id i
                                let chars_count = patch.2.chars().count();
                                for j in 0..chars_count {
                                    if let Some(s) = patch.2.chars().nth(j) {
                                        let insert_op = rga.insert(patch.0 + j, s, txnid).unwrap();
                                        rga.interpret_op(&insert_op);
                                        operations.push(insert_op);
                                    }
                                }
                            }
                            let msg = Message {
                                ops: operations,
                                deps: parents.clone(),
                                txnid,
                            };
                            sender.send(msg);
                            
                            // update heads and graph
                            graph.insert(txnid);
                            for parent in parents {
                                heads.remove(&parent);
                            }
                            heads.insert(txnid as usize);
                            
                            // check if we have applied the final txn
                            if num_of_txns_applied == num_of_txns as i32 {
                                let char_list = rga.get_list();
                                let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
                                assert_eq!(result.len(), end_content.len());
                                assert_eq!(result, end_content);
                                done = true;
                                break;
                            }
                        } else {
                            // we can't apply it anymore
                            wait_for_parents = parents;
                            // undo
                            txns.insert(0, txn);
                            txnid -= 1;
                            println!("wait for {:?}", wait_for_parents);
                            break;
                        }
                    }
                }
                if done {
                    break;
                }

                loop {
                    // we now try to receive all the operations from the other peer
                    while let Ok(msg) = receiver.try_recv() {
                        println!("received msg {} [{}]", msg.txnid, id);
                        buffer.push(msg.clone());
                    }
                    for i in 0..buffer.len() {
                        if buffer.len() > 0 {
                            let m = buffer.remove(0);
                            // we must have the deps in the graph
                            let mut has_deps = true;
                            for dep in m.deps.clone() {
                                if !graph.contains(&(dep as i32)) {
                                    has_deps = false;
                                    break
                                }
                            }
                            if !has_deps {
                                buffer.push(m.clone());
                                continue;
                            }
                            println!("applying msg {} [{}]", m.txnid, id);
                            num_of_txns_applied += 1;
                            let ops = m.ops.clone();
                            for op in ops {
                                rga.interpret_op(&op);
                            }
                            graph.insert(m.txnid);
                            heads.insert(m.txnid as usize);
                            for parent in m.deps.clone() {
                                heads.remove(&parent);
                            }
                            
                            if num_of_txns_applied == num_of_txns as i32 {
                                let char_list = rga.get_list();
                                let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
                                assert_eq!(result.len(), end_content.len());
                                assert_eq!(result, end_content);
                                done = true;
                                break;
                            }
                        } else {
                            panic!("error");
                        }
                        if heads == wait_for_parents {
                            break;
                        }
                    }
                    if done {
                        break;
                    }
                    if heads == wait_for_parents {
                        println!("heads == wait_for_parents");
                        break;
                    }
                }
                if done {
                    break;
                }
            }
        });
    }
}

impl BFTPeer {
    pub fn new(
        id: usize,
        receiver: Receiver<BFTMessage>,
        sender: Sender<BFTMessage>,
        data: TestData,
    ) -> Self {
        BFTPeer {
            id,
            receiver,
            sender,
            data,
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        let id = self.id;
        let receiver = self.receiver;
        let sender = self.sender;
        let end_content = self.data.end_content.clone();
        let mut txns = self.data.txns.clone();
        let num_of_txns = txns.len();

        return thread::spawn(move || {
            let mut graph = HashSet::new();
            let mut heads = HashSet::new();
            let bft_rga = BFTRGA::new();
            let mut handler = BFTCRDTHandler::new(bft_rga);
            let mut txnid = -1;
            let mut buffer = Vec::new();
            let mut wait_for_parents = HashSet::new();
            let mut number_of_txns_applied = 0;
            let mut done = false;
            loop {
                // we first try to apply all the txns locally until we can't apply any more (missing nodes from remote peers)
                loop {
                    if txns.is_empty() {
                        break;
                    }
                    let txn = txns.remove(0);
                    txnid += 1;
                    if txn.agent.unwrap() == id {
                        println!("applying txn {} [{}]", txnid, id);
                        // convert txn.parents to a HashSet and check if it's equal to heads
                        let parents = txn.clone().parents.unwrap().iter().cloned().collect::<HashSet<_>>();
                        if parents == heads {
                            number_of_txns_applied += 1;
                            // apply the txn
                            let mut operations = Vec::new();
                            for patch in txn.patches {
                                for _ in 0..patch.1 {
                                    let delete_op = handler.crdt.delete(patch.0).unwrap();
                                    let node = handler.handle_local_op(delete_op.clone());
                                    operations.push(node);
                                }
                                // it would be enough to guarantee the uniqueness of id using increment integer
                                // for each character in the patch, we insert a new node with the id i
                                let chars_count = patch.2.chars().count();
                                for j in 0..chars_count {
                                    if let Some(s) = patch.2.chars().nth(j) {
                                        let insert_op = handler.crdt.insert(patch.0 + j, s, txnid).unwrap();
                                        let node = handler.handle_local_op(insert_op.clone());
                                        operations.push(node);
                                    }
                                }
                            }
                            let msg = BFTMessage {
                                ops: operations,
                                deps: parents.clone(),
                                txnid,
                            };
                            sender.send(msg).unwrap();
                            
                            // update heads and graph
                            graph.insert(txnid);
                            for parent in parents {
                                heads.remove(&parent);
                            }
                            heads.insert(txnid as usize);
                            
                            // check if we have applied the final txn
                            if number_of_txns_applied == num_of_txns as i32 {
                                let char_list = handler.crdt.get_list();
                                let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
                                assert_eq!(result.len(), end_content.len());
                                assert_eq!(result, end_content);
                                done = true;
                                break;
                            }
                        } else {
                            // we can't apply it anymore
                            wait_for_parents = parents;
                            // undo
                            txns.insert(0, txn);
                            txnid -= 1;
                            println!("wait for {:?} [{}]", wait_for_parents, id);
                            break;
                        }
                    }
                }
                if done {
                    break;
                }
                

                loop {
                    // we now try to receive all the operations from the other peer
                    while let Ok(msg) = receiver.try_recv() {
                        println!("received msg {} [{}]", msg.txnid, id);
                        buffer.push(msg.clone());
                    }
                    for i in 0..buffer.len() {
                        if buffer.len() > 0 {
                            let m = buffer.remove(0);
                            // we must have the deps in the graph
                            let mut has_deps = true;
                            for dep in m.deps.clone() {
                                if !graph.contains(&(dep as i32)) {
                                    has_deps = false;
                                    break
                                }
                            }
                            if !has_deps {
                                buffer.push(m.clone());
                                continue;
                            }
                            println!("applying msg {} [{}]", m.txnid, id);
                            number_of_txns_applied += 1;

                            let ops = m.ops.clone();
                            for op in ops {
                                handler.handle_remote_node(op);
                            }
                            graph.insert(m.txnid);
                            heads.insert(m.txnid as usize);
                            for parent in m.deps.clone() {
                                heads.remove(&parent);
                            }
                            
                            if number_of_txns_applied == num_of_txns as i32 {
                                let char_list = handler.crdt.get_list();
                                let result = char_list.iter().map(|c| c.to_string()).collect::<Vec<String>>().join("");
                                println!("{:?}", handler.pending_nodes);
                                assert_eq!(result.len(), end_content.len());
                                assert_eq!(result, end_content);
                                done = true;
                                break;
                            }
                        } else {
                            panic!("error");
                        }
                        if heads == wait_for_parents {
                            break;
                        }
                    }
                    if done {
                        break;
                    }
                    if heads == wait_for_parents {
                        println!("heads == wait_for_parents");
                        break;
                    }
                }
                if done {
                    break;
                }
            }
        });
    }
}

// Helper function to create 2 peers and run them
pub fn run_peers(dataset_name: &str) {
    let mut out = get_output_file(format!("concurrent_editing-{dataset_name}").as_str());
    
    // Create channels for the two peers
    let (tx1, rx2) = mpsc::channel::<Message>(); // Peer 1 sends to Peer 2
    let (tx2, rx1) = mpsc::channel::<Message>(); // Peer 2 sends to Peer 1
    
    let data = load_testing_data(format!("comparison/editing_trace/{}.json.gz", dataset_name).as_str());
    
    // Create and run Peer 1
    let peer1 = Peer::new(0, rx1, tx1, data.clone());
    
    // Create and run Peer 2
    let peer2 = Peer::new(1, rx2, tx2, data.clone());

    // time the execution
    let start_time = Instant::now();
    let handle1 = peer1.run();
    let handle2 = peer2.run();
    
    // Wait for both peers to finish
    handle1.join();
    handle2.join();
    let elapsed = start_time.elapsed();
    writeln!(out, "RGA concurrent time for {} is {:?}", dataset_name, elapsed).unwrap();
    println!("RGA concurrent execution time: {:?}", elapsed);
    
    // Now run with BFT_RGA
    // Create channels for the two BFT peers
    let (bft_tx1, bft_rx2) = mpsc::channel::<BFTMessage>(); // BFT Peer 1 sends to BFT Peer 2
    let (bft_tx2, bft_rx1) = mpsc::channel::<BFTMessage>(); // BFT Peer 2 sends to BFT Peer 1
    
    // Create and run BFT Peer 1
    let bft_peer1 = BFTPeer::new(0, bft_rx1, bft_tx1, data.clone());
    
    // Create and run BFT Peer 2
    let bft_peer2 = BFTPeer::new(1, bft_rx2, bft_tx2, data.clone());

    // time the execution
    let bft_start_time = Instant::now();
    let bft_handle1 = bft_peer1.run();
    let bft_handle2 = bft_peer2.run();

    // Wait for both peers to finish
    bft_handle1.join();
    bft_handle2.join();
    let bft_elapsed = bft_start_time.elapsed();
    writeln!(out, "BFT-RGA concurrent time for {} is {:?}", dataset_name, bft_elapsed).unwrap();
    println!("BFT-RGA concurrent execution time: {:?}", bft_elapsed);
}