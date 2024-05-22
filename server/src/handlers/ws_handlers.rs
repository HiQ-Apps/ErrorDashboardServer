use actix::{Actor, Addr, AsyncContext, ActorContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use uuid::Uuid;

use shared_types::error_dtos::ErrorDto;
use crate::managers::namespace_manager::{NamespaceServer, Subscribe, Unsubscribe};

#[derive(Debug, Clone)]
pub struct WsNamespaceSession {
    pub namespace_id: Uuid,
    pub addr: Addr<NamespaceServer>,
}

impl Actor for WsNamespaceSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr.do_send(Subscribe {
            namespace_id: self.namespace_id,
            addr: ctx.address(),
        });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        self.addr.do_send(Unsubscribe {
            namespace_id: self.namespace_id,
            addr: ctx.address(),
        });
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsNamespaceSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Binary(_)) => (),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewError(pub ErrorDto);

impl Handler<NewError> for WsNamespaceSession {
    type Result = ();

    fn handle(&mut self, msg: NewError, ctx: &mut Self::Context) {
        if msg.0.namespace_id == self.namespace_id {
            ctx.text(serde_json::to_string(&msg.0).unwrap());
        }
    }
}
