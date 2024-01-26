use std::io::Write;
use std::time::Duration;

use crate::ai_services::asst::msg::{get_text_content, user_msg};
use crate::utils::cli::{ico_check, ico_deleted_ok};
use crate::Result;
use async_openai::types::{
    AssistantObject, CreateAssistantRequest, CreateRunRequest, CreateThreadRequest,
    ModifyAssistantRequest, RunStatus, ThreadObject,
};
use async_openai::{config::OpenAIConfig, types::AssistantToolsRetrieval, Client};
use console::Term;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// region: Modules

pub mod msg;

// endregion: Modules
// region: constants
const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];
const POLLING_DURATION: u16 = 500;
// endregion: Constants

// region:Types
pub struct CreateConfig {
    pub name: String,
    pub model: String,
}

#[derive(Debug, From, Display, Deref)]
pub struct AsstId(String);

#[derive(Debug, From, Display, Deref, Serialize, Deserialize)]
pub struct ThreadId(String);

#[derive(Debug, From, Display, Deref)]
pub struct FileId(String);
// endregion:Types

// region: Assistant Crud
pub async fn find_by_name(
    client: &Client<OpenAIConfig>,
    name: &str,
) -> Result<Option<AssistantObject>> {
    let assitants = client.assistants().list(DEFAULT_QUERY).await?.data;
    let assistant = assitants
        .into_iter()
        .find(|asst| asst.name.is_some() && asst.name.as_ref() == Some(&name.to_string()));
    Ok(assistant)
}

pub async fn create_assistant(
    client: &Client<OpenAIConfig>,
    config: CreateConfig,
) -> Result<AsstId> {
    let assistants = client.assistants();
    let asst_obj = assistants
        .create(CreateAssistantRequest {
            model: config.model,
            name: Some(config.name),
            tools: Some(vec![AssistantToolsRetrieval::default().into()]),
            ..Default::default()
        })
        .await?;
    Ok(asst_obj.id.into())
}
pub async fn delete_assistant(client: &Client<OpenAIConfig>, asst_id: &AsstId) -> Result<()> {
    let assistants = client.assistants();

    // TODO: delete files

    // delete the assistant
    assistants.delete(&asst_id).await?;
    Ok(())
}
// endregion: Assistant Crud

pub async fn load_or_create(
    client: &Client<OpenAIConfig>,
    config: CreateConfig,
    recreate: bool,
) -> Result<AsstId> {
    let assistant_object = find_by_name(client, &config.name).await?;
    let mut assistant_id = assistant_object.map(|o| AsstId::from(o.id));

    if let (true, Some(asst_id)) = (recreate, assistant_id.as_ref()) {
        // FixME: delete
        delete_assistant(&client, asst_id).await?;
        assistant_id.take();
        println!("{} assistant {} deleted", ico_deleted_ok(), &config.name);
    }
    if let Some(assistant_id) = assistant_id {
        println!("{} assistant {} loaded", ico_check(), &config.name);
        Ok(assistant_id)
    } else {
        let asst_name = config.name.clone();
        let assistant = create_assistant(client, config).await?;
        println!("{} assistant {} ", ico_check(), &asst_name);
        Ok(assistant)
    }
}

pub async fn upload_instructions(
    client: &Client<OpenAIConfig>,
    asst_id: &AsstId,
    instructions: String,
) -> Result<()> {
    let assistants = client.assistants();
    let modify_request = ModifyAssistantRequest {
        instructions: Some(instructions),
        ..Default::default()
    };
    assistants.update(&asst_id, modify_request).await?;
    Ok(())
}
// region: Thread
pub async fn create_thread(client: &Client<OpenAIConfig>) -> Result<ThreadId> {
    let threads = client.threads();
    let thread = threads
        .create(CreateThreadRequest {
            ..Default::default()
        })
        .await?;
    Ok(thread.id.into())
}
pub async fn get_thread(
    client: &Client<OpenAIConfig>,
    thread_id: &ThreadId,
) -> Result<ThreadObject> {
    let threads = client.threads();
    let thread = threads.retrieve(thread_id).await?;

    Ok(thread)
}
pub async fn run_thread_msg(
    client: &Client<OpenAIConfig>,
    asst_id: &AsstId,
    thread_id: &ThreadId,
    msg: &str,
) -> Result<String> {
    let msg = user_msg(msg);
    //-- Attach msg to the thread
    let _msg_obj = client.threads().messages(thread_id).create(msg).await?;

    //-- Attach msg to the thread

    //-- create a run for the thread
    let run_request = CreateRunRequest {
        assistant_id: asst_id.to_string(),
        ..Default::default()
    };
    let run = client.threads().runs(thread_id).create(run_request).await?;
    //-- create a run for the thread

    // -- loop to get the result
    let term = Term::stdout();
    loop {
        term.write_str(">")?;
        let run = client.threads().runs(thread_id).retrieve(&run.id).await?;
        term.write_str("< ")?;
        match run.status {
            RunStatus::Completed => {
                term.write_str("\n")?;
                return get_first_thread_msg_content(client, thread_id).await;
            }
            RunStatus::Queued | RunStatus::InProgress => (),
            other => {
                term.write_str("\n")?;
                return Err(format!("ERROR WHILE RUNNING:{:?}", other).into());
            }
        }
        sleep(Duration::from_millis(POLLING_DURATION.into())).await;
    }
    // -- loop to get the result
}

pub async fn get_first_thread_msg_content(
    client: &Client<OpenAIConfig>,
    thread_id: &ThreadId,
) -> Result<String> {
    let message = client
        .threads()
        .messages(thread_id)
        .list(&[("limit", "1")])
        .await?;
    let msg = message
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "no message found".to_string())?;
    let text_content = get_text_content(msg)?;
    Ok(text_content)
}
// endregion: Thread
