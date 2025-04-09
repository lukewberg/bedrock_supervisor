pub mod rcon {
    tonic::include_proto!("_");
}

use tonic::{Request, Response, Status};
use rcon::rcon_server::{RconServer, Rcon};
use rcon::{DaemonicStatus, StatusRequest};

#[derive(Debug)]
pub struct RconService {}

#[tonic::async_trait]
impl Rcon for RconService {
    async fn get_status(&self, request: Request<StatusRequest>) -> Result<Response<DaemonicStatus>, Status> {
        Ok(Response::new(DaemonicStatus {
            status: "ok".to_string(),
        }))
    }
}