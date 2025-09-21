use std::marker::PhantomData;
use serde::Deserialize;
use crate::prelude::PromptExecutableError;

pub struct SendPromptResult<'a, S>(Result<Option<S>, PromptExecutableError>, PhantomData<&'a ()>)
where
    S: Deserialize<'a> + Send + Sync + 'a;

impl<'a, S> SendPromptResult<'a, S>
where
    S: Deserialize<'a> + Send + Sync + 'a
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

impl<'a, S> From<Option<S>> for SendPromptResult<'a, S>
where
    S: Deserialize<'a> + Send + Sync + 'a
{
    fn from(value: Option<S>) -> Self {
        Self(Ok(value), PhantomData)
    }
}

impl<'a, S> SendPromptResult<'a, S>
where

    S: Deserialize<'a> + Send + Sync + 'a
{
    pub fn ok(value: impl Into<Option<S>>) -> Self {
        Self(Ok(value.into()), PhantomData)
    }
    pub fn err(value: impl Into<PromptExecutableError>) -> Self {
        Self(Err(value.into()), PhantomData)
    }
}