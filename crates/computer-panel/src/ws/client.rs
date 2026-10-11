use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    pin::pin,
    rc::Rc,
};

use computer_model::protocol::{message::ClientMessage, request::Request};
use dioxus::{
    hooks::Coroutine,
    signals::{ReadableExt, Signal},
};
use futures_channel::oneshot::{self, Sender};
use futures_util::future::{select, Either};
use gloo_timers::future::TimeoutFuture;

use crate::state::ConnectionStatus;

const TIMEOUT_MS: u32 = 10_000;

type Answer = Result<(), String>;

#[derive(Clone, Default)]
pub(crate) struct Pending(Rc<RefCell<HashMap<u64, Sender<Answer>>>>);

impl Pending {
    pub(crate) fn resolve(&self, id: u64, answer: Answer) {
        let sender = self.0.borrow_mut().remove(&id);
        if let Some(sender) = sender {
            let _ = sender.send(answer);
        }
    }

    pub(crate) fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

#[derive(Clone)]
pub struct Client {
    coroutine: Coroutine<ClientMessage>,
    status: Signal<ConnectionStatus>,
    next_id: Rc<Cell<u64>>,
    pending: Pending,
}

impl Client {
    pub(crate) fn new(
        coroutine: Coroutine<ClientMessage>,
        status: Signal<ConnectionStatus>,
        pending: Pending,
    ) -> Self {
        Self { coroutine, status, pending, next_id: Rc::default() }
    }

    pub async fn request(&self, request: Request) -> Answer {
        if !matches!(*self.status.peek(), ConnectionStatus::Connected) {
            return Err("Not connected".into());
        }

        let id = self.next_id.replace(self.next_id.get().wrapping_add(1));
        let (sender, receiver) = oneshot::channel();
        self.pending.0.borrow_mut().insert(id, sender);
        let _guard = Guard { pending: &self.pending, id };

        self.coroutine.send(ClientMessage::Request { id, request });

        match select(pin!(receiver), pin!(TimeoutFuture::new(TIMEOUT_MS))).await {
            Either::Left((answer, _)) => {
                answer.unwrap_or_else(|_| Err("Connection closed".into()))
            }
            Either::Right(_) => Err("The server did not answer in time".into()),
        }
    }
}

struct Guard<'a> {
    pending: &'a Pending,
    id: u64,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.pending.0.borrow_mut().remove(&self.id);
    }
}