use std::marker::PhantomData;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::CreateChatCompletionResponse;
use serde::Deserialize;
use thiserror::Error;
use crate::feature::async_openai::executable::{PromptExecutableWithModel, PromptRetryExecutableWithModel};
use crate::feature::async_openai::retry::RetryStrategy;
use crate::prelude::{Context, PromptError};

#[derive(Debug, Error)]
pub enum PromptExecutableError
{
    #[error("model not set")]
    ModelNotSet,
    #[error("invalid model selection: {0}")]
    InvalidModelSelection(usize),
    #[error("fail building prompt")]
    FailBuildingPrompt(#[from] PromptError),
    #[error("openai api error: {0}")]
    OpenAI(#[from] OpenAIError),
    #[error("Fail After Retry, retry error: {0}")] //TODO: better error msg
    RetryFail(Box<PromptExecutableError>),
    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error)
}
pub struct RetryableExecuteError<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    pub error: PromptExecutableError,
    pub origin: PromptRetryExecutableWithModel<'a, C, S>,
    pub context: &'a mut C,
    pub client: &'a Client<OpenAIConfig>,
    pub model_selected: Option<usize>,
}
impl<'a, C, S> RetryableExecuteError<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    pub fn new(
        error: PromptExecutableError,
        origin: PromptRetryExecutableWithModel<'a, C, S>,
        context: &'a mut C,
        client: &'a Client<OpenAIConfig>,
        model_selected: Option<usize>
    ) -> Self {
        Self { error, origin, context, client, model_selected}
    }
    pub async fn retry(self, retry_strategy: RetryStrategy<'a, C, S>) -> Result<Option<S>, PromptExecutableError> {
        todo!()
    }
}
impl<'a, C, S> From<(PromptRetryExecutableWithModel<'a, C, S>, PromptExecutableError, &'a mut C, &'a Client<OpenAIConfig>, Option<usize>)> for RetryableExecuteError<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    fn from(value: (PromptRetryExecutableWithModel<'a, C, S>, PromptExecutableError, &'a mut C, &'a Client<OpenAIConfig>, Option<usize>)) -> Self {
        Self::new(value.1, value.0, value.2, value.3, value.4)
    }
}
impl<'a, C, S> From<(PromptRetryExecutableWithModel<'a, C, S>, PromptExecutableError, &'a mut C, &'a Client<OpenAIConfig>, Option<usize>)> for RetryablePromptResult<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    fn from(value: (PromptRetryExecutableWithModel<'a, C, S>, PromptExecutableError, &'a mut C, &'a Client<OpenAIConfig>, Option<usize>)) -> Self {
        Self::err(value)
    }
}

pub struct PromptResult<'a, S>(Result<Option<S>, PromptExecutableError>, PhantomData<&'a ()>)
where
    S: Deserialize<'a>;
pub struct RetryablePromptResult<'a, C, S>(Result<Option<S>, RetryableExecuteError<'a, C, S>>)
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a;
impl<'a, C, S> RetryablePromptResult<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a,
{
    pub async fn retry<F, Fut>(self, retry_strategy: RetryStrategy<'a, C, S>) -> Result<Option<S>, PromptExecutableError>
    where
        F: Fn(RetryableExecuteError<'a, C, S>) -> Fut,
        Fut: Future<Output = Result<Option<S>, PromptExecutableError>>
    {
        match self.0 {
            Ok(s) => Ok(s),
            Err(e) => {
                e.retry(retry_strategy).await
            }
        }
    }
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }
    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }
    pub fn unwrap(self) -> Option<S> {
        match self.0 {
            Ok(s) => s,
            Err(e) => panic!("RetryablePromptResult is err: {}", e.error)
        }
    }
    pub fn unwrap_err(self) -> RetryableExecuteError<'a, C, S> {
        match self.0 {
            Ok(s) => panic!("RetryablePromptResult is ok."),
            Err(e) => e
        }
    }
}
impl<'a, S> PromptResult<'a, S>
where
    S: Deserialize<'a>
{
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }
    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }
    pub fn unwrap(self) -> Option<S> {
        match self.0 {
            Ok(s) => s,
            Err(e) => panic!("PromptResult is err: {e}.")
        }
    }
    pub fn unwrap_err(self) -> PromptExecutableError {
        match self.0 {
            Ok(s) => panic!("PromptResult is ok."),
            Err(e) => e
        }
    }
}
impl<'a, S> From<Option<S>> for PromptResult<'a, S>
where
    S: Deserialize<'a>
{
    fn from(value: Option<S>) -> Self {
        Self(Ok(value), PhantomData)
    }
}
impl<'a, S> PromptResult<'a, S>
where

    S: Deserialize<'a>
{
    pub fn ok(value: impl Into<Option<S>>) -> Self {
        Self(Ok(value.into()), PhantomData)
    }
    pub fn err(value: impl Into<PromptExecutableError>) -> Self {
        Self(Err(value.into()), PhantomData)
    }
}
impl<'a, C, S> RetryablePromptResult<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    pub fn ok(value: impl Into<Option<S>>) -> Self {
        Self(Ok(value.into()))
    }
    pub fn err(value: impl Into<RetryableExecuteError<'a, C, S>>) -> Self {
        Self(Err(value.into()))
    }
}
impl<'a, C, S> From<RetryableExecuteError<'a, C, S>> for RetryablePromptResult<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a
{
    fn from(value: RetryableExecuteError<'a, C, S>) -> Self {
        Self(Err(value))
    }
}