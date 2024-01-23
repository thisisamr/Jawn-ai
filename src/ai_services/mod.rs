// region: Modules
pub mod asst;

use async_openai::{config::OpenAIConfig, Client};

use crate::Result;

// endregion: Modules

// region: Client
const OPEN_AI_KEY: &str = "OPENAI_API_KEY";

pub fn new_openai_client() -> Result<Client<OpenAIConfig>> {
    if std::env::var(OPEN_AI_KEY).is_ok() {
        Ok(Client::new())
    } else {
        println!("please provide a key ");
        Err("no api key found".into())
    }
}

// endregion: Client
