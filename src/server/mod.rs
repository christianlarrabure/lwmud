mod connections;
pub mod hooks;

use futures::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, RwLock};

use crate::client::{Client, ClientInput};
use crate::verb::{VerbCollection, VerbContext, VerbContextBuilder, VerbInfo, VerbOutput};
use crate::{MudError, Verb};

pub use connections::{ConnectionStatus, ConnectionUpdate};

use self::hooks::HookCollection;

#[derive(Clone)]
pub struct MudServer {
    pub name: String,
    pub host: String,
    pub clients: Arc<Mutex<Vec<Client>>>,
    pub verbs: VerbCollection,
    pub hooks: HookCollection,
    pub connections: Option<mpsc::Sender<ConnectionUpdate>>,
}

impl MudServer {
    pub fn new(name: String, host: String) -> Self {
        MudServer {
            name,
            host,
            clients: Arc::new(Mutex::new(Vec::new())),
            verbs: Arc::new(RwLock::new(Vec::new())),
            hooks: Arc::new(RwLock::new(Vec::new())),
            connections: None,
        }
    }

    pub async fn listen(&self, mut channel: Receiver<ClientInput>) {
        let verbs = self.verbs.clone();
        let hooks = self.hooks.clone();
        let clients = self.clients.clone();
        tokio::spawn(async move {
            while let Some(input) = channel.recv().await {
                let client = input.client;
                let clients = clients.clone();
                let raw_input = input.raw;
                if raw_input.is_empty() {
                    continue;
                }
                let input = raw_input.split_whitespace();
                let input: Vec<String> = input.map(|s| s.to_string()).collect();
                let vc = VerbContextBuilder::new()
                    .client(client.clone())
                    .clients(clients.clone())
                    .input(input)
                    .build(verbs.clone(), hooks.clone());
                let verb = vc.input.clone().unwrap()[0].clone();
                let verb = verb.to_lowercase().trim().to_string();
                let verbs = verbs.read().await;
                let verb = verbs.iter().find(|v| v.is_match(&verb));
                if verb.is_none() {
                    client.send("Unknown command.");
                    continue;
                }
                let verb = verb.unwrap();
                let _ = (verb.verb)(vc).await;
            }
        });
    }

    pub async fn setup(mut self) -> Result<(), MudError> {
        let connections = self.create_connections_channel().await;
        self.connections = Some(connections);
        let (tx, rx) = mpsc::channel::<ClientInput>(100);
        self.listen(rx).await;
        let listener = TcpListener::bind(&self.host)
            .await
            .map_err(|_| MudError::InvalidAddress)?;
        loop {
            let tx = tx.clone();
            let incoming_connection = listener.accept().await;
            if let Err(e) = incoming_connection {
                println!("ERROR: {}", e);
                continue;
            }
            let (socket, _) = incoming_connection.unwrap();
            let _ = Client::new(socket, tx, self.connections.clone().unwrap());
        }
    }

    pub async fn add_verb<F, Fut>(&self, aliases: Vec<&str>, verb: F)
    where
        F: Fn(VerbContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = VerbOutput> + Send + Sync + 'static,
    {
        let verb: Verb = Box::new(move |vc| {
            Box::pin(verb(vc)) as Pin<Box<dyn Future<Output = VerbOutput> + Send + Sync + 'static>>
        });
        let vi = VerbInfo {
            aliases: aliases.into_iter().map(|s| s.to_string()).collect(),
            verb,
        };
        let mut verbs = self.verbs.write().await;
        verbs.push(vi);
    }
}
