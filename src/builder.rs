use std::sync::{Arc, Mutex};

use crate::server::MudServer;
use crate::MudError;
extern crate tokio;

#[derive(Debug)]
pub struct MudServerConfig {
    name: Option<String>,
    host: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MudServerBuilder {
    config: Arc<Mutex<MudServerConfig>>,
}

impl Default for MudServerBuilder {
    fn default() -> Self {
        MudServerBuilder {
            config: Arc::new(Mutex::new(MudServerConfig {
                name: None,
                host: None,
            }))
        }
    }
}

impl MudServerBuilder {
    pub fn new() -> Self {
        Self::default()
   }

    /// Bind the server to a specific address.
    /// It should be in the format of 127.0.0.1:4000.
    pub fn bind(&mut self, addr: &str) -> Self {
        {
            let mut config = self.config.lock().unwrap();
            config.host = Some(addr.into());
        }
        self.clone()
    }

    /// Sets the name of the server.
    pub fn name(&mut self, name: &str) -> Self {
        {
            let mut config = self.config.lock().unwrap();
            config.name = Some(name.into());
        }
        self.clone()
    }

    pub async fn run(&self) -> Result<MudServer, MudError> {
        let config = self.config.lock().unwrap();
        let addr = config.host.as_ref().ok_or(MudError::NoAddress)?;
        let name = config.name.clone().unwrap_or_else(|| "Server".into());
        println!("{} listening on {}...", name, addr);

        let server = MudServer::new(name, addr.clone());
        Ok(server)
    }
}
