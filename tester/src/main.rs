use protocol::bftcrdtrpc::bftcrdt_tester_service_server::{BftcrdtTesterService, BftcrdtTesterServiceServer};
use protocol::bftcrdtrpc::{OrSetRequest, OrSetResponse};
use tonic::{transport::Server, Request, Response, Status};

// Our server implementation
#[derive(Debug, Default)]
pub struct BftCrdtTesterServer {}

#[tonic::async_trait]
impl BftcrdtTesterService for BftCrdtTesterServer {
    async fn test_or_set_once(
        &self,
        request: Request<OrSetRequest>,
    ) -> Result<Response<OrSetResponse>, Status> {
        // Implementation will go here later
        todo!("Implement test_or_set_once")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse()?;
    let tester = BftCrdtTesterServer::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(BftcrdtTesterServiceServer::new(tester))
        .serve(addr)
        .await?;

    Ok(())
}
