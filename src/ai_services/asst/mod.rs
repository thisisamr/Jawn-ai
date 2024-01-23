use crate::Result;
use async_openai::types::{AssistantObject, CreateAssistantRequest, ModifyAssistantRequest};
use async_openai::{config::OpenAIConfig, types::AssistantToolsRetrieval, Client};
use derive_more::{Deref, Display, From};

// region: constants
const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];
// endregion: Constants

// region:Types 
pub struct CreateConfig {
    pub name: String,
    pub model: String,
}

#[derive(Debug, From, Display, Deref)]
pub struct AsstId(String);

#[derive(Debug, From, Display, Deref)]
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
pub async fn delete_assistant(client:&Client<OpenAIConfig>,asst_id:&AsstId)->Result<()>{
   let assistants =client.assistants();

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
        println!("assistant {} deleted",config.name);
    }
    if let Some(assistant_id) = assistant_id {
        println!("assistant {} loaded", &config.name);
        Ok(assistant_id)
    } else {
        let asst_name = config.name.clone();
        let assistant = create_assistant(client, config).await?;
        Ok(assistant)
    }
}

pub async fn upload_instructions(client:&Client<OpenAIConfig>,asst_id:&AsstId,instructions:String)->Result<()>{
    let assistants = client.assistants();
    let modify_request = ModifyAssistantRequest{
        instructions:Some(instructions),..Default::default()
    };
    assistants.update(&asst_id, modify_request).await?;
    Ok(())
}