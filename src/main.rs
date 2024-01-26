#![allow(non_snake_case)]
#![allow(unused)]

use crate::ai_services::asst::{
    create_thread, load_or_create, run_thread_msg, upload_instructions, CreateConfig,
};
use crate::ai_services::new_openai_client;
use crate::utils::cli::{ico_check, ico_res, prompt, txt_res};
use dotenv::dotenv;
use jawn::Jawn;
use textwrap::wrap;
// region: Modules
mod ai_services;
mod error;
mod jawn;
mod utils;
pub use self::error::{Error, Result};

// endregion: Modules

// region: Types
/// Input command from the user
enum Cmd {
    Quit,
    Chat(String),
    RefreshAll,
    RefreshConv,
    REfreshInst,
    RefreshFiles,
}
impl Cmd {
    fn from_input(input: impl Into<String>) -> Self {
        let input = input.into();
        if input == "/q" {
            Self::Quit
        } else if input == "/r" || input == "/ra" {
            Self::RefreshAll
        } else if input == "/ri" {
            Self::REfreshInst
        } else if input == "/rf" {
            Self::RefreshFiles
        } else if input == "/rc" {
            Self::RefreshConv
        } else {
            Self::Chat(input)
        }
    }
}
// endregion: Types
#[tokio::main]
async fn main() {
    dotenv().ok();
    match start().await {
        Ok(()) => println!("bye"),
        Err(e) => println!("error: {}", e),
    }
}
const DEFAULT_DIR: &str = "jawn";
async fn start() -> Result<()> {
    let mut j = Jawn::init_from_dir(DEFAULT_DIR, false).await?;
    let mut conv = j.load_or_create_conv(false).await?;
    loop {
        println!();
        let input = prompt("Ask Me ")?;
        let cmd = Cmd::from_input(input);
        use Cmd::*;
        match cmd {
            Quit => break,
            Chat(msg) => {
                let res = j.chat(&conv, &msg).await?;
                let res = wrap(&res, 80).join("\n");
                println!("{} {}", ico_res(), txt_res(res));
            }
            other => println!("command not supported"),
        }
    }
    println!("hello world");
    Ok(())
}
