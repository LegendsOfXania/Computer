use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    pin::pin,
    rc::Rc,
};

use computer_model::{
    page::{Page, PageKind},
    protocol::{message::ClientMessage, request::Request},
};
use dioxus::{
    hooks::Coroutine,
    signals::{ReadableExt, Signal},
};
use futures_channel::oneshot::{self, Sender};
use futures_util::future::{select, Either};
use gloo_timers::future::TimeoutFuture;

use crate::{id, state::{ConnectionStatus, app::AppState}};

const TIMEOUT_MS: u32 = 10_000;


#[derive(Clone, Default)]
pub(crate) struct Pending(Rc<RefCell<HashMap<u64, Sender<Result<(), String>>>>>);

impl Pending {
    pub(crate) fn resolve(&self, id: u64, answer: Result<(), String>) {
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

    pub async fn push(&self) -> Result<(), String> {
        self.request(Request::Push).await
    }

    pub async fn create_page(&self, name: String) -> Result<(), String> {
        let page = Page {
            id: id::generate(),
            name,
            kind: PageKind::Static,
            priority: 0,
            chapter: None,
            entries: Vec::new(),
        };

        self.request(Request::CreatePage { page }).await
    }

    pub async fn delete_page(&self, page_id: u64) -> Result<(), String> {
        self.request(Request::DeletePage { page_id }).await
    }

    pub async fn edit_page(
        &self,
        app: &AppState,
        page_id: u64,
        change: impl FnOnce(&mut Page),
    ) -> Result<(), String> {
        let Some(mut page) = app.with_page(page_id, Clone::clone) else {
            return Ok(());
        };

        change(&mut page);
        self.request(Request::UpdatePage { page }).await
    }

    async fn request(&self, request: Request) -> Result<(), String> {
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