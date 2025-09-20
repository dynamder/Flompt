use std::marker::PhantomData;
use std::pin::Pin;
use serde::Deserialize;
use crate::feature::async_openai::prompt_result::{PromptExecutableError, RetryableExecuteError};
use crate::prelude::Context;

pub struct RetryStrategy<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a,
{
    retry_times: usize,
    retry_func: Box<dyn Fn(RetryableExecuteError<'a, C, S>) -> Pin<Box<dyn Future<Output = Result<Option<S>, PromptExecutableError>> + Send + '_>> + Send + Sync + 'static>,
    _marker: PhantomData<&'a (C, S)>,
}
impl<'a, C, S> RetryStrategy<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a,
{
    pub fn new<F, Fut>(
        retry_times: usize,
        retry_func: F
    ) -> Self
    where
        F: Fn(RetryableExecuteError<'a, C, S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<S>, PromptExecutableError>> + Send + 'a
    {
        let boxed_func = move |error| {
            let fut = retry_func(error);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Result<Option<S>, PromptExecutableError>> + Send + 'a>>
        };
        Self {
            retry_times,
            retry_func: Box::new(boxed_func),
            _marker: PhantomData,
        }
    }
    pub fn default_strategy(retry_times: usize) -> Self {
        Self::new(
            retry_times,
            move |error: RetryableExecuteError<'a, C, S>| {
                Self::default_retry_func(error, retry_times)
            }
        )
    }
    async fn default_retry_func(retryable_error: RetryableExecuteError<'a, C, S>, retry_times: usize) -> Result<Option<S>, PromptExecutableError>
    where
        C: Send + Sync + 'a,
        S: Send + Sync + 'a
    {
        let mut error_retry = retryable_error;

        for _ in 0..retry_times {
            let RetryableExecuteError {
                origin,
                error,
                context,
                client,
                model_selected
            } = error_retry;
            let retry_result = match error {
                PromptExecutableError::ModelNotSet => return Err(PromptExecutableError::ModelNotSet),
                PromptExecutableError::InvalidModelSelection(_) => {
                    origin.execute_with_retry(context, client, None).await
                }
                PromptExecutableError::FailBuildingPrompt(e) => return Err(PromptExecutableError::FailBuildingPrompt(e)),
                PromptExecutableError::OpenAI(e) => {
                    todo!()
                }
                PromptExecutableError::RetryFail(_) => unreachable!("We are retrying here, and only here"),
                PromptExecutableError::Deserialize(_) => {
                    origin.execute_with_retry(context, client, None).await
                }
            };
            if retry_result.is_err() {
                error_retry = retry_result.unwrap_err();
            } else {
                return Ok(retry_result.unwrap());
            }
        }
        Err(error_retry.error)
    }
}
impl<'a, C, S> Default for RetryStrategy<'a, C, S>
where
    C: Context + Send + Sync + 'a,
    S: Deserialize<'a> + Send + Sync + 'a,
{
    fn default() -> Self {
        Self::default_strategy(3)
    }
}