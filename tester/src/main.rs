use std::collections::HashMap;
use protocol::bftcrdtrpc::bftcrdt_tester_service_server::{BftcrdtTesterService, BftcrdtTesterServiceServer};
use protocol::bftcrdtrpc::{or_set_response, OrSetRequest, OrSetResponse};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;
use crdts::bft_crdts::bft_crdt::BFTCRDTTester;
use crdts::bft_crdts::bft_orset::{BFTORSet, BFTORSetOp};
use crdts::bft_crdts::hash_graph::Node;
use protocol::bftcrdtrpc::or_set_node_message::Operation;

mod logger;

// Our server implementation
#[derive(Debug, Default)]
pub struct BftCrdtTesterServer {}

#[tonic::async_trait]
impl BftcrdtTesterService for BftCrdtTesterServer {
    async fn test_or_set_once(
        &self,
        request: Request<OrSetRequest>,
    ) -> Result<Response<OrSetResponse>, Status> {
        let mut tester: BFTCRDTTester<BFTORSetOp<String>, BFTORSet<String>> = BFTCRDTTester::new(BFTORSet::new());

        for node in request.into_inner().nodes {
            // Store it as String for convenience (to align with hash function defined in Scala)
            let op :BFTORSetOp<String> = match node.operation {
                Some(inner_op) => match inner_op {
                    Operation::Add(e) => {
                        BFTORSetOp::Add(e.elem.to_string())
                    }
                    Operation::Rem(r) => {
                        BFTORSetOp::Remove(r.elem.to_string(), r.ids)
                    }
                }
                None => return Err(Status::invalid_argument("Operation not provided")),
            };
            let hash_node = Node {
                predecessors: node.predecessors,
                value: op,
            };
            
            tester.handle_node(hash_node);
        }
        let mut result_map: HashMap<i32, or_set_response::ElemIds> = Default::default();
        for (k, v) in tester.crdt.elements.iter() {
            let mut elem_ids: Vec<String> = v.iter().map(|id| id.to_string()).collect();
            // Sort the elem_ids
            elem_ids.sort();
            
            // parse String to i32
            result_map.insert(k.parse().unwrap(), or_set_response::ElemIds {
                elem_id: elem_ids
            });
        }
        
        let reply = OrSetResponse {
            result_map: result_map,
        };
        
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _file_appender_guard = logger::init(String::from("debug"), "tokio=error,crdts=trace")?;
    let addr = "[::1]:50052".parse()?;
    let tester = BftCrdtTesterServer::default();

    info!("Server listening on {}", addr);

    Server::builder()
        .add_service(BftcrdtTesterServiceServer::new(tester))
        .serve(addr)
        .await?;

    Ok(())
}
