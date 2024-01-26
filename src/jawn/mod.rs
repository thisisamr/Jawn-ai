use asst::ThreadId;
use async_openai::{config::OpenAIConfig, Client};
use derive_more::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::remove_file;
//region: Modules
use crate::{
    ai_services::{
        asst::{
            self, create_thread, get_thread, load_or_create, run_thread_msg, upload_instructions,
            AsstId,
        },
        new_openai_client,
    },
    utils::{
        cli::ico_check,
        files::{ensure_dir, load_from_json, load_from_toml, read_to_string, save_to_file},
    },
    Result,
};

use self::config::Config;

mod config;
//endregion:Modules

const JAWN_TOML: &str = "jawn.toml";

#[derive(Debug)]
pub struct Jawn {
    pub dir: PathBuf,
    pub client: Client<OpenAIConfig>,
    pub asst_id: AsstId,
    pub config: Config,
}

#[derive(Debug, From, Deserialize, Serialize, Deref)]
pub struct Conv {
    pub thread_id: ThreadId,
}
/// public functions
impl Jawn {
    pub async fn init_from_dir(dir: impl AsRef<Path>, recreate_asst: bool) -> Result<Self> {
        let dir = dir.as_ref();
        //--load form the directory
        let config: Config = load_from_toml(dir.join(JAWN_TOML)).await?;
        let client = new_openai_client()?;
        let asst_id = load_or_create(&client, (&config).into(), recreate_asst).await?;

        let jawn = Jawn {
            dir: dir.into(),
            client,
            asst_id,
            config,
        };
        // upload instructions
        jawn.upload_instructions().await?;

        Ok(jawn)
    }

    pub async fn upload_instructions(&self) -> Result<bool> {
        let instructionfile = self.dir.join(&self.config.instruction_file);
        if instructionfile.exists() {
            let instruction_content = read_to_string(&instructionfile).await?;
            asst::upload_instructions(&self.client, &self.asst_id, instruction_content).await?;
            println!("{} instructions uploaded", ico_check());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn load_or_create_conv(&self, recreate: bool) -> Result<Conv> {
        let conv_file = self.data_dir().await?.join("conv.json");
        if recreate && conv_file.exists() {
            remove_file(&conv_file).await?;
        }
        // FIXME: NEED to handle serde parsing error
        let conv = if let Ok(conv) = load_from_json::<Conv>(&conv_file).await {
            get_thread(&self.client, &conv.thread_id)
                .await
                .map_err(|_| format!("Cannot find thread_id fro {:?}", conv))?;
            println!("{} Conversation loaded", ico_check());
            conv
        } else {
            let thread_id = create_thread(&self.client).await?;

            println!("{} Conversation created", ico_check());
            let conv = thread_id.into();
            save_to_file(&conv_file, &conv).await?;
            conv
        };
        Ok(conv)
    }
    pub async fn chat(&self, conv: &Conv, msg: &str) -> Result<String> {
        let res = run_thread_msg(&self.client, &self.asst_id, &conv.thread_id, msg).await?;
        Ok(res)
    }
}

/// private functions
impl Jawn {
    async fn data_dir(&self) -> Result<PathBuf> {
        let data_dir = self.dir.join(".jawn");
        ensure_dir(&data_dir).await?;
        Ok(data_dir)
    }
    async fn data_files_dir(&self) -> Result<PathBuf> {
        let dir = self.data_dir().await?.join("files");
        ensure_dir(&dir).await?; // FIXME
        Ok(dir)
    }
}
