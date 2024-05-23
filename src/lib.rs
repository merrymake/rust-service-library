pub mod envelope;
pub mod merrymake;
pub mod mime_types;

pub use envelope::Envelope;
pub use merrymake::{
    broadcast_to_channel, join_channel, post_event_to_rapids, post_str_to_rapids, post_to_rapids,
    reply_file_to_origin, reply_file_to_origin_with_content_type, reply_str_to_origin,
    reply_to_origin,
};
pub use mime_types::MimeType;

/// This is the root call for a Merrymake service.
/// # Arguments
/// * `handlers` -- Used to link actions in the Merrymake.json file to code.
/// * `init` -- Used to define code to run after deployment but before release. Useful for smoke tests or database consolidation. Similar to an 'init container'
#[macro_export]
macro_rules! merrymake_service {
    ( { actions: { $( $action:literal : $handler:ident ) , * } $(, init: $init:ident )? } ) => {
        {
            use merrymake_service_library::merrymake::get_args;
            let (arg_action, envelope, payload) = get_args()?;
            match arg_action.as_str() {
                $(
                    $action => $handler(payload, envelope),
                )*
                $(
                    _ => $init(),
                )?
                _ => Ok(())
            }
        }
    };
}
