use actix::{Actor, Addr, Handler, Message, Context};
use uuid::Uuid;
use std::collections::HashMap;
use crate::handlers::ws_handlers::{WsNamespaceSession, NewError};

#[derive(Debug, Clone)]
pub struct NamespaceServer {
    sessions: HashMap<Uuid, Vec<Addr<WsNamespaceSession>>>,
}

impl NamespaceServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for NamespaceServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub namespace_id: Uuid,
    pub addr: Addr<WsNamespaceSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe {
    pub namespace_id: Uuid,
    pub addr: Addr<WsNamespaceSession>,
}

impl Handler<Subscribe> for NamespaceServer {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) {
        self.sessions
            .entry(msg.namespace_id)
            .or_insert_with(Vec::new)
            .push(msg.addr);
    }
}

impl Handler<Unsubscribe> for NamespaceServer {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) {
        if let Some(subs) = self.sessions.get_mut(&msg.namespace_id) {
            subs.retain(|addr| addr != &msg.addr);
        }
    }
}

impl Handler<NewError> for NamespaceServer {
    type Result = ();

    fn handle(&mut self, msg: NewError, _: &mut Self::Context) {
        if let Some(subs) = self.sessions.get(&msg.0.namespace_id) {
            for addr in subs {
                addr.do_send(NewError(msg.0.clone()));
            }
        }
    }
}
