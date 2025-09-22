use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse};
use serde::Deserialize;
use crate::feature::async_openai::prompt_result::PromptExecutableError::{InvalidModelSelection, ModelNotSet};
use crate::feature::async_openai::prompt_result::PromptResult;
#[cfg(feature = "send")]
use crate::feature::send::control::SendPromptVariant;
#[cfg(feature = "send")]
use crate::feature::send::result::SendPromptResult;
use crate::prelude::{Context, Prompt, PromptVariant};

pub struct PromptExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: &'a PromptVariant<'a, C>,
    processor: Box<dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + 'a>,
}
#[cfg(feature = "send")]
pub struct SendPromptExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a> + Send + Sync + 'a
{
    prompt: &'a SendPromptVariant<'a, C>,
    processor: Box<dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + 'a>,
}
impl<'a, C> PromptVariant<'a, C>
where
    C: Context
{
    pub fn to_executable<S, F>(&'a self, processor: F) -> PromptExecutable<'a, C, S>
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
#[cfg(feature = "send")]
impl<'a, C> SendPromptVariant<'a, C>
where
    C: Context
{
    pub fn to_executable<S, F>(&'a self, processor: F) -> SendPromptExecutable<'a, C, S>
    where
        S: Deserialize<'a> + Send + Sync + 'a,
        F: Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + Send + Sync + 'a
    {
        SendPromptExecutable {
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
#[cfg(feature = "send")]
pub struct SendPromptExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a> + Send + Sync + 'a
{
   prompt: SendPromptExecutable<'a, C, S>,
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
#[cfg(feature = "send")]
impl<'a, C, S> SendPromptExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a> + Send + Sync + 'a
{
    pub fn get_processor(&self) -> &dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> {
        &self.processor
    }
    pub fn models(self, models: Vec<&'a str>) -> SendPromptExecutableWithModel<'a, C, S> {
        SendPromptExecutableWithModel {
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
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
    pub fn inner_variant(&self) -> &PromptVariant<'a, C> {
        self.prompt.prompt
    }
    pub fn models(&self) -> &Vec<&'a str> {
        &self.models
    }
 
    pub async fn execute(&self, context: &'a mut C,client: &'a Client<OpenAIConfig>, select_model: Option<usize>) -> PromptResult<'a, S> {
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
#[cfg(feature = "send")]
impl<'a, C, S> SendPromptExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a> + Send + Sync + 'a
{
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
    pub fn inner_variant(&self) -> &SendPromptVariant<'a, C> {
        self.prompt.prompt
    }
    pub fn models(&self) -> &Vec<&'a str> {
        &self.models
    }
    pub async fn execute(&self, context: &'a mut C,client: &'a Client<OpenAIConfig>, select_model: Option<usize>) -> SendPromptResult<'a, S> {
        if self.models.is_empty() {
            return SendPromptResult::err(ModelNotSet);
        }
        let selected_model = select_model.unwrap_or(0);
        let model = if let Some(&model) = self.models.get(selected_model) {
            model
        } else {
            return SendPromptResult::err(InvalidModelSelection(selected_model))
        };
        let prompt_str = match self.prompt.prompt.prompt_str(context) {
            Ok(Some(prompt_str)) => prompt_str,
            Ok(None) => return SendPromptResult::ok(None),
            Err(e) => return SendPromptResult::err(e),
        };

        let user_message =  match ChatCompletionRequestUserMessageArgs::default()
            .content(&*prompt_str)
            .build() {
            Ok(user_message) => user_message,
            Err(e) => return SendPromptResult::err(e),
        };

        let request = CreateChatCompletionRequestArgs::default()
            .messages(vec![
                user_message.into()
            ])
            .model(model)
            .build();
        let request = match request {
            Ok(request) => request,
            Err(e) => return SendPromptResult::err(e),
        };

        let response = client.chat()
            .create(request)
            .await;
        let response = match response {
            Ok(response) => response,
            Err(e) => return SendPromptResult::err(e),
        };
        let deserialized_result = self.prompt.get_processor()(response, context);
        match deserialized_result {
            Ok(result) => SendPromptResult::ok(result),
            Err(e) => SendPromptResult::err(e),
        }
    }
}