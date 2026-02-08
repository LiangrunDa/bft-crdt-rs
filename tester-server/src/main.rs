use std::collections::HashMap;
use std::cell::RefCell;
use protocol::bftcrdtrpc::bftcrdt_tester_service_server::{BftcrdtTesterService, BftcrdtTesterServiceServer};
use protocol::bftcrdtrpc::{or_set_response, OrSetRequest, OrSetResponse, RgaRequest, RgaResponse};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;
use crdts::bft_crdts::bft_crdt::BFTCRDTTester;
use crdts::bft_crdts::bft_orset::{BFTORSet, BFTORSetOp};
use crdts::bft_crdts::bft_rga::{BFTRGAOp, BFTRGA};
use crdts::bft_crdts::hash_graph::{Node};
use protocol::bftcrdtrpc::or_set_node_message::Operation as OrSetOperation;
use protocol::bftcrdtrpc::rga_node_message::Operation as RGAOperation;

mod logger;

// Helper function to convert Vec<u8> to [u8; 32]
fn vec_to_array32(vec: Vec<u8>) -> Result<[u8; 32], Status> {
    vec.try_into()
        .map_err(|v: Vec<u8>| Status::invalid_argument(format!("Expected 32 bytes, got {}", v.len())))
}

// Our server implementation
#[derive(Debug, Default)]
pub struct BftCrdtTesterServer {}

#[tonic::async_trait]
impl BftcrdtTesterService for BftCrdtTesterServer {
    
    async fn test_or_set_once(
        &self,
        request: Request<OrSetRequest>,
    ) -> Result<Response<OrSetResponse>, Status> {
        let mut tester: BFTCRDTTester<BFTORSetOp<i32>, BFTORSet<i32>> = BFTCRDTTester::new(BFTORSet::new());

        for node in request.into_inner().nodes {
            let op :BFTORSetOp<i32> = match node.operation {
                Some(inner_op) => match inner_op {
                    OrSetOperation::Add(e) => {
                        BFTORSetOp::Add(e.elem)
                    }
                    OrSetOperation::Rem(r) => {
                        let ids: Result<Vec<[u8; 32]>, Status> = r.ids.into_iter().map(vec_to_array32).collect();
                        BFTORSetOp::Remove(r.elem, ids?)
                    }
                }
                None => return Err(Status::invalid_argument("Operation not provided")),
            };
            let predecessors: Result<Vec<[u8; 32]>, Status> = node.predecessors.into_iter().map(vec_to_array32).collect();
            let hash_node = Node {
                predecessors: predecessors?,
                value: op,
                cached_hash: RefCell::new(None),
            };
            
            info!("Processing OR-SET node: {:?}", hash_node);
            tester.handle_node(hash_node);
        }
        let mut result_map: HashMap<i32, or_set_response::ElemIds> = Default::default();
        for (k, v) in tester.crdt.elements.iter() {
            let mut elem_ids: Vec<String> = v.iter().map(|id| hex::encode(id)).collect();
            // Sort the elem_ids
            elem_ids.sort();
            
            result_map.insert(*k, or_set_response::ElemIds {
                elem_id: elem_ids
            });
        }
        
        let reply = OrSetResponse {
            result_map,
        };
        
        Ok(Response::new(reply))
    }

    async fn test_rga_once(&self, request: Request<RgaRequest>) -> Result<Response<RgaResponse>, Status> {

        let mut tester: BFTCRDTTester<BFTRGAOp<String, i32>, BFTRGA<String, i32>> = BFTCRDTTester::new(BFTRGA::new());

        for node in request.into_inner().nodes {
            let op :BFTRGAOp<String, i32> = match node.operation {
                Some(inner_op) => match inner_op {
                    RGAOperation::Insert(e) => {
                        let elem_id = match e.elem_id {
                            Some(id) => Some((id.first, vec_to_array32(id.second)?)),
                            None => None,
                        };
                        BFTRGAOp::Insert(e.value, e.id, elem_id)
                    }
                    RGAOperation::Delete(d) => {
                        let elem_id = match d.elem_id {
                            Some(id) => (id.first, vec_to_array32(id.second)?),
                            None => return Err(Status::invalid_argument("elem_id not provided for Delete operation")),
                        };
                        BFTRGAOp::Delete(elem_id)
                    }
                }
                None => return Err(Status::invalid_argument("Operation not provided")),
            };
            let predecessors: Result<Vec<[u8; 32]>, Status> = node.predecessors.into_iter().map(vec_to_array32).collect();
            let hash_node = Node {
                predecessors: predecessors?,
                value: op,
                cached_hash: RefCell::new(None),
            };

            info!("Processing RGA node: {:?}", hash_node);
            tester.handle_node(hash_node);
        }
        
        let int_list = tester.crdt.get_list();
        let result: String = int_list.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");
        
        let reply = RgaResponse {
            result,
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _file_appender_guard = logger::init(String::from("debug"), "tokio=error,crdts=trace")?;
    let addr = "[::1]:50051".parse()?;
    let tester = BftCrdtTesterServer::default();

    info!("Server listening on {}", addr);

    Server::builder()
        .add_service(BftcrdtTesterServiceServer::new(tester))
        .serve(addr)
        .await?;

    Ok(())
}
