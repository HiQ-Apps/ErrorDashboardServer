use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use uuid::Uuid;

use ::shared_types::error_dtos::CreateErrorDTO;

#[derive(Debug, Clone)]
pub struct NamespaceServer {
    sessions: Arc<Mutex<HashMap<Uuid, Vec<mpsc::UnboundedSender<CreateErrorDTO>>>>>,
    api_state: watch::Sender<CreateErrorDTO>,
}

impl NamespaceServer {
    pub fn new() -> Self {
        let (api_state_tx, _) = watch::channel(CreateErrorDTO {
            id: Uuid::new_v4(),
            message: String::new(),
            resolved: false,
            namespace_id: Uuid::new_v4(),
            user_affected: String::new(),
            stack_trace: String::new(),
        });

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            api_state: api_state_tx,
        }
    }

    pub async fn subscribe(&self, namespace_id: Uuid, tx: mpsc::UnboundedSender<CreateErrorDTO>) {
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(namespace_id)
            .or_insert_with(Vec::new)
            .push(tx);
    }

    pub async fn unsubscribe(
        &self,
        namespace_id: &Uuid,
        tx: &mpsc::UnboundedSender<CreateErrorDTO>,
    ) {
        let mut sessions = self.sessions.lock().await;
        if let Some(subscribers) = sessions.get_mut(namespace_id) {
            subscribers.retain(|subscriber| !subscriber.same_channel(tx));

            if subscribers.is_empty() {
                sessions.remove(namespace_id);
            }
        }
    }

    pub async fn broadcast_error(&self, error: CreateErrorDTO) {
        let sessions = self.sessions.lock().await;
        if let Some(subscribers) = sessions.get(&error.namespace_id) {
            for tx in subscribers {
                let _ = tx.send(error.clone());
            }
        }
    }
}
