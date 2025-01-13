use crdts::bft_crdts::bft_crdt::BFTCRDTGenerator;
use crdts::bft_crdts::bft_rga::{BFTRGA, BFTRGAOp};
use crdts::bft_crdts::hash_graph::Node;
use tracing::{error, info};
use protocol::bftcrdtrpc::bftcrdt_tester_service_client::BftcrdtTesterServiceClient;
use protocol::bftcrdtrpc::rga_node_message::{InsertMessage, Operation, DeleteMessage, ElemId};
use protocol::bftcrdtrpc::RgaRequest;
use crate::cli::Args;
use rand::{Rng};

const MAX_VALUE: i32 = 100;

pub struct RGAGenerator {
    generator: BFTCRDTGenerator<BFTRGAOp<String, i32>, BFTRGA<String, i32>>,
}

impl RGAGenerator {
    pub fn new(seed: u64) -> Self {
        let rga = BFTRGA::new();
        let generator = BFTCRDTGenerator::new(rga, seed);
        RGAGenerator {
            generator,
        }
    }
    
    pub fn generate_valid_insert(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let value = self.generator.rng.gen_range(0..MAX_VALUE);
        let list_size = self.generator.crdt.get_list().len();
        let insert_idx = if list_size == 0 { 0 } else { self.generator.rng.gen_range(0..=list_size) };
        // in reality, we would use this id to control the exact position of the insert, but for testing we can just generate a random id
        let id = format!("id_{}", self.generator.rng.gen_range(0..100000));
        
        if let Some(insert_op) = self.generator.crdt.insert(insert_idx, value, id) {
            self.generator.generate_and_interpret_valid_node(insert_op)
        } else {
            panic!("Index out of range")
        }
    }
    
    pub fn generate_valid_delete(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let list_size = self.generator.crdt.get_list().len();
        if list_size == 0 {
            // If list is empty, generate an insert instead
            return self.generate_valid_insert();
        }
        
        let delete_idx = self.generator.rng.gen_range(0..list_size);
        if let Some(delete_op) = self.generator.crdt.delete(delete_idx) {
            self.generator.generate_and_interpret_valid_node(delete_op)
        } else {
            panic!("Index out of range")
        }
    }
    
    pub fn generate_random_struct_valid_insert(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let value = self.generator.rng.gen_range(0..MAX_VALUE);
        let list_size = self.generator.crdt.get_list().len();
        let insert_idx = if list_size == 0 { 0 } else { self.generator.rng.gen_range(0..=list_size) };
        let id = format!("id_{}", self.generator.rng.gen_range(0..100000));
        
        if let Some(insert_op) = self.generator.crdt.insert(insert_idx, value, id) {
            self.generator.generate_and_interpret_random_struct_valid_node(insert_op)
        } else {
            panic!("Index out of range")
        }
    }
    
    pub fn generate_random_struct_valid_delete(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let list_size = self.generator.crdt.get_list().len();
        if list_size == 0 {
            // If list is empty, generate an insert instead
            return self.generate_random_struct_valid_insert();
        }
        
        let delete_idx = self.generator.rng.gen_range(0..list_size);
        if let Some(delete_op) = self.generator.crdt.delete(delete_idx) {
            self.generator.generate_and_interpret_random_struct_valid_node(delete_op)
        } else {
            panic!("Index out of range")
        }
    }
    
    pub fn generate_random_insert(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let value = self.generator.rng.gen_range(0..MAX_VALUE);
        let list_size = self.generator.crdt.get_list().len();
        let insert_idx = if list_size == 0 { 0 } else { self.generator.rng.gen_range(0..=list_size) };
        // in reality, we would use this id to control the exact position of the insert, but for testing we can just generate a random id
        let id = format!("id_{}", self.generator.rng.gen_range(0..100000));

        if let Some(insert_op) = self.generator.crdt.insert(insert_idx, value, id) {
            self.generator.generate_and_interpret_random_node(insert_op)
        } else {
            panic!("Index out of range")
        }
    }
    
    pub fn generate_random_delete(&mut self) -> Node<BFTRGAOp<String, i32>> {
        let list_size = self.generator.crdt.get_list().len();
        if list_size == 0 {
            // If list is empty, generate an insert instead
            return self.generate_valid_insert();
        }

        let delete_idx = self.generator.rng.gen_range(0..list_size);
        if let Some(delete_op) = self.generator.crdt.delete(delete_idx) {
            self.generator.generate_and_interpret_random_node(delete_op)
        } else {
            panic!("Index out of range")
        }
    }
}

pub struct RGAExperiment {
    rga_generator: RGAGenerator,
    args: Args,
}

impl RGAExperiment {
    pub fn new(args: Args) -> Self {
        let generator = RGAGenerator::new(args.seed);
        RGAExperiment {
            rga_generator: generator,
            args,
        }
    }
    
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let client1_addr = format!("http://{}", self.args.server1);
        let client2_addr = format!("http://{}", self.args.server2);

        let mut client1 = BftcrdtTesterServiceClient::connect(client1_addr).await?;
        let mut client2 = BftcrdtTesterServiceClient::connect(client2_addr).await?;
        
        let inputs = self.generate_input(self.args.num);
        let request = RgaRequest {
            nodes: inputs.iter().map(|node| self.convert_rga_node_to_rga_node_message(node.clone())).collect(),
        };
        
        let response1 = client1.test_rga_once(request.clone()).await?;
        info!("Server1 result: {}", response1.get_ref().result);

        let response2 = client2.test_rga_once(request.clone()).await?;
        info!("Server2 result: {}", response2.get_ref().result);
        
        if response1.get_ref() != response2.get_ref() {
            error!("Responses from server1 and server2 are different");
            return Err(anyhow::anyhow!("Responses from server1 and server2 are different"));
        }
        
        Ok(())
    }
    
    fn convert_rga_node_to_rga_node_message(&self, node: Node<BFTRGAOp<String, i32>>) -> protocol::bftcrdtrpc::RgaNodeMessage {
        let predecessors = node.predecessors.clone();
        let operation = match node.value {
            BFTRGAOp::Insert(value, id, after) => {
                Operation::Insert(InsertMessage {
                    value,
                    id,
                    elem_id: after.map(|(first, second)| ElemId { first, second }),
                })
            }
            BFTRGAOp::Delete((first, second)) => {
                Operation::Delete(DeleteMessage {
                    elem_id: Some(ElemId { first, second }),
                })
            }
        };
        
        protocol::bftcrdtrpc::RgaNodeMessage {
            predecessors,
            operation: Some(operation),
        }
    }
    
    fn generate_input(&mut self, exp_num: u8) -> Vec<Node<BFTRGAOp<String, i32>>> {
        match exp_num {
            0 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                        // 20% chance to generate delete
                        let delete = self.rga_generator.generate_valid_delete();
                        inputs.push(delete);
                    } else {
                        // 80% chance to generate insert
                        let insert = self.rga_generator.generate_valid_insert();
                        inputs.push(insert);
                    }
                }
                inputs
            }
            1 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.rga_generator.generator.rng.gen::<f32>() < 0.5 {
                        // 50% chance to generate random structurally valid node
                        if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate delete
                            let delete = self.rga_generator.generate_random_struct_valid_delete();
                            inputs.push(delete);
                        } else {
                            // 80% chance to generate insert
                            let insert = self.rga_generator.generate_random_struct_valid_insert();
                            inputs.push(insert);
                        }
                    } else {
                        // 50% chance to generate valid node
                        if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate delete
                            let delete = self.rga_generator.generate_valid_delete();
                            inputs.push(delete);
                        } else {
                            // 80% chance to generate insert
                            let insert = self.rga_generator.generate_valid_insert();
                            inputs.push(insert);
                        }
                    }
                }
                inputs
            }
            2 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    let rand = self.rga_generator.generator.rng.gen::<f32>();
                    if rand < 0.2 {
                        // 20% chance to generate random node
                        if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate delete
                            let delete = self.rga_generator.generate_random_delete();
                            inputs.push(delete);
                        } else {
                            // 80% chance to generate insert
                            let insert = self.rga_generator.generate_random_insert();
                            inputs.push(insert);
                        }
                    } else if rand < 0.6 {
                        // 40% chance to generate random structurally valid node
                        if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate delete
                            let delete = self.rga_generator.generate_random_struct_valid_delete();
                            inputs.push(delete);
                        } else {
                            // 80% chance to generate insert
                            let insert = self.rga_generator.generate_random_struct_valid_insert();
                            inputs.push(insert);
                        }
                    } else {
                        // 40% chance to generate valid node
                        if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                            // 20% chance to generate delete
                            let delete = self.rga_generator.generate_valid_delete();
                            inputs.push(delete);
                        } else {
                            // 80% chance to generate insert
                            let insert = self.rga_generator.generate_valid_insert();
                            inputs.push(insert);
                        }
                    }
                }
                inputs
            }
            3 => {
                let mut inputs = vec![];
                for _ in 0..self.args.depth {
                    if self.rga_generator.generator.rng.gen::<f32>() < 0.2 {
                        // 20% chance to generate delete
                        let delete = self.rga_generator.generate_random_delete();
                        inputs.push(delete);
                    } else {
                        // 80% chance to generate insert
                        let insert = self.rga_generator.generate_random_insert();
                        inputs.push(insert);
                    }
                }
                inputs
            }
            _ => panic!("Experiment {} not implemented", exp_num),
        }
    }
}
