use std::collections::HashMap;
use tokio::sync::{mpsc, watch, Mutex};
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use::shared_types::notification_dtos::NotificationDTO;

#[derive(Debug, Clone)]

pub struct NotificationServer {
    sessions: Arc<Mutex<HashMap<Uuid, Vec<mpsc::UnboundedSender<NotificationDTO>>>>>,
    api_state: watch::Sender<NotificationDTO>,
}

impl NotificationServer {
    pub fn new() -> Self {
        let (api_state_tx, _) = watch::channel(
            NotificationDTO {
                id: Uuid::new_v4(),
                title: String::new(),
                source: String::new(),
                user_id: Uuid::new_v4(),
                text: String::new(),
                is_read: false,
                created_at: Utc::now(),
            }
        );

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            api_state: api_state_tx
        }
    }

    pub async fn subscribe(&self, user_id: Uuid, tx: mpsc::UnboundedSender<NotificationDTO>) {
        let mut sessions = self.sessions.lock().await;
        sessions.entry(user_id).or_insert_with(Vec::new).push(tx);
    }

    pub async fn unsubscribe(&self, user_id: &Uuid, tx: &mpsc::UnboundedSender<NotificationDTO>) {
        let mut sessions = self.sessions.lock().await;
        if let Some(subscribers) = sessions.get_mut(user_id) {
            subscribers.retain(|subscriber| !subscriber.same_channel(tx));

            if subscribers.is_empty() {
                sessions.remove(user_id);
            }
        }
    }

    pub async fn broadcast_notification(&self, notification: NotificationDTO) {
        let sessions = self.sessions.lock().await;
        if let Some(subscribers) = sessions.get(&notification.id) {
            for tx in subscribers {
                let _ = tx.send(notification.clone());
            }
        }
    }

}
