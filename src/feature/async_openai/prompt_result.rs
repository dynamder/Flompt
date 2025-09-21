use std::marker::PhantomData;
use async_openai::error::OpenAIError;
use serde::Deserialize;
use thiserror::Error;
use crate::prelude::PromptError;

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
    #[cfg(feature = "retry")]
    #[error("Fail After Retry, retry error: {0}")] //TODO: better error msg
    RetryFail(Box<PromptExecutableError>),
    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error)
}

pub struct PromptResult<'a, S>(Result<Option<S>, PromptExecutableError>, PhantomData<&'a ()>)
where
    S: Deserialize<'a>;
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


