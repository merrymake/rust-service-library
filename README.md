# Rust Service Library for Merrymake

This is the official Rust service library for Merrymake. It defines all the basic functions needed to work with Merrymake.

## Getting Started

Add following dependency to your Mist service:

```
mist-tools = { git = "https://github.com/aekvi/mist-tools-rust.git", tag = "v0.1.1" }
```

Also, Mist requires an entry point '`app`'. You can specify your `src/main.rs` file as this entry point by adding the following to your `Cargo.toml`:

```
[[bin]]
name = "app"
path = "src/main.rs"
```

## Usage

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

pub fn handle_hello(buffer: Vec<u8>, _envelope: Envelope) -> Result<(), String> {
    let payload = str::from_utf8(&buffer).unwrap();
    reply_str_to_origin(format!("Hello, {}!", payload), mime_types::TXT).unwrap();
    Ok(())
}
```

## Tutorials and templates

For more information check out our tutorials at [merrymake.dev](https://merrymake.dev).

All templates are available through our CLI and on our [GitHub](https://github.com/merrymake).
