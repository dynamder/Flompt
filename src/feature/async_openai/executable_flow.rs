use std::iter::Peekable;
use serde::Deserialize;
use crate::feature::async_openai::executable::{PromptExecutableWithModel};
#[cfg(feature = "send")]
use crate::feature::async_openai::executable::{SendPromptExecutableWithModel};
use crate::prelude::{IfPrompt, LoopPrompt, PromptVariant};
use crate::prompt::context::Context;
use crate::prompt::error::{IfPromptBuilderError, LoopPromptBuilderError};

#[derive(Default)]
pub struct ExecutablePromptChain<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    prompts: Vec<ExecutablePromptVariant<'a, C, S>>
}
impl<'a, C, S> ExecutablePromptChain<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    pub fn new() -> Self {
        ExecutablePromptChain {
            prompts: Vec::new()
        }
    }
    pub fn push(&mut self, prompt: impl Into<ExecutablePromptVariant<'a, C, S>>) {
        self.prompts.push(prompt.into());
    }
    pub fn flow(&'a self) -> ExecutableFlow<'a, C, S> {
        ExecutableFlow {
            prompts: self.prompts.iter().peekable()
        }
    }
}

pub struct ExecutableFlow<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    prompts: Peekable<std::slice::Iter<'a, ExecutablePromptVariant<'a, C, S>>>
}
impl<'a, C, S> ExecutableFlow<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    pub fn next_with(&mut self, context: &C) -> Option<&PromptExecutableWithModel<C, S>> {
        loop {
            let mut final_break = false;
            let res = self.prompts.peek().and_then(|&prompt| {
                let mut cur_prompt = prompt;
                loop {
                    match cur_prompt {
                        ExecutablePromptVariant::Direct(_)=> {
                            final_break = true;
                            break Some(cur_prompt)
                        },
                        ExecutablePromptVariant::If(if_prompt) => {
                            if if_prompt.get_condition()(context) {
                                cur_prompt = (if_prompt.get_then())
                            } else {
                                cur_prompt = if let Some(otherwise) = if_prompt.get_otherwise() {
                                    otherwise
                                } else {
                                    break None
                                }
                            }
                        },
                        ExecutablePromptVariant::Loop(loop_prompt) => {
                            if loop_prompt.get_condition()(context) {
                                cur_prompt = loop_prompt.get_prompt();
                            } else {
                                break None
                            }
                        }
                    }
                }
            });
            if final_break {
                self.prompts.next();
                break match res.unwrap() { //NOTE: res must be Some if final_break is true
                    ExecutablePromptVariant::Direct(prompt) => Some(prompt),
                    _ => unreachable!() //NOTE: Variant must be Direct if final_break is true
                }
            }
            if res.is_none() && self.prompts.next().is_none() {
                break None
            }
        }
    }
}
pub struct ExecutableIfPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    condition: Box<dyn Fn(&C) -> bool>,
    then: ExecutablePromptVariant<'a, C, S>,
    otherwise: Option<ExecutablePromptVariant<'a, C, S>>
}
impl<'a, C, S> ExecutableIfPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    pub fn new(condition: impl Fn(&C) -> bool + 'static, then: ExecutablePromptVariant<'a, C, S>, otherwise: Option<ExecutablePromptVariant<'a, C, S>>) -> Self {
        ExecutableIfPrompt {
            condition: Box::new(condition),
            then,
            otherwise,
        }
    }
    pub fn get_then(&self) -> &ExecutablePromptVariant<'a, C, S> {
        &self.then
    }
    pub fn get_otherwise(&self) -> Option<&ExecutablePromptVariant<'a, C, S>> {
        self.otherwise.as_ref()
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
}
pub struct ExecutableLoopPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    condition: Box<dyn Fn(&C) -> bool>,
    prompt: ExecutablePromptVariant<'a, C, S>
}
impl<'a, C, S> ExecutableLoopPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    pub fn new(condition: impl Fn(&C) -> bool + 'static, prompt: ExecutablePromptVariant<'a, C, S>) -> Self {
        ExecutableLoopPrompt {
            condition: Box::new(condition),
            prompt,
        }
    }
    pub fn get_prompt(&self) -> &ExecutablePromptVariant<'a, C, S> {
        &self.prompt
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
}
pub enum ExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    Direct(PromptExecutableWithModel<'a, C, S>), //Naive and Template
    If(Box<ExecutableIfPrompt<'a, C, S>>),
    Loop(Box<ExecutableLoopPrompt<'a, C, S>>),
}

impl<'a, C, S> From<ExecutableIfPrompt<'a, C, S>> for ExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    fn from(if_prompt: ExecutableIfPrompt<'a, C, S>) -> Self {
        ExecutablePromptVariant::If(Box::new(if_prompt))
    }
}
impl<'a, C, S> From<ExecutableLoopPrompt<'a, C, S>> for ExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    fn from(loop_prompt: ExecutableLoopPrompt<'a, C, S>) -> Self {
        ExecutablePromptVariant::Loop(Box::new(loop_prompt))
    }
}
impl<'a, C, S> From<PromptExecutableWithModel<'a, C, S>> for ExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de>
{
    fn from(prompt: PromptExecutableWithModel<'a, C, S>) -> Self {
        ExecutablePromptVariant::Direct(prompt)
    }
}


#[cfg(feature = "send")]
#[derive(Default)]
pub struct SendExecutablePromptChain<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    prompts: Vec<SendExecutablePromptVariant<'a, C, S>>
}
#[cfg(feature = "send")]
impl<'a, C, S> SendExecutablePromptChain<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    pub fn new() -> Self {
        SendExecutablePromptChain {
            prompts: Vec::new()
        }
    }
    pub fn push(&mut self, prompt: impl Into<SendExecutablePromptVariant<'a, C, S>>) {
        self.prompts.push(prompt.into());
    }
    pub fn flow(&'a self) -> SendExecutableFlow<'a, C, S> {
        SendExecutableFlow {
            prompts: self.prompts.iter().peekable()
        }
    }
}
#[cfg(feature = "send")]
pub struct SendExecutableFlow<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    prompts: Peekable<std::slice::Iter<'a, SendExecutablePromptVariant<'a, C, S>>>
}
#[cfg(feature = "send")]
impl<'a, C, S> SendExecutableFlow<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    pub fn next_with(&mut self, context: &C) -> Option<&SendPromptExecutableWithModel<C, S>> {
        loop {
            let mut final_break = false;
            let res = self.prompts.peek().and_then(|&prompt| {
                let mut cur_prompt = prompt;
                loop {
                    match cur_prompt {
                        SendExecutablePromptVariant::Direct(_)=> {
                            final_break = true;
                            break Some(cur_prompt)
                        },
                        SendExecutablePromptVariant::If(if_prompt) => {
                            if if_prompt.get_condition()(context) {
                                cur_prompt = (if_prompt.get_then())
                            } else {
                                cur_prompt = if let Some(otherwise) = if_prompt.get_otherwise() {
                                    otherwise
                                } else {
                                    break None
                                }
                            }
                        },
                        SendExecutablePromptVariant::Loop(loop_prompt) => {
                            if loop_prompt.get_condition()(context) {
                                cur_prompt = loop_prompt.get_prompt();
                            } else {
                                break None
                            }
                        }
                    }
                }
            });
            if final_break {
                self.prompts.next();
                break match res.unwrap() { //NOTE: res must be Some if final_break is true
                    SendExecutablePromptVariant::Direct(prompt) => Some(prompt),
                    _ => unreachable!() //NOTE: Variant must be Direct if final_break is true
                }
            }
            if res.is_none() && self.prompts.next().is_none() {
                break None
            }
        }
    }
}
#[cfg(feature = "send")]
pub struct SendExecutableIfPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    condition: Box<dyn Fn(&C) -> bool>,
    then: SendExecutablePromptVariant<'a, C, S>,
    otherwise: Option<SendExecutablePromptVariant<'a, C, S>>
}
#[cfg(feature = "send")]
impl<'a, C, S> SendExecutableIfPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    pub fn new(condition: impl Fn(&C) -> bool + Send + Sync + 'static, then: SendExecutablePromptVariant<'a, C, S>, otherwise: Option<SendExecutablePromptVariant<'a, C, S>>) -> Self {
        SendExecutableIfPrompt {
            condition: Box::new(condition),
            then,
            otherwise,
        }
    }
    pub fn get_then(&self) -> &SendExecutablePromptVariant<'a, C, S> {
        &self.then
    }
    pub fn get_otherwise(&self) -> Option<&SendExecutablePromptVariant<'a, C, S>> {
        self.otherwise.as_ref()
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
}
#[cfg(feature = "send")]
pub struct SendExecutableLoopPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> +  Send + Sync
{
    condition: Box<dyn Fn(&C) -> bool>,
    prompt: SendExecutablePromptVariant<'a, C, S>
}
#[cfg(feature = "send")]
impl<'a, C, S> SendExecutableLoopPrompt<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync
{
    pub fn new(condition: impl Fn(&C) -> bool + Send + Sync + 'static, prompt: SendExecutablePromptVariant<'a, C, S>) -> Self {
        SendExecutableLoopPrompt {
            condition: Box::new(condition),
            prompt,
        }
    }
    pub fn get_prompt(&self) -> &SendExecutablePromptVariant<'a, C, S> {
        &self.prompt
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
}
#[cfg(feature = "send")]
pub enum SendExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync
{
    Direct(SendPromptExecutableWithModel<'a, C, S>), //Naive and Template
    If(Box<SendExecutableIfPrompt<'a, C, S>>),
    Loop(Box<SendExecutableLoopPrompt<'a, C, S>>),
}
#[cfg(feature = "send")]
impl<'a, C, S> From<SendExecutableIfPrompt<'a, C, S>> for SendExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync
{
    fn from(if_prompt: SendExecutableIfPrompt<'a, C, S>) -> Self {
        SendExecutablePromptVariant::If(Box::new(if_prompt))
    }
}
#[cfg(feature = "send")]
impl<'a, C, S> From<SendExecutableLoopPrompt<'a, C, S>> for SendExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync
{
    fn from(loop_prompt: SendExecutableLoopPrompt<'a, C, S>) -> Self {
        SendExecutablePromptVariant::Loop(Box::new(loop_prompt))
    }
}
#[cfg(feature = "send")]
impl<'a, C, S> From<SendPromptExecutableWithModel<'a, C, S>> for SendExecutablePromptVariant<'a, C, S>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync
{
    fn from(prompt: SendPromptExecutableWithModel<'a, C, S>) -> Self {
        SendExecutablePromptVariant::Direct(prompt)
    }
}


#[cfg(feature = "send")]
pub struct SendExecutableIfPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync,
    U: Fn(&C) -> bool + 'static,
{
    then: Option<SendExecutablePromptVariant<'a, C, S>>,
    otherwise: Option<SendExecutablePromptVariant<'a, C, S>>,
    condition: Option<U>,
}
#[cfg(feature = "send")]
impl<'a, C, S, U> SendExecutableIfPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync,
    U: Fn(&C) -> bool + 'static
{
    pub fn new() -> Self {
        SendExecutableIfPromptBuilder {
            then: None,
            otherwise: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<SendExecutableIfPrompt<'a, C, S>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.then.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(SendExecutableIfPrompt {
            then: self.then.unwrap(),
            otherwise: self.otherwise,
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn then(mut self, then: impl Into<SendExecutablePromptVariant<'a, C, S>>) -> Self {
        self.then = Some(then.into());
        self
    }
    pub fn otherwise(mut self, otherwise: impl Into<SendExecutablePromptVariant<'a, C, S>>) -> Self {
        self.otherwise = Some(otherwise.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}
#[cfg(feature = "send")]
#[derive(Default)]
pub struct SendExecutableLoopPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync,
    U: Fn(&C) -> bool,
{
    prompt: Option<SendExecutablePromptVariant<'a, C, S>>,
    condition: Option<U>,
}
#[cfg(feature = "send")]
impl<'a, C, S, U> SendExecutableLoopPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de> + Send + Sync,
    U: Fn(&C) -> bool + 'static,
{
    pub fn new() -> Self {
        SendExecutableLoopPromptBuilder {
            prompt: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<SendExecutableLoopPrompt<'a, C, S>, LoopPromptBuilderError> {
        if self.condition.is_none() {
            return Err(LoopPromptBuilderError::MissingCondition);
        }
        if self.prompt.is_none() {
            return Err(LoopPromptBuilderError::MissingPrompt);
        }
        Ok(SendExecutableLoopPrompt {
            prompt: self.prompt.unwrap(),
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn prompt(mut self, prompt: impl Into<SendExecutablePromptVariant<'a, C, S>>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}

pub struct ExecutableIfPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de>,
    U: Fn(&C) -> bool + 'static,
{
    then: Option<ExecutablePromptVariant<'a, C, S>>,
    otherwise: Option<ExecutablePromptVariant<'a, C, S>>,
    condition: Option<U>,
}
impl<'a, C, S, U> ExecutableIfPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de>,
    U: Fn(&C) -> bool + 'static
{
    pub fn new() -> Self {
        ExecutableIfPromptBuilder {
            then: None,
            otherwise: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<ExecutableIfPrompt<'a, C, S>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.then.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(ExecutableIfPrompt {
            then: self.then.unwrap(),
            otherwise: self.otherwise,
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn then(mut self, then: impl Into<ExecutablePromptVariant<'a, C, S>>) -> Self {
        self.then = Some(then.into());
        self
    }
    pub fn otherwise(mut self, otherwise: impl Into<ExecutablePromptVariant<'a, C, S>>) -> Self {
        self.otherwise = Some(otherwise.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}
#[derive(Default)]
pub struct ExecutableLoopPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de>,
    U: Fn(&C) -> bool,
{
    prompt: Option<ExecutablePromptVariant<'a, C, S>>,
    condition: Option<U>,
}
impl<'a, C, S, U> ExecutableLoopPromptBuilder<'a, C, S, U>
where
    C: Context,
    S: for<'de> Deserialize<'de>,
    U: Fn(&C) -> bool + 'static,
{
    pub fn new() -> Self {
        ExecutableLoopPromptBuilder {
            prompt: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<ExecutableLoopPrompt<'a, C, S>, LoopPromptBuilderError> {
        if self.condition.is_none() {
            return Err(LoopPromptBuilderError::MissingCondition);
        }
        if self.prompt.is_none() {
            return Err(LoopPromptBuilderError::MissingPrompt);
        }
        Ok(ExecutableLoopPrompt {
            prompt: self.prompt.unwrap(),
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn prompt(mut self, prompt: impl Into<ExecutablePromptVariant<'a, C, S>>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}