use merrymake_service_library::{merrymake_service, reply_str_to_origin, Envelope};
use std::str;

pub fn main() -> Result<(), String> {
    merrymake_service!(
        {
            actions: {
                "handleHello": handle_hello
            }
        }
    )
}

pub fn handle_hello(buffer: Vec<u8>, _envelope: Envelope) -> Result<(), String> {
    let payload = str::from_utf8(&buffer).unwrap();
    reply_str_to_origin(format!("Hello, {}!", payload).as_str()).unwrap();
    Ok(())
}
