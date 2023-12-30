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
    pub fn new(json: &str) -> Result<Self, &'static str> {
        let envelope: Envelope =
            serde_json::from_str(json).map_err(|_| "Unable to parse envelope from json")?;
        Ok(envelope)
    }
}
