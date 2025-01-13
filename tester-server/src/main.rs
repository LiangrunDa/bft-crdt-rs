use std::collections::HashMap;
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
                        BFTORSetOp::Remove(r.elem, r.ids)
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
                        let elem_id = e.elem_id
                            .map(|id| (id.first, id.second));
                        BFTRGAOp::Insert(e.value, e.id, elem_id)
                    }
                    RGAOperation::Delete(d) => {
                        let elem_id = d.elem_id.map(|id| (id.first, id.second));
                        BFTRGAOp::Delete(elem_id.unwrap())
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
