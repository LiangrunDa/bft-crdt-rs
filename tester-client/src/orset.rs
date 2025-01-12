use crdts::bft_crdts::bft_crdt::{BFTCRDTGenerator};
use crdts::bft_crdts::bft_orset::{BFTORSet, BFTORSetOp};
use crdts::bft_crdts::hash_graph::{Node};
use tracing::{error, info};
use protocol::bftcrdtrpc::bftcrdt_tester_service_client::BftcrdtTesterServiceClient;
use protocol::bftcrdtrpc::or_set_node_message::{AddMessage, Operation, RemMessage};
use protocol::bftcrdtrpc::OrSetRequest;
use crate::cli::Args;
use rand;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

const MAX_NATURAL_NUMBER: i32 = 200;

pub struct ORSetGenerator {
    generator: BFTCRDTGenerator<BFTORSetOp<i32>, BFTORSet<i32>>,
    rng: StdRng,
}

impl ORSetGenerator {
    pub fn new(seed: u64) -> Self {
        let orset = BFTORSet::new();
        let generator = BFTCRDTGenerator::new(orset, seed);
        ORSetGenerator {
            generator,
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    pub fn generate_valid_add(&mut self) -> Node<BFTORSetOp<i32>> {
        let element = self.rng.gen_range(0..MAX_NATURAL_NUMBER);
        let add_op =  self.generator.crdt.add(element);
        self.generator.generate_and_interpret_valid_node(add_op)
    }
    
    pub fn generate_valid_rem(&mut self) -> Node<BFTORSetOp<i32>> {
        let all_values = self.generator.crdt.get_set();
        // Check if the set is empty
        if all_values.is_empty() {
            // If empty, generate an add operation instead
            return self.generate_valid_add();
        }
        // randomly choose an element to remove
        let random_idx = self.rng.gen_range(0..all_values.len());
        let element = all_values.iter().nth(random_idx).unwrap().clone();
        let rem_op = self.generator.crdt.remove_elem(element);
        self.generator.generate_and_interpret_valid_node(rem_op)
    }
    
    pub fn generate_random_struct_valid_add_node(&mut self) -> Node<BFTORSetOp<i32>> {
        // generate random number from 0 to MAX_NATURAL_NUMBER
        let element = self.rng.gen_range(0..MAX_NATURAL_NUMBER);
        let add_op = self.generator.crdt.add(element);
        self.generator.generate_and_interpret_random_struct_valid_node(add_op)
    }
    
    pub fn generate_random_struct_valid_rem_node(&mut self) -> Node<BFTORSetOp<i32>> {
        let all_values = self.generator.crdt.get_set();
        // Check if the set is empty
        if all_values.is_empty() {
            // If empty, generate an add operation instead
            return self.generate_random_struct_valid_add_node();
        }
        // randomly choose an element to remove
        let random_idx = self.rng.gen_range(0..all_values.len());
        let element = all_values.iter().nth(random_idx).unwrap().clone();
        let rem_op = self.generator.crdt.remove_elem(element);
        self.generator.generate_and_interpret_random_struct_valid_node(rem_op)
    }
    
    pub fn generate_random_add(&mut self) -> Node<BFTORSetOp<i32>> {
        let element = self.rng.gen_range(0..MAX_NATURAL_NUMBER);
        let add_op = self.generator.crdt.add(element);
        self.generator.generate_and_interpret_random_node(add_op)
    }
    
    pub fn generate_random_rem(&mut self) -> Node<BFTORSetOp<i32>> {
        let element = self.rng.gen_range(0..MAX_NATURAL_NUMBER);
        let rem_op = self.generator.crdt.remove_elem(element);
        self.generator.generate_and_interpret_random_node(rem_op)
    }
}


pub struct ORSetExperiment {
    generator: ORSetGenerator,
    args: Args,
}

impl ORSetExperiment {
    pub fn new(args: Args) -> Self {
        let generator = ORSetGenerator::new(args.seed);
        ORSetExperiment {
            generator,
            args,
        }
    }
    
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let client1_addr = format!("http://{}", self.args.server1);
        let client2_addr = format!("http://{}", self.args.server2);

        let mut client1 = BftcrdtTesterServiceClient::connect(client1_addr).await?;
        let mut client2 = BftcrdtTesterServiceClient::connect(client2_addr).await?;
        let inputs = self.generate_input(self.args.num);
        let request: OrSetRequest = OrSetRequest {
            nodes: inputs.iter().map(|node| self.convert_orset_node_to_orset_node_message(node.clone())).collect(),
        };
        
        let response1 = client1.test_or_set_once(request.clone()).await?;
        
        // Collect all key-value pairs for server1 into a single string
        let mut keys1: Vec<_> = response1.get_ref().result_map.keys().collect();
        keys1.sort();
        let server1_results: Vec<String> = keys1.iter()
            .map(|k| format!("{}: {:?}", k, response1.get_ref().result_map.get(k).unwrap()))
            .collect();
        info!("Server1 results: {}", server1_results.join(", "));

        let response2 = client2.test_or_set_once(request.clone()).await?;
        
        // Collect all key-value pairs for server2 into a single string
        let mut keys2: Vec<_> = response2.get_ref().result_map.keys().collect();
        keys2.sort();
        let server2_results: Vec<String> = keys2.iter()
            .map(|k| format!("{}: {:?}", k, response2.get_ref().result_map.get(k).unwrap()))
            .collect();
        info!("Server2 results: {}", server2_results.join(", "));
        
        if response1.get_ref() != response2.get_ref() {
            error!("Responses from server1 and server2 are different");
            return Err(anyhow::anyhow!("Responses from server1 and server2 are different"));
        }
        
        Ok(())
    }
    
    fn convert_orset_node_to_orset_node_message(&self, node: Node<BFTORSetOp<i32>>) -> protocol::bftcrdtrpc::OrSetNodeMessage {
        let predecessors = node.predecessors.clone();
        let operation = match node.value {
            BFTORSetOp::Add(e) => {
                Operation::Add(AddMessage {
                    elem: e,
                })
            }
            
            BFTORSetOp::Remove(e, ids) => {
                Operation::Rem(RemMessage {
                    elem: e,
                    ids,
                })
                
            }
        };
        protocol::bftcrdtrpc::OrSetNodeMessage {
            predecessors,
            operation: Some(operation),
        }
    }
    
    fn generate_input(&mut self, exp_num: u8) -> Vec<Node<BFTORSetOp<i32>>> {
        
        match exp_num {
            0 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.generator.rng.gen::<f32>() < 0.2 {
                        // 20% chance to generate remove
                        let rem = self.generator.generate_valid_rem();
                        inputs.push(rem);
                    } else {
                        // 80% chance to generate add
                        let add = self.generator.generate_valid_add();
                        inputs.push(add);
                    }
                }
                inputs
            }
            1 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.generator.rng.gen::<f32>() < 0.5 {
                        // 50% chance to generate random structurally valid node
                        if self.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate remove
                            let rem = self.generator.generate_random_struct_valid_rem_node();
                            inputs.push(rem);
                        } else {
                            // 80% chance to generate add
                            let add = self.generator.generate_random_struct_valid_add_node();
                            inputs.push(add);
                        }
                    } else {
                        // 50% chance to generate valid node
                        if self.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate remove
                            let rem = self.generator.generate_valid_rem();
                            inputs.push(rem);
                        } else {
                            // 80% chance to generate add
                            let add = self.generator.generate_valid_add();
                            inputs.push(add);
                        }
                    }
                }
                inputs
            }
            2 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    let rand = self.generator.rng.gen::<f32>();
                    if rand < 0.2 {
                        // 20% chance to generate random node
                        if self.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate remove
                            let rem = self.generator.generate_random_rem();
                            inputs.push(rem);
                        } else {
                            // 80% chance to generate add
                            let add = self.generator.generate_random_add();
                            inputs.push(add);
                        }
                    } else if rand < 0.6 {
                        // 40% chance to generate random structurally valid node
                        if self.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate remove
                            let rem = self.generator.generate_random_struct_valid_rem_node();
                            inputs.push(rem);
                        } else {
                            // 80% chance to generate add
                            let add = self.generator.generate_random_struct_valid_add_node();
                            inputs.push(add);
                        }
                    } else {
                        // 40% chance to generate valid node
                        if self.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate remove
                            let rem = self.generator.generate_valid_rem();
                            inputs.push(rem);
                        } else {
                            // 80% chance to generate add
                            let add = self.generator.generate_valid_add();
                            inputs.push(add);
                        }
                    }
                }
                inputs
            }
            3 => {
                println!("3");
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.generator.rng.gen::<f32>() < 0.2 {
                        // 20% chance to generate remove
                        let rem = self.generator.generate_random_rem();
                        inputs.push(rem);
                    } else {
                        // 80% chance to generate add
                        let add = self.generator.generate_random_add();
                        inputs.push(add);
                    }
                }
                inputs
            }
            _ => panic!("Experiment {} not implemented", exp_num),
        }
    }
}


