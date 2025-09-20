pub mod prompt_result;
pub mod executable;
pub mod retry;
pub mod send_control;

use async_openai::Client;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::CreateChatCompletionResponse;
use serde::Deserialize;
use crate::feature::async_openai::prompt_result::PromptResult;
use crate::prelude::{Context, PromptVariant};

