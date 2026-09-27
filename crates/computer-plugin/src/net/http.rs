use std::collections::HashMap;

use crate::net::assets::Asset;

pub struct ParsedRequest {
    pub path: String,
    pub ws_key: Option<String>,
}

pub fn parse(buf: &[u8]) -> Result<Option<ParsedRequest>, ()> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut headers);

    match request.parse(buf) {
        Ok(httparse::Status::Complete(_)) => {}
        Ok(httparse::Status::Partial) => return Ok(None),
        Err(_) => return Err(()),
    };

    let path = request.path.unwrap_or("/").to_owned();

    let ws_key = request
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("sec-websocket-key"))
        .map(|header| String::from_utf8_lossy(header.value).into_owned());

    Ok(Some(ParsedRequest { path, ws_key }))
}

pub fn resolve<'a>(assets: &'a HashMap<String, Asset>, request_path: &str) -> Option<&'a Asset> {
    let path = request_path.split('?').next().unwrap_or("");
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    assets.get(path)
}