use serde::Deserialize;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateChatCompletionResponse};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use crate::prelude::{Context, Prompt, RetryablePromptResult, SendPromptVariant};
use crate::prelude::PromptExecutableError::{InvalidModelSelection, ModelNotSet};

pub struct PromptRetryExecutable<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: &'a SendPromptVariant<'a, C>,
    processor: Box<dyn Fn(CreateChatCompletionResponse, &mut C) -> Result<Option<S>, serde_json::Error> + Send + Sync + 'a>,
}

pub struct PromptRetryExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    prompt: PromptRetryExecutable<'a, C, S>,
    models: Vec<&'a str>,
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

impl<'a, C, S> PromptRetryExecutableWithModel<'a, C, S>
where
    C: Context,
    S: Deserialize<'a>
{
    pub fn model_count(&self) -> usize {
        self.models.len()
    }
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

impl<'a, C> SendPromptVariant<'a, C>
where
    C: Context
{
    pub fn to_retry_executable<S, F>(&'a self, processor: F) -> PromptRetryExecutable<'a, C, S>
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