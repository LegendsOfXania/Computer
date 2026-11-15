use std::{collections::HashMap, pin::pin};

use computer_model::protocol::{message::ClientMessage, request::Request};
use dioxus::{
    hooks::Coroutine,
    signals::{CopyValue, ReadableExt, Signal, WritableExt},
};
use futures_channel::oneshot::{self, Sender};
use futures_util::future::{Either, select};
use gloo_timers::future::TimeoutFuture;

use crate::state::ConnectionStatus;

const TIMEOUT_MS: u32 = 10_000;

type Answer = Result<(), String>;

#[derive(Clone, Copy)]
pub(crate) struct Pending(CopyValue<HashMap<u64, Sender<Answer>>>);

impl Pending {
    pub(crate) fn new() -> Self {
        Self(CopyValue::new(HashMap::new()))
    }

    pub(crate) fn resolve(mut self, id: u64, answer: Answer) {
        let sender = self.0.write().remove(&id);

        if let Some(sender) = sender {
            let _ = sender.send(answer);
        }
    }

    pub(crate) fn clear(mut self) {
        self.0.write().clear();
    }
}

#[derive(Clone, Copy)]
pub struct Client {
    coroutine: Coroutine<ClientMessage>,
    status: Signal<ConnectionStatus>,
    next_id: CopyValue<u64>,
    pending: Pending,
}

impl Client {
    pub(crate) fn new(
        coroutine: Coroutine<ClientMessage>,
        status: Signal<ConnectionStatus>,
        pending: Pending,
    ) -> Self {
        Self {
            coroutine,
            status,
            next_id: CopyValue::new(0),
            pending,
        }
    }

    pub async fn request(mut self, request: Request) -> Answer {
        if *self.status.peek() != ConnectionStatus::Connected {
            return Err("Not connected".into());
        }

        let id = self.next_id.with_mut(|next| {
            let id = *next;
            *next = id.wrapping_add(1);
            id
        });

        let (sender, receiver) = oneshot::channel();

        self.pending.0.write().insert(id, sender);
        self.coroutine.send(ClientMessage::Request { id, request });

        match select(pin!(receiver), pin!(TimeoutFuture::new(TIMEOUT_MS))).await {
            Either::Left((answer, _)) => answer.unwrap_or_else(|_| Err("Connection closed".into())),
            Either::Right(_) => {
                self.pending.0.write().remove(&id);

                Err("The server did not answer in time".into())
            }
        }
    }
}
