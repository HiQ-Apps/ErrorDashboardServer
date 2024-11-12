use actix_ws::{Message, Session};
use tokio::sync::mpsc;
use uuid::Uuid;
use std::sync::Arc;

use crate::managers::namespace_manager::NamespaceServer;

pub async fn namespace_error_ws_session(
    mut session: Session,
    namespace_id: Uuid,
    namespace_server: Arc<NamespaceServer>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    namespace_server.subscribe(namespace_id, tx.clone()).await;

    while let Some(msg) = rx.recv().await {
        let msg_text = serde_json::to_string(&msg).unwrap();
        if session.text(msg_text).await.is_err() {
            break;
        }
    }

    namespace_server.unsubscribe(&namespace_id, &tx).await;
}
