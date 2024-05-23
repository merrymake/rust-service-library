use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Envelope {
    /// Id of this particular message.
    /// Note: it is _not_ unique, since multiple rivers can deliver the same message.
    /// The combination of (river, messageId) is unique.
    #[serde(alias = "messageId")]
    pub message_id: String,
    /// Id shared by all messages in the current trace, ie. stemming from the same
    /// origin.
    #[serde(alias = "traceId")]
    pub trace_id: String,
    /// (Optional) Id corresponding to a specific originator. This id is rotated
    /// occasionally, but in the short term it is unique and consistent. Same
    /// sessionId implies the trace originated from the same device.
    #[serde(alias = "sessionId")]
    pub session_id: Option<String>,
}

impl Envelope {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        serde_json::from_slice(bytes).map_err(|_| "Unable to parse envelope from json")
    }
}

impl FromStr for Envelope {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| "Unable to parse envelope from json")
    }
}
