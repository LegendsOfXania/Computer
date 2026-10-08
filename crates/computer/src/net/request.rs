use computer_model::protocol::{event::Event, request::Request};

use crate::data::library;

pub fn handle(request: Request) -> Result<Event, String> {
    let result = match request {
        Request::CreatePage { page } => library::create_page(page),
        Request::UpdatePage { page } => library::edit_page(page),
        Request::DeletePage { page_id } => library::delete_page(page_id),
        Request::CreateEntry { entry } => library::create_entry(entry),
        Request::UpdateEntry { entry } => library::edit_entry(entry),
        Request::DeleteEntry { key } => library::delete_entry(key),
        Request::MoveEntry { key, page_id } => library::move_entry(key, page_id),
        Request::Push => library::push().map(|()| Event::Pushed),
    };

    if let Err(error) = &result {
        tracing::error!("Request failed: {error}");
    }

    result
}
