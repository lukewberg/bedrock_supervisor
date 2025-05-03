pub mod rcon {
    tonic::include_proto!("_");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("rcon");

use crate::management::rcon::{
    ListBackupsRequest, ListBackupsResponse, RestoreBackupProgressResponse, RestoreBackupRequest,
    ServerStdioRequest, ServerStdioResponse,
};
use crate::server_manager::ServerManager;
use std::pin::Pin;
use tokio::spawn;
use tokio_stream::{Stream, StreamExt};

use rcon::rcon_service_server::RconService;
use rcon::{GetStatusRequest, GetStatusResponse};
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tonic::{Request, Response, Status, Streaming};

// #[derive(Debug)]
pub struct Rcon {
    server_manager: ServerManager,
}

impl Rcon {
    pub fn new(server_manager: ServerManager) -> Self {
        Self { server_manager }
    }
}

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

    type ServerSTDIOStream =
        Pin<Box<dyn Stream<Item = Result<ServerStdioResponse, Status>> + Send>>;

    async fn server_stdio(
        &self,
        request: Request<Streaming<ServerStdioRequest>>,
    ) -> Result<Response<Self::ServerSTDIOStream>, Status> {
        let rx = self.server_manager.wrapper.stdout_subscribe();
        let input = self.server_manager.wrapper.get_stdin();
        let mut request_stream = request.into_inner();
        spawn(async move {
            while let Some(message) = request_stream.next().await {
                match message {
                    Ok(inner) => input.send(inner).await.unwrap(),
                    Err(_) => {
                        todo!()
                    }
                };
            }
        });
        let stream = BroadcastStream::new(rx).filter_map(|item| item.ok());
        Ok(Response::new(Box::pin(stream)))
    }
}

impl From<&str> for ServerStdioRequest {
    fn from(value: &str) -> Self {
        Self {
            command: value.to_string(),
        }
    }
}
