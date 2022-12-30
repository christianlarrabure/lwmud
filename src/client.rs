use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};

 use crate::server::{ConnectionStatus, ConnectionUpdate};

/// A client represents any connection to the server.
#[derive(Debug, Clone)]
pub struct Client {
    pub addr: String,
    // bytes
    write_channel: mpsc::Sender<Vec<u8>>,
    connections: mpsc::Sender<ConnectionUpdate>,
}

impl Client {
    pub fn new(
        conn: TcpStream,
        output_channel: Sender<ClientInput>,
        connections: Sender<ConnectionUpdate>,
    ) -> Self {
        let addr = &conn.peer_addr().unwrap().to_string();

        let (mut read, mut write) = Self::get_read_write(conn);
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                write.write_all(&bytes).await.unwrap();
            }
        });
        let client = Client {
            addr: addr.clone(),
            write_channel: tx,
            connections,
        };
        {
            let client = client.clone();
            tokio::spawn(async move {
                let output_channel = output_channel.clone();
                let mut input = String::new();
                while let Ok(len) = read.read_line(&mut input).await {
                    let line = input.trim().to_string();
                    if len == 0 {
                        break;
                    }
                    let client_input = ClientInput {
                        client: client.clone(),
                        raw: line,
                    };
                    output_channel.send(client_input).await.unwrap();
                    input.clear();
                }
                client.on_disconnect().await;
            });
        }
        {
            let client = client.clone();
            tokio::spawn(async move {
                let client = client.clone();
                client.on_connect().await;
            });
        }
        client
    }

    async fn on_connect(self) {
        let connections = self.connections.clone();
        let _ = connections
            .send(ConnectionUpdate {
                client: self,
                status: ConnectionStatus::Connected,
            })
            .await;
    }

    async fn on_disconnect(self) {
        let connections = self.connections.clone();
        let _ = connections
            .send(ConnectionUpdate {
                client: self,
                status: ConnectionStatus::Disconnected,
            })
            .await;
    }

    fn get_read_write(conn: TcpStream) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
        let (read, write) = conn.into_split();
        let read = BufReader::new(read);
        (read, write)
    }

    pub fn send(&self, bytes: &str) {
        let channel = self.write_channel.clone();
        let payload = bytes.as_bytes().to_vec();
        tokio::spawn(async move {
            channel.send(payload).await.unwrap();
        });
    }
}

#[derive(Debug)]
pub struct ClientInput {
    pub client: Client,
    pub raw: String,
}
