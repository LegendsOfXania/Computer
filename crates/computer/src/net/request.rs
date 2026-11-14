use computer_model::protocol::{event::Event, request::Request};

use crate::data::library;

pub fn handle(request: Request) -> Result<Event, String> {
    let result = match Event::from(request) {
        Event::Pushed => library::push().map(|()| Event::Pushed),
        event => library::apply(event),
    };

    if let Err(error) = &result {
        tracing::error!("Request failed: {error}");
    }

    result
}
