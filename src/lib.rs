pub mod envelope;
pub mod merrymake;
pub mod mime_types;

pub use envelope::Envelope;
pub use merrymake::{
    post_event_to_rapids, post_str_to_rapids, post_to_rapids, reply_file_to_origin,
    reply_file_to_origin_with_content_type, reply_str_to_origin, reply_to_origin,
};
pub use mime_types::MimeType;

#[macro_export]
macro_rules! merrymake_service {
    ( { actions: { $( $action:literal : $handler:ident ) , * } $(, init: $init:ident )? } ) => {
        {
            use merrymake_service_library::merrymake::{get_args, get_payload};
            let (arg_action, envelope) = get_args()?;
            match arg_action.as_str() {
            $(
                $action => $handler(get_payload()?, envelope),
            )*
            $(
                _ => $init()?,
            )?
            _ => Ok(())
            }
        }
    };
}
