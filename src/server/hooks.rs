use std::{pin::Pin, sync::Arc};

use futures::Future;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::{VerbContext, verb::{VerbOutput, VerbContextBuilder}, Verb, MudError};

use super::MudServer;

pub type HookCollection = Arc<RwLock<Vec<HookInfo>>>;

#[derive(PartialEq, Debug, Clone)]
pub enum Hook {
    OnPlayerConnect,
    OnPlayerDisconnect,
}

pub struct HookInfo {
    pub hook: Hook,
    pub verb: Verb,
}

impl HookInfo {
    pub fn is_match(&self, hook: Hook) -> bool {
        hook == self.hook
    }
}

impl MudServer {
    pub async fn add_hook<F, Fut>(&self, hook: Hook, verb: F)
    where
        F: Fn(VerbContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = VerbOutput> + Send + Sync + 'static,
    {
        let verb: Verb = Box::new(move |vc| {
            Box::pin(verb(vc)) as Pin<Box<dyn Future<Output = VerbOutput> + Send + Sync + 'static>>
        });
        let vi = HookInfo {
            hook, 
            verb,
        };
        let mut hooks = self.hooks.write().await;
        hooks.push(vi);
    }

    pub async fn run_hook(&self, hook_type: Hook, vc: VerbContextBuilder) -> Result<Value, MudError> {        let hooks = self.hooks.read().await;
        let mut matched = false;
        for hook in hooks.iter() {
            if hook.is_match(hook_type.clone()) {
                let vc = vc.clone().build(self.verbs.clone(), self.hooks.clone());
                let _ = (hook.verb)(vc).await?;
                matched = true;
            }
        }
        if matched {
            Ok(Value::Null)
        }else{
            Err(MudError::VerbNotFound)
            
        }
   }
}
