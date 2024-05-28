use crate::{mime_types, Envelope, MimeType};
use serde::Serialize;
use std::env;
use std::io::{self, Read, Write};
use std::net::{self, Shutdown};

pub struct Headers {
    content_type: MimeType,
}

/// Post an event to the central message queue (Rapids) with a payload.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
pub fn post_to_rapids(event: &str, body: &[u8]) -> Result<(), String> {
    let packed = pack(event, &body).unwrap();
    let addr = env::var("RAPIDS").map_err(|_| "RAPIDS environment variable not set")?;
    let mut stream = net::TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream.write_all(&packed).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    stream.shutdown(Shutdown::Both).map_err(|e| e.to_string())
}

/// Post an event to the central message queue (Rapids) with a string payload.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
pub fn post_str_to_rapids(event: &str, body: impl Into<String>) -> Result<(), String> {
    post_to_rapids(event, body.into().as_bytes())
}

/// Post an event to the central message queue (Rapids) with a json payload.
/// # Arguments
/// * `event` --       the event to post
/// * `body` --        the payload
pub fn post_json_to_rapids<T: serde::Serialize + ?Sized>(
    event: &str,
    body: &T,
) -> Result<(), String> {
    let body = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    post_to_rapids(event, body.as_slice())
}

/// Post an event to the central message queue (Rapids) without a payload.
/// # Arguments
/// * `event` -- the event to post
pub fn post_event_to_rapids(event: &str) -> Result<(), String> {
    post_to_rapids(event, &[])
}

/// Post a reply back to the originator of the trace, with a payload and its
/// content type.
/// # Arguments
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn reply_to_origin(body: &[u8], headers: &Headers) -> Result<(), String> {
    #[derive(Serialize)]
    pub struct InternalHeaders {
        #[serde(rename = "contentType")]
        content_type: String,
    }
    #[derive(Serialize)]
    struct Reply {
        headers: InternalHeaders,
        content: Vec<u8>,
    }
    let reply = Reply {
        headers: InternalHeaders {
            content_type: headers.content_type.to_string(),
        },
        content: body.to_vec(),
    };
    post_to_rapids("$reply", &serde_json::to_vec(&reply).unwrap())
}

/// Post a reply back to the originator of the trace, with a payload and its
/// content type.
/// # Arguments
/// * `body` --        the payload
/// * `contentType` -- the content type of the payload
pub fn reply_str_to_origin(body: &str) -> Result<(), String> {
    reply_to_origin(
        body.as_bytes(),
        &Headers {
            content_type: mime_types::TXT,
        },
    )
}

pub fn get_args() -> Result<(String, Envelope, Vec<u8>), String> {
    let bytes = get_bytes()?;
    let (action_bytes, i) = read_next_byte_chunk(&bytes)?;
    let action = String::from_utf8(action_bytes).map_err(|e| e.to_string())?;
    let (envelope_bytes, j) = read_next_byte_chunk(&bytes[i..])?;
    let envelope = Envelope::from_bytes(&envelope_bytes)?;
    let (payload, _) = read_next_byte_chunk(&bytes[(i + j)..])?;
    Ok((action, envelope, payload))
}

fn pack(event: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    let event = event.as_bytes();
    let bytes = vec![
        &length_to_bytes(&event.len())[..],
        event,
        &length_to_bytes(&body.len())[..],
        body,
    ]
    .concat();
    Ok(bytes)
}

fn get_bytes() -> Result<Vec<u8>, &'static str> {
    let mut buffer = Vec::with_capacity(16);
    io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|_| "unable to read from stdin")?;
    Ok(buffer)
}

fn length_to_bytes(length: &usize) -> [u8; 3] {
    [(length >> 16) as u8, (length >> 8) as u8, *length as u8]
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

fn read_next_byte_chunk(bytes: &[u8]) -> Result<(Vec<u8>, usize), &'static str> {
    let len = bytes_to_index(&bytes[..3])?;
    let end = len + 3;
    Ok((Vec::from(&bytes[3..end]), end))
}
