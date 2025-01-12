use crdts::bft_crdts::bft_crdt::{BFTCRDTGenerator};
use crdts::bft_crdts::bft_orset::{BFTORSet, BFTORSetOp};
use crdts::bft_crdts::hash_graph::{Node};
use rand;
use tracing::{error, info};
use protocol::bftcrdtrpc::bftcrdt_tester_service_client::BftcrdtTesterServiceClient;
use protocol::bftcrdtrpc::or_set_node_message::{AddMessage, Operation, RemMessage};
use protocol::bftcrdtrpc::OrSetRequest;
use crate::cli::Args;

pub struct ORSetGenerator {
    generator: BFTCRDTGenerator<BFTORSetOp<i32>, BFTORSet<i32>>,
}

impl ORSetGenerator {
    pub fn new() -> Self {
        let orset = BFTORSet::new();
        let generator = BFTCRDTGenerator::new(orset);
        ORSetGenerator {
            generator,
        }
    }
    
    pub fn generate_valid_add(&mut self, element: i32) -> Node<BFTORSetOp<i32>> {
        let add_op =  self.generator.crdt.add(element);
        self.generator.generate_and_interpret_valid_node(add_op)
    }
    
    pub fn generate_valid_rem(&mut self, element: i32) -> Node<BFTORSetOp<i32>> {
        let rem_op = self.generator.crdt.remove_elem(element);
        self.generator.generate_and_interpret_valid_node(rem_op)
    }
    
    pub fn generate_random_struct_valid_add_node(&mut self) -> Node<BFTORSetOp<i32>> {
        let element = rand::random::<i32>();
        let add_op = self.generator.crdt.add(element);
        self.generator.generate_and_interpret_random_struct_valid_node(add_op)
    }
    
    pub fn generate_random_struct_valid_rem_node(&mut self) -> Node<BFTORSetOp<i32>> {
        let element = rand::random::<i32>();
        let rem_op = self.generator.crdt.remove_elem(element);
        self.generator.generate_and_interpret_random_struct_valid_node(rem_op)
    }
    
    pub fn generate_random_add(&mut self) -> Node<BFTORSetOp<i32>> {
        // generate random i32 
        let element = rand::random::<i32>();
        let add_op = self.generator.crdt.add(element);
        self.generator.generate_and_interpret_random_node(add_op)
    }
    
    pub fn generate_random_rem(&mut self) -> Node<BFTORSetOp<i32>> {
        // generate random i32 
        let element = rand::random::<i32>();
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
        let generator = ORSetGenerator::new();
        ORSetExperiment {
            generator,
            args,
        }
    }
    
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let inputs = self.generate_input(self.args.num);
        let request: OrSetRequest = OrSetRequest {
            nodes: inputs.iter().map(|node| self.convert_orset_node_to_orset_node_message(node.clone())).collect(),
        };

        let client1_addr = format!("http://{}", self.args.server1);
        let client2_addr = format!("http://{}", self.args.server2);

        let mut client1 = BftcrdtTesterServiceClient::connect(client1_addr).await?;
        let mut client2 = BftcrdtTesterServiceClient::connect(client2_addr).await?;
        
        let response1 = client1.test_or_set_once(request.clone()).await?;
        info!("Response from server1: {:?}", response1.get_ref());
        let response2 = client2.test_or_set_once(request.clone()).await?;
        info!("Response from server2: {:?}", response2.get_ref());
        
        if (response1.get_ref() == response2.get_ref()) {
            info!("Responses from server1 and server2 are the same");
            Ok(())
        } else {
            error!("Responses from server1 and server2 are different");
            Err(anyhow::anyhow!("Responses from server1 and server2 are different"))
        }
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
            0 => { // basic add and remove
                let mut inputs = vec![];
                let add1 = self.generator.generate_valid_add(1);
                let add2 = self.generator.generate_valid_add(2);
                let add3 = self.generator.generate_valid_add(3);
                let rem1 = self.generator.generate_valid_rem(1);
                let rem2 = self.generator.generate_valid_rem(2);
                inputs.push(add1);
                inputs.push(add2);
                inputs.push(add3);
                inputs.push(rem1);
                inputs.push(rem2);
                inputs
            }
            _ => panic!("Experiment {} not implemented", exp_num),
        }
    }
}


