use actix::{Message, Recipient};
use uuid::Uuid;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient {
    pub namespace_id: Uuid,
    pub client: Recipient<BroadcastMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterClient {
    pub namespace_id: Uuid,
    pub client: Recipient<BroadcastMessage>,
}
