use computer_model::protocol::{event::Event, request::Request};

use crate::data::library;

pub fn handle(request: Request) -> Result<Event, String> {
    let result = match request {
        Request::CreatePage { page } => library::create_page(page),
        Request::UpdatePage { page } => library::edit_page(page),
        Request::DeletePage { page_id } => library::delete_page(page_id),
        Request::CreateEntry { entry, page_id } => library::create_entry(entry, page_id),
        Request::UpdateEntry { entry } => library::edit_entry(entry),
        Request::DeleteEntry { entry_id } => library::delete_entry(entry_id),
        Request::MoveEntry { entry_id, page_id } => library::move_entry(entry_id, page_id),
        Request::Push => library::push().map(|()| Event::Pushed),
    };

    if let Err(error) = &result {
        tracing::error!("Request failed: {error}");
    }

    result
}
