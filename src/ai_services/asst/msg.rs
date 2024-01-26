use async_openai::types::{CreateMessageRequest, MessageObject};

use crate::Result;

// region: Message Constructor
pub fn user_msg(content: impl Into<String>) -> CreateMessageRequest {
    CreateMessageRequest {
        role: "user".to_string(),
        content: content.into(),
        ..Default::default()
    }
}
// endregion: Message Constructor

// region: Content extraction

pub fn get_text_content(msg: MessageObject) -> Result<String> {
    let msg_content = msg
        .content
        .into_iter()
        .next()
        .ok_or_else(|| "no msg content ffound")?;

    let txt = match msg_content {
        async_openai::types::MessageContent::Text(text) => text.text.value,
        async_openai::types::MessageContent::ImageFile(image) => {
            return Err("not supporting images right now".into())
        }
    };
    Ok(txt)
}

// endregion: Content extraction
