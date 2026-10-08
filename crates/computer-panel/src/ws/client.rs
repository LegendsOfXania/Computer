use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    pin::pin,
    rc::Rc,
};

use computer_model::protocol::{
    message::ClientMessage,
    request::Request,
};
use dioxus::{
    hooks::Coroutine,
    signals::{ReadableExt, Signal},
};
use futures_channel::oneshot::{self, Sender};
use futures_util::future::{select, Either};
use gloo_timers::future::TimeoutFuture;

use crate::state::ConnectionStatus;

const REQUEST_TIMEOUT_MS: u32 = 10_000;

pub(crate) type Pending =
    Rc<RefCell<HashMap<u64, Sender<Result<(), String>>>>>;

pub struct Client {
    coroutine: Coroutine<ClientMessage>,
    status: Signal<ConnectionStatus>,
    next_id: Cell<u64>,
    pending: Pending,
}

impl Client {
    pub(crate) fn new(
        coroutine: Coroutine<ClientMessage>,
        pending: Pending,
        status: Signal<ConnectionStatus>,
    ) -> Self {
        Self {
            coroutine,
            status,
            next_id: Cell::new(0),
            pending,
        }
    }

    pub async fn request(&self, request: Request) -> Result<(), String> {
        if !matches!(*self.status.peek(), ConnectionStatus::Connected) {
            return Err("Not connected".to_string());
        }

        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1));

        let (sender, receiver) = oneshot::channel();

        self.pending.borrow_mut().insert(id, sender);

        let _guard = PendingGuard {
            pending: Rc::clone(&self.pending),
            id,
        };

        self.coroutine.send(ClientMessage::Request { id, request });

        match select(pin!(receiver), pin!(TimeoutFuture::new(REQUEST_TIMEOUT_MS))).await {
            Either::Left((answer, _)) => {
                answer.map_err(|_| "Connection closed".to_string())?
            }

            Either::Right(_) => {
                Err("The server did not answer in time".to_string())
            }
        }
    }
}

struct PendingGuard {
    pending: Pending,
    id: u64,
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        self.pending.borrow_mut().remove(&self.id);
    }
}
