use crate::{mime_types, Envelope, MimeType};
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::{self, Read};

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
    let envelope = Envelope::new(envelope_str.as_str())?;
    let action = args
        .pop()
        .ok_or("unable to read 'action' from program arguments")?;

    Ok((action, envelope))
}

fn internal_post_to_rapids(
    event: &str,
    request_completer: impl FnOnce(RequestBuilder) -> Result<Response, reqwest::Error>,
) -> Result<(), String> {
    let rapids_url = env::var("RAPIDS").map_err(|_| "RAPIDS environment variable not set")?;
    let event_url = format!("{}/{}", rapids_url, event);

    let client = Client::new();
    let init_request_builder = client.post(&event_url);

    request_completer(init_request_builder)
        .map_err(|_| format!("unable to post event '{}' to url '{}'", event, event_url))?;

    Ok(())
}

/// Post an event to the central message queue (Rapids), with a payload and its
/// content type.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn post_to_rapids(event: &str, body: Vec<u8>, content_type: MimeType) -> Result<(), String> {
    internal_post_to_rapids(event, |r| {
        r.header("Content-Type", content_type.to_string())
            .body(body)
            .send()
    })
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
    internal_post_to_rapids(event, |r| {
        r.header("Content-Type", content_type.to_string())
            .body(body.into())
            .send()
    })
}

/// Post an event to the central message queue (Rapids), without a payload.
/// # Arguments
/// * `event` -- the event to post
pub fn post_event_to_rapids(event: &str) -> Result<(), String> {
    internal_post_to_rapids(event, |r| r.send())
}
/// Post a reply back to the originator of the trace, with a payload and its
/// content type.
/// # Arguments
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn reply_to_origin(body: Vec<u8>, content_type: MimeType) -> Result<(), String> {
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

    post_to_rapids("$reply", body, content_type)
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
