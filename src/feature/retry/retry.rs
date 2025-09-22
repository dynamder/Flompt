use std::marker::PhantomData;
use std::pin::Pin;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use serde::Deserialize;
use crate::feature::retry::executable::PromptRetryExecutableWithModel;
use crate::feature::async_openai::prompt_result::PromptExecutableError;
use crate::feature::retry::result::{RetryableExecuteError, RetryablePromptResult};
use crate::prelude::Context;

pub struct RetryStrategy;
impl RetryStrategy {
    pub async fn default_retry<C, S>(retryable_error: RetryableExecuteError<'_, C, S>, retry_times: usize) -> Result<Option<S>, PromptExecutableError>
    where
        C: Context + Send + Sync + 'static,
        S: for<'de> Deserialize<'de> + Send + Sync + 'static
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
                    let model_list_size = origin.model_count();
                    let (res, should_retry) = default_process_openai_error(e, origin, context, client, model_selected, model_list_size).await;
                    if !should_retry && res.is_err() {
                        return Err(res.unwrap_err().error);
                    }
                    res
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
        Err(PromptExecutableError::RetryFail(
            Box::new(error_retry.error)
        ))
    }
}
async fn default_process_openai_error<'a, C, S>(
    error: OpenAIError,
    origin: PromptRetryExecutableWithModel<'a, C, S>,
    context: &'a mut C,
    client: &'a Client<OpenAIConfig>,
    model_selected: Option<usize>,
    model_list_size: usize
) -> (RetryablePromptResult<'a, C, S>, bool) //bool表示是否还应继续重试
where
    C: Context + Send + Sync + 'static,
    S: for<'de> Deserialize<'de> + Send + Sync + 'static
{
    match error {
        OpenAIError::Reqwest(_) | OpenAIError::JSONDeserialize(_) | OpenAIError::StreamError(_) => {
            (origin.execute_with_retry(context, client, model_selected).await,true)
        },
        OpenAIError::ApiError(e) => {
            let model_selected_u = model_selected.unwrap_or(0);
            if let Some(code) = &e.code {
                if code.contains("429") {
                    if model_list_size > 1 {
                        (
                            origin.execute_with_retry(context, client, Some((model_selected_u + 1usize) % model_list_size)).await,
                            true
                        )
                    }else{
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        (origin.execute_with_retry(context, client, model_selected).await, true)
                    }
                }else if code.contains("500") || code.contains("503") {
                    tokio::time::sleep(std::time::Duration::from_secs(40)).await;
                    (origin.execute_with_retry(context, client, model_selected).await, true)
                }else {
                    return (RetryablePromptResult::err(
                        RetryableExecuteError::new(
                            OpenAIError::ApiError(e).into(),
                            origin,
                            context,
                            client,
                            model_selected
                        )
                    ), false)
                }
            }else {
                return (RetryablePromptResult::err(
                    RetryableExecuteError::new(
                        OpenAIError::ApiError(e).into(),
                        origin,
                        context,
                        client,
                        model_selected
                    )
                ), false)
            }
        },
        _ => {
            return (RetryablePromptResult::err(
                RetryableExecuteError::new(
                    error.into(),
                    origin,
                    context,
                    client,
                    model_selected
                )
            ), false)
        }
    }
}