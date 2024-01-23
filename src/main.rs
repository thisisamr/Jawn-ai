#![allow(non_snake_case)]

use crate::ai_services::asst::{ load_or_create, CreateConfig};
use crate::ai_services::new_openai_client;
// region: Modules
mod ai_services;
mod error;

pub use self::error::{Error, Result};

// endregion: Modules
#[tokio::main]
async fn main() {
    match start().await {
        Ok(()) => println!("bye"),
        Err(e) => println!("error: {}", e),
    }
}

async fn start() -> Result<()> {
    let config = CreateConfig {
        name: "Jawn".to_string(),
        model: "gpt-3.5-turbo-1106".to_string(),
    };
    let client = new_openai_client()?;
    let asst_id = load_or_create(&client, config,false).await?;
    println!("this is the id--->{}", asst_id);
    Ok(())
}
