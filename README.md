# Rust Service Library for Merrymake

This is the official Rust service library for Merrymake. It defines all the basic functions needed to work with Merrymake.

## Usage

To add this library to your project dependencies add the following line in the `[dependencies]` section in your `Cargo.toml`:

```
merrymake-service-library = { git = "https://github.com/merrymake/rust-service-library.git", tag = "v0.2.0" }
```

Here is the most basic example of how to use this library: 

```rust
use merrymake_service_library::{merrymake_service, mime_types, reply_str_to_origin, Envelope};
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

pub fn handle_hello(_buffer: Vec<u8>, _envelope: Envelope) -> Result<(), String> {
    let payload = str::from_utf8(&_buffer).unwrap();
    reply_str_to_origin(format!("Hello, {}!", payload), mime_types::TXT).unwrap();
    Ok(())
}
```

## Tutorials and templates

For more information check out our tutorials at [merrymake.dev](https://merrymake.dev).

All templates are available through our CLI and on our [GitHub](https://github.com/merrymake).
