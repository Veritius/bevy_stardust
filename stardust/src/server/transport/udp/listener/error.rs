use std::io::Write;

use json::{Error, object};
use super::WaitingClient;

pub(super) fn json_error(
    client: &mut WaitingClient,
    _err: Error,
) {
    let response = object! {
        "code": 200
    };

    let response = response.dump();
    let _ = client.stream.write(response.as_bytes());
    client.hiccups += 1;
}