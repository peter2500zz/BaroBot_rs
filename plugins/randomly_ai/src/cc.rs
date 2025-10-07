
use anyhow::Result;
use async_openai::{
    config::OpenAIConfig, types::{
        ChatCompletionRequestAssistantMessage, 
        ChatCompletionRequestAssistantMessageContent, 
        ChatCompletionRequestMessage, 
        ChatCompletionRequestSystemMessageArgs, 
        ChatCompletionRequestUserMessageArgs, 
        CreateChatCompletionRequestArgs
    }, Client
};

#[derive(Debug, Clone)]
pub struct ChatCore {
    client: Client<OpenAIConfig>,
    model: String,
    max_token: u32,
    prompt: String,

    pub messages: Vec<ChatCompletionRequestMessage>
}

impl ChatCore {
    pub fn new(api_base: &str, api_key: &str, model: &str, max_token: u32, prompt: &str) -> Result<Self> {
        let cfg = OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(api_key);

        let client = Client::with_config(cfg);

        let mut cc = Self {
            client,
            model: model.to_string(),
            max_token,
            prompt: prompt.to_string(),
            messages: Vec::new()
        };

        cc.clear_memory()?;

        Ok(cc)
    }

    pub fn clear_memory(&mut self) -> Result<()> {
        self.messages.clear();

        Ok(())
    }

    pub fn add_user_text(&mut self, text: &str, name: &str) -> Result<()> {
        self.messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .name(name)
                .content(text)
                .build()?
                .into(),
        );

        Ok(())
    }

    pub async fn chat(&mut self, text: &str, name: &str, temperature: f32) -> Result<String> {
        let mut temp_msg = self.messages.clone();

        temp_msg.insert(0, ChatCompletionRequestSystemMessageArgs::default()
            .content(self.prompt.clone())
            .build()?
            .into());

        temp_msg.push(
            ChatCompletionRequestUserMessageArgs::default()
                .name(name)
                .content(text)
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.max_token)
            .model(&self.model)
            .temperature(temperature)
            // .response_format(ResponseFormat::JsonObject)
            .messages(temp_msg.clone())
            .build()?;

        let response = self.client.chat().create(request).await?;

        temp_msg.push(
            ChatCompletionRequestAssistantMessage {
                content: response.choices[0].message.content.clone().and_then(|v| Some(ChatCompletionRequestAssistantMessageContent::from(v))),
                refusal: response.choices[0].message.refusal.clone(),
                ..Default::default()
            }.into()
        );

        self.messages = temp_msg;

        Ok(response.choices[0].message.content.clone().unwrap_or_default())
    }
}
