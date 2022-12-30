use tokio::sync::mpsc::{self, Sender};

use crate::{client::Client, verb::VerbContextBuilder};

use super::{MudServer, hooks::Hook};

pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

pub struct ConnectionUpdate {
    pub client: Client,
    pub status: ConnectionStatus,
}

impl MudServer {
    pub async fn create_connections_channel(&self) -> Sender<ConnectionUpdate> {
        let (tx, mut rx) = mpsc::channel::<ConnectionUpdate>(32);
        let clients = self.clients.clone();
        let server = self.clone();
        tokio::spawn(async move {
            while let Some(update) = rx.recv().await {
                let clients = clients.clone();
                match update.status {
                    ConnectionStatus::Connected => {
                        let vc = VerbContextBuilder::new().client(update.client.clone()).server(server.clone());
                        let _ = server.run_hook(Hook::OnPlayerConnect, vc).await;
                        let mut clients = clients.lock().unwrap();
                        clients.push(update.client.clone());
                    }
                    ConnectionStatus::Disconnected => {
                        let vc = VerbContextBuilder::new().client(update.client.clone()).server(server.clone());
                        let _ = server.run_hook(Hook::OnPlayerDisconnect, vc).await;
                        let mut clients = clients.lock().unwrap();
                        clients.retain(|c| c.addr != update.client.addr);
                    }
                }
            }
        });
        tx
    }
}
