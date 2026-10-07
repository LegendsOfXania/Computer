use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use computer_model::protocol::{
    message::ClientMessage,
    request::Request,
};
use dioxus::hooks::Coroutine;
use futures_channel::oneshot::{self, Sender};

pub struct Client {
    coroutine: Coroutine<ClientMessage>,
    next_id: Cell<u64>,
    pending: Rc<RefCell<HashMap<u64, Sender<Result<(), String>>>>>,
}

impl Client {
    pub(crate) fn new(
        coroutine: Coroutine<ClientMessage>,
        pending: Rc<RefCell<HashMap<u64, Sender<Result<(), String>>>>>,
    ) -> Self {
        Self {
            coroutine,
            next_id: Cell::new(0),
            pending,
        }
    }

    pub async fn request(&self, request: Request) -> Result<(), String> {
        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1));

        let (sender, receiver) = oneshot::channel();

        self.pending.borrow_mut().insert(id, sender);

        self.coroutine.send(ClientMessage::Request { id, request });

        receiver
            .await
            .map_err(|_| "Connection closed".to_string())?
    }
}