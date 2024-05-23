use crate::{mime_types, Envelope, MimeType};
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net;
use std::str::FromStr;
use ureq::{Request, Response};

pub fn get_payload() -> Result<Vec<u8>, &'static str> {
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|_| "unable to read from stdin")?;
    Ok(buffer)
}

pub fn get_args() -> Result<(String, Envelope), &'static str> {
    let mut args: Vec<_> = env::args().collect();
    let envelope_str = args
        .pop()
        .ok_or("unable to read 'envelope' from program arguments")?;
    let envelope = Envelope::from_str(envelope_str.as_str())?;
    let action = args
        .pop()
        .ok_or("unable to read 'action' from program arguments")?;

    Ok((action, envelope))
}

/// Reads the bytes from stdin.
fn get_bytes() -> Result<Vec<u8>, String> {
    let mut bytes: Vec<u8> = Vec::with_capacity(16); // 16 bytes is a fair minimum capacity
    let _ = io::stdin()
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn length_to_bytes(length: usize) -> [u8; 3] {
    todo!()
}

fn bytes_to_index(bytes: &[u8]) -> Result<usize, &'static str> {
    if bytes.len() < 3 {
        Err("byte vector too small to interpret as number")
    } else {
        let left = usize::from(bytes[0]) << 16;
        let mid = usize::from(bytes[1]) << 8;
        let right = usize::from(bytes[2]);
        Ok(left | mid | right)
    }
}

/// TODO: Maybe return `end` index instead so we avoid copying the bytes
fn read_next_byte_chunk(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
    let len = bytes_to_index(&bytes[..3])?;
    let end = len + 3;
    Ok((Vec::from(&bytes[3..end]), Vec::from(&bytes[end..])))
}

/// Used for tcp
pub fn get_args_and_action() -> Result<(String, Envelope, Vec<u8>), String> {
    let bytes = get_bytes()?;
    let (action_bytes, rest_bytes1) = read_next_byte_chunk(&bytes)?;
    let (envelope_bytes, rest_bytes2) = read_next_byte_chunk(&rest_bytes1)?;
    let (payload, _) = read_next_byte_chunk(&rest_bytes2)?;
    let action = String::from_utf8(action_bytes).map_err(|e| e.to_string())?;
    let envelope = Envelope::from_bytes(&envelope_bytes)?;
    Ok((action, envelope, payload))
}

/// Returns `true` if the `tcp` feature is enabled.
pub fn tcp_is_enabled() -> bool {
    env::args().count() == 2
}

fn internal_http_post_to_rapids(
    event: &str,
    request_completer: impl FnOnce(Request) -> Result<Response, ureq::Error>,
) -> Result<(), String> {
    let rapids_url = env::var("RAPIDS").map_err(|_| "RAPIDS environment variable not set")?;
    let event_url = format!("{}/{}", rapids_url, event);

    let init_request_builder = ureq::post(&event_url);

    request_completer(init_request_builder)
        .map_err(|_| format!("unable to post event '{}' to url '{}'", event, event_url))?;

    Ok(())
}

fn pack(event: &str, body: &[u8], content_type: &MimeType) -> Result<Vec<u8>, String> {
    if event == "$reply" {
        // { content: Vec<u8>, headers: { contentType: String } }
        todo!()
    } else {
        let event = serde_json::to_vec(event).map_err(|e| e.to_string())?;
        let body = Vec::from(body);
        let bytes = vec![event, body].concat();
        Ok(bytes)
    }
}

/// Post an event to the central message queue (Rapids), with a payload and its
/// content type.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn post_to_rapids(event: &str, body: &[u8], content_type: &MimeType) -> Result<(), String> {
    if tcp_is_enabled() {
        let content = pack(event, body, content_type)?;
        internal_tcp_post_to_rapids(&content)
    } else {
        internal_http_post_to_rapids(event, |r| {
            r.set("Content-Type", content_type.to_string().as_str())
                .send_bytes(body)
        })
    }
}

fn internal_tcp_post_to_rapids(bytes: &[u8]) -> Result<(), String> {
    let addr = env::var("RAPIDS").map_err(|_| "RAPIDS environment variable not set")?;
    let mut stream = net::TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream.write_all(bytes).map_err(|e| e.to_string())
}

/// Post an event to the central message queue (Rapids), with a payload and its
/// content type.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn post_str_to_rapids(
    event: &str,
    body: impl Into<String>,
    content_type: MimeType,
) -> Result<(), String> {
    internal_http_post_to_rapids(event, |r| {
        r.set("Content-Type", content_type.to_string().as_str())
            .send_string(body.into().as_str())
    })
}

/// Post an event to the central message queue (Rapids), without a payload.
/// # Arguments
/// * `event` -- the event to post
pub fn post_event_to_rapids(event: &str) -> Result<(), String> {
    internal_http_post_to_rapids(event, |r| r.call())
}
/// Post a reply back to the originator of the trace, with a payload and its
/// content type.
/// # Arguments
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn reply_to_origin(body: &[u8], content_type: MimeType) -> Result<(), String> {
    post_to_rapids("$reply", body, content_type)
}

/// Post a reply back to the originator of the trace, with a payload and its
/// content type.
/// # Arguments
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn reply_str_to_origin(body: impl Into<String>, content_type: MimeType) -> Result<(), String> {
    post_str_to_rapids("$reply", body, content_type)
}

/// Send a file back to the originator of the trace.
/// # Arguments
/// * `path` --        the path to the file starting from main/resources
/// * `contentType` -- the content type of the file
pub fn reply_file_to_origin_with_content_type(
    path: &str,
    content_type: MimeType,
) -> Result<(), String> {
    let mut file = File::open(path).map_err(|_| format!("unable to open file '{}'", path))?;

    let mut body = Vec::new();
    file.read_to_end(&mut body)
        .map_err(|_| format!("unable to read file '{}'", path))?;

    post_to_rapids("$reply", &body, content_type)
}

/// Send a file back to the originator of the trace.
/// # Arguments
/// * `path` -- the path to the file starting from main/resources
pub fn reply_file_to_origin(path: &str) -> Result<(), String> {
    let file_ext = path.split('.').last();
    match file_ext {
        Some(f) => {
            let content_type = mime_types::ext2mime(f);
            match content_type {
                Some(ct) => reply_file_to_origin_with_content_type(path, ct),
                None => Err(format!("unknown file extension from file path '{}'", path)),
            }
        }
        None => Err(format!(
            "unable to locate file extension from file path '{}'",
            path
        )),
    }
}

// Subscribe to a channel, so events will stream back messages broadcast to that
/// channel. You can join multiple channels. You stay in the channel until the
/// request is terminated.
///
/// Note: The origin-event has to be set as "streaming: true" in the
/// event-catalogue.
/// # Arguments
/// * `channel` -- the channel to join
pub fn join_channel(channel: impl Into<String>) -> Result<(), String> {
    post_str_to_rapids("$join", channel, mime_types::TXT)
}
/// Broadcast a message (event and payload) to all listeners in a channel.
/// # Arguments
/// * `to` -- the channel to broadcast to
/// * `event` -- the event-type of the message
/// * `payload` -- the payload of the message
pub fn broadcast_to_channel(
    to: impl Into<String>,
    event: impl Into<String>,
    payload: impl Into<String>,
) -> Result<(), String> {
    #[derive(Serialize)]
    struct Body {
        to: String,
        event: String,
        payload: String,
    }

    post_str_to_rapids(
        "$broadcast",
        serde_json::to_string(&Body {
            to: to.into(),
            event: event.into(),
            payload: payload.into(),
        })
        .unwrap(),
        mime_types::JSON,
    )
}

#[cfg(test)]
mod test {}
