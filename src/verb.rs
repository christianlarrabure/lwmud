use crate::{client::Client, MudError, server::{hooks::HookCollection, MudServer}};
use futures::Future;
use serde_json::Value;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::RwLock;

pub type VerbOutput = Result<Value, MudError>;
pub type Verb = Box<
    dyn Fn(VerbContext) -> Pin<Box<dyn Future<Output = VerbOutput> + Send + Sync + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub type VerbCollection = Arc<RwLock<Vec<VerbInfo>>>;

pub struct VerbInfo {
    pub aliases: Vec<String>,
    pub verb: Verb,
}

impl VerbInfo {
    pub fn is_match(&self, verb: &str) -> bool {
        let verb = verb.to_lowercase();
        for alias in &self.aliases {
            if verb == alias.to_lowercase() {
                return true;
            }
        }
        false
    }
}

/// Before getting into a verb's context, we should determine what is a verb. A verb is any hook that can be called by other verbs.
/// There are verbs that are called by default, such as on_connect.
///
/// # VerbContext
/// The context of a verb provides all the information that a verb needs to run.
///
/// ## Client
/// The client is provided by some default verbs such as on_connect, and determines the client connecting to the game.
pub struct VerbContext {
    pub client: Option<Client>,
    pub input: Option<Vec<String>>,
    verbs: VerbCollection,
    hooks: HookCollection,
    clients: Option<Arc<Mutex<Vec<Client>>>>,
}

impl VerbContext {
    pub fn client(&self) -> Result<Client, MudError> {
        if let Some(client) = &self.client {
            Ok(client.clone())
        } else {
            Err(MudError::VerbNoClient)
        }
    }

    pub async fn run_verb(self, verb: &str) -> Result<Value, MudError> {
        let verbs = self.verbs.read().await;
        let verb = verbs.iter().find(|v| v.is_match(verb));
        if verb.is_none() {
            return Err(MudError::VerbNotFound);
        }
        let vc = VerbContextBuilder::new()
            .client(self.client.clone().unwrap())
            .build(self.verbs.clone(), self.hooks.clone());
        let verb = verb.unwrap();
        let res = (verb.verb)(vc).await?;
        Ok(res)
    }

    pub fn clients(&self) -> Result<Arc<Mutex<Vec<Client>>>, MudError> {
        if let Some(clients) = &self.clients {
            Ok(clients.clone())
        } else {
            Err(MudError::VerbnoClients)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VerbContextBuilder {
    client: Option<Client>,
    input: Option<Vec<String>>,
    clients: Option<Arc<Mutex<Vec<Client>>>>,
}

impl VerbContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clients(mut self, clients: Arc<Mutex<Vec<Client>>>) -> Self {
        self.clients = Some(clients);
        self
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn input(mut self, args: Vec<String>) -> Self {
        self.input = Some(args);
        self
    }

    pub fn server(mut self, server: MudServer) -> Self {
        self.clients = Some(server.clients);
        self
    }

    pub fn build(self, verbs: VerbCollection, hooks: HookCollection) -> VerbContext {
        VerbContext {
            client: self.client,
            verbs,
            input: self.input,
            clients: self.clients,
            hooks,
        }
    }
}
