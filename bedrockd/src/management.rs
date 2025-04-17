pub mod rcon {
    tonic::include_proto!("_");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("rcon");

use crate::management::rcon::{
    ListBackupsRequest, ListBackupsResponse, RestoreBackupProgressResponse, RestoreBackupRequest,
    ServerStdioRequest, ServerStdioResponse,
};
use rcon::rcon_service_server::RconService;
use rcon::{GetStatusRequest, GetStatusResponse};
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct Rcon {}

#[tonic::async_trait]
impl RconService for Rcon {
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        Ok(Response::new(GetStatusResponse {
            status: "Ok".to_string(),
        }))
    }

    async fn list_backups(
        &self,
        request: Request<ListBackupsRequest>,
    ) -> Result<Response<ListBackupsResponse>, Status> {
        Ok(Response::new(ListBackupsResponse { backups: vec![] }))
    }

    type RestoreBackupStream = ReceiverStream<Result<RestoreBackupProgressResponse, Status>>;

    async fn restore_backup(
        &self,
        request: Request<RestoreBackupRequest>,
    ) -> Result<Response<Self::RestoreBackupStream>, Status> {
        todo!()
    }

    type ServerSTDIOStream = ReceiverStream<Result<ServerStdioResponse, Status>>;

    async fn server_stdio(
        &self,
        request: Request<Streaming<ServerStdioRequest>>,
    ) -> Result<Response<Self::ServerSTDIOStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<ServerStdioResponse, Status>>(4);
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
