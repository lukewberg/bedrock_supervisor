pub mod rcon {
    tonic::include_proto!("rcon.v1");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("rcon");

use crate::management::rcon::{
    BackupMetadata, ListBackupsRequest, ListBackupsResponse, RestoreBackupRequest,
    RestoreBackupResponse, ScheduleType, ServerStdioRequest, ServerStdioResponse,
};
use std::pin::Pin;
use tokio::spawn;
use tokio_stream::{Stream, StreamExt};

use crate::backup_manager::BackupManager;
use crate::config::BackupFrequency;
use crate::wrapper::Wrapper;
use chrono::{DateTime, Utc};
use rcon::rcon_service_server::RconService;
use rcon::{GetStatusRequest, GetStatusResponse};
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tonic::{Request, Response, Status, Streaming};

// #[derive(Debug)]
pub struct Rcon {
    backup_manager: BackupManager,
    wrapper: Wrapper,
}

impl Rcon {
    pub fn new(server_manager: BackupManager, wrapper: Wrapper) -> Self {
        Self {
            backup_manager: server_manager,
            wrapper,
        }
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
        let req = request.into_inner();
        let frequency = match ScheduleType::try_from(req.schedule_type) {
            Ok(ScheduleType::Minute) => BackupFrequency::Minute,
            Ok(ScheduleType::Hourly) => BackupFrequency::Hourly,
            Ok(ScheduleType::Daily) => BackupFrequency::Daily,
            Ok(ScheduleType::Weekly) => BackupFrequency::Weekly,
            Ok(ScheduleType::Unspecified) | Err(_) => {
                return Err(Status::invalid_argument("schedule_type must be specified"));
            }
        };

        let archives = self
            .backup_manager
            .list_backups(&frequency)
            .map_err(|e| Status::internal(e.to_string()))?;

        let backups = archives
            .into_iter()
            .map(|a| BackupMetadata {
                id: a.id,
                timestamp: DateTime::<Utc>::from(a.modified).to_rfc3339(),
                size: a.size as i64,
            })
            .collect();

        Ok(Response::new(ListBackupsResponse { backups }))
    }

    type RestoreBackupStream = ReceiverStream<Result<RestoreBackupResponse, Status>>;

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
        let (history, rx) = self.wrapper.stdout_subscribe_with_history();
        let input = self.wrapper.get_stdin();
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
        let stream = async_stream::stream! {
            for line in history {
                yield Ok(line);
            }
            let mut live = BroadcastStream::new(rx).filter_map(|item| item.ok());
            while let Some(item) = live.next().await {
                yield item;
            }
        };
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
