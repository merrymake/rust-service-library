# Rust Service Library for Merrymake

This is the official Rust service library for Merrymake. It defines all the basic functions needed to work with Merrymake.

## Usage

Here is the most basic example of how to use this library: 

```rust
use merrymake_tools::{merrymake_service, Envelope};
use std::str;

pub fn main() -> Result<(), String> {
    merrymake_service!(
        {
            actions: {
                "handleHello": handleHello
            }
        }
    )
}

pub fn handleHello(_buffer: Vec<u8>, _envelope: Envelope) -> Result<(), String> {
    let payload = str::from_utf8(&_buffer).unwrap();
    reply_str_to_origin(format!("Hello, {}!", payload), MimeType.TXT);
    Ok(())
}
```

## Tutorials and templates

For more information check out our tutorials at [merrymake.dev](https://merrymake.dev).

All templates are available through our CLI and on our [GitHub](https://github.com/merrymake).
