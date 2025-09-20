use async_openai::{Client, Models};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse};
use serde::Deserialize;
use crate::feature::async_openai::prompt_result::PromptExecutableError::{InvalidModelSelection, ModelNotSet};
use crate::feature::async_openai::prompt_result::{PromptResult, RetryablePromptResult};
use crate::feature::async_openai::send_control::SendPromptVariant;
use crate::prelude::{Context, Prompt, PromptVariant};

pub struct PromptExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: PromptVariant<'a, C>,
    processor: Box<dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + 'a>,
}
pub struct PromptRetryExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: SendPromptVariant<'a, C>,
    processor: Box<dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + Send + Sync + 'a>,
}
impl<'a, C> PromptVariant<'a, C>
where
    C: Context
{
    pub fn to_executable<S, F>(self, processor: F) -> PromptExecutable<'a, C, S>
    where
        S: Deserialize<'a>,
        F: Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + 'a
    {
        PromptExecutable {
            prompt: self,
            processor: Box::new(processor),
        }
    }
}
impl<'a, C> SendPromptVariant<'a, C>
where
    C: Context
{
    pub fn to_executable<S, F>(self, processor: F) -> PromptRetryExecutable<'a, C, S>
    where
        S: Deserialize<'a>,
        F: Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + Send + Sync + 'a
    {
        PromptRetryExecutable {
            prompt: self,
            processor: Box::new(processor),
        }
    }
}
pub struct PromptExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: PromptExecutable<'a, C, S>,
    models: Vec<&'a str>,
}
pub struct PromptRetryExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: PromptRetryExecutable<'a, C, S>,
    models: Vec<&'a str>,
}
impl<'a, C, S> PromptExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    pub fn get_processor(&self) -> &dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> {
        &self.processor
    }
    pub fn models(self, models: Vec<&'a str>) -> PromptExecutableWithModel<'a, C, S> {
        PromptExecutableWithModel {
            prompt: self,
            models,
        }
    }
}
impl<'a, C, S> PromptRetryExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    pub fn get_processor(&self) -> &(dyn (Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error>) + Send + Sync) {
        &self.processor
    }
    pub fn models(self, models: Vec<&'a str>) -> PromptRetryExecutableWithModel<'a, C, S> {
        PromptRetryExecutableWithModel {
            prompt: self,
            models,
        }
    }
}
impl<'a, C, S> PromptExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    pub async fn execute(self, context: &'a mut C,client: &'a Client<OpenAIConfig>, select_model: Option<usize>) -> PromptResult<'a, S> {
        if self.models.is_empty() {
            return PromptResult::err(ModelNotSet);
        }
        let selected_model = select_model.unwrap_or(0);
        let model = if let Some(&model) = self.models.get(selected_model) {
            model
        } else {
            return PromptResult::err(InvalidModelSelection(selected_model))
        };
        let prompt_str = match self.prompt.prompt.prompt_str(context) {
            Ok(Some(prompt_str)) => prompt_str,
            Ok(None) => return PromptResult::ok(None),
            Err(e) => return PromptResult::err(e),
        };

        let user_message =  match ChatCompletionRequestUserMessageArgs::default()
            .content(&*prompt_str)
            .build() {
            Ok(user_message) => user_message,
            Err(e) => return PromptResult::err(e),
        };

        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                user_message.into()
            ])
            .model(model)
            .build();
        let request = match request {
            Ok(request) => request,
            Err(e) => return PromptResult::err(e),
        };

        let response = client.chat()
            .create(request)
            .await;
        let response = match response {
            Ok(response) => response,
            Err(e) => return PromptResult::err(e),
        };
        let deserialized_result = self.prompt.get_processor()(response, context);
        match deserialized_result {
            Ok(result) => PromptResult::ok(result),
            Err(e) => PromptResult::err(e),
        }
    }
}
impl<'a, C, S> PromptRetryExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    pub async fn execute_with_retry(self, context: &'a mut C,client: &'a Client<OpenAIConfig>, select_model: Option<usize>) -> RetryablePromptResult<'a, C, S>
    where
        C: Send + Sync + 'a,
        S: Send + Sync + 'a
    {
        if self.models.is_empty() {
            return RetryablePromptResult::err((self, ModelNotSet, context, client, select_model));
        }
        let selected_model = select_model.unwrap_or(0);
        let model = if let Some(&model) = self.models.get(selected_model) {
            model
        } else {
            return RetryablePromptResult::err((self, InvalidModelSelection(selected_model), context, client, select_model))
        };
        let prompt_str = match self.prompt.prompt.prompt_str(context) {
            Ok(Some(prompt_str)) => prompt_str,
            Ok(None) => return RetryablePromptResult::ok(None),
            Err(e) => return RetryablePromptResult::err((self, e.into(), context, client, select_model)),
        };

        let user_message =  match ChatCompletionRequestUserMessageArgs::default()
            .content(&*prompt_str)
            .build() {
            Ok(user_message) => user_message,
            Err(e) => return RetryablePromptResult::err((self, e.into(), context, client, select_model)),
        };

        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                user_message.into()
            ])
            .model(model)
            .build();
        let request = match request {
            Ok(request) => request,
            Err(e) => return RetryablePromptResult::err((self, e.into(), context, client, select_model)),
        };

        let response = client.chat()
            .create(request)
            .await;
        let response = match response {
            Ok(response) => response,
            Err(e) => return RetryablePromptResult::err((self, e.into(), context, client, select_model)),
        };
        let deserialized_result = self.prompt.get_processor()(response, context);
        match deserialized_result {
            Ok(result) => RetryablePromptResult::ok(result),
            Err(e) => RetryablePromptResult::err((self, e.into(), context, client, select_model)),
        }
    }
}