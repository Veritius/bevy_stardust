//! Functions that allow quick responses, ie telling the client something went wrong.

use std::io::Write;
use json::{object, JsonValue};
use super::WaitingClient;

// Avoid some copy and pasting
pub(super) fn respond_with(client: &mut WaitingClient, hiccup: bool, json: JsonValue) {
    let response = json.dump();
    let _ = client.stream.write(response.as_bytes());
    if hiccup { client.hiccups += 1; }
}

pub(super) fn respond_with_code(client: &mut WaitingClient, code: u16) {
    // Is the code being sent an error code?
    let hiccup = match code {
        _ => unimplemented!(),
    };

    respond_with(client, hiccup, object! { "code": code });
}