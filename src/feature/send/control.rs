use std::borrow::Cow;
use crate::prelude::{IfPrompt, PromptTemplate, PromptVariant};
use crate::prompt::context::Context;
use crate::prompt::error::{IfPromptBuilderError, PromptError};
use crate::prompt::naive::{Prompt};

pub struct SendIfPrompt<'a, C>
where
    C: Context + 'a
{
    then: SendPromptVariant<'a, C>,
    otherwise: Option<SendPromptVariant<'a, C>>,
    condition: Box<dyn Fn(&C) -> bool + Send + Sync + 'a>,
}
impl<'a, C> SendIfPrompt<'a, C>
where
    C: Context
{
    pub fn get_then(&self) -> &SendPromptVariant<C> {
        &self.then
    }
    pub fn get_otherwise(&self) -> Option<&SendPromptVariant<C>> {
        self.otherwise.as_ref()
    }
    pub fn get_condition(&self) -> &(dyn Fn(&C) -> bool + Send + Sync + 'a) {
        &self.condition
    }
}
impl<C> Prompt<C> for SendIfPrompt<'_, C>
where
    C: Context
{
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        if (self.condition)(context) {
            self.then.prompt_str(context)
        } else {
            if let Some(otherwise) = self.otherwise.as_ref() {
                otherwise.prompt_str(context)
            }
            else {
                Ok(None)
            }
        }
    }
}
pub struct SendLoopPrompt<'a,  C>
where
    C: Context + 'a,
{
    prompt: SendPromptVariant<'a, C>,
    condition: Box<dyn Fn(&C) -> bool + Send + Sync + 'a>,
}
impl<'a, C> SendLoopPrompt<'a, C>
where
    C: Context,
{
    pub fn get_prompt(&self) -> &SendPromptVariant<C> {
        &self.prompt
    }
    pub fn get_condition(&self) -> &(dyn Fn(&C) -> bool + Send + Sync + 'a) {
        &self.condition
    }
}
impl<'a, C> Prompt<C> for SendLoopPrompt<'a, C>
where
    C: Context,
{
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        self.prompt.prompt_str(context)
    }
}

//Builders
#[derive(Default)]
pub struct SendIfPromptBuilder<'a, C, U>
where
    C: Context + 'a,
    U: Fn(&C) -> bool + Send + Sync + 'a,
{
    then: Option<SendPromptVariant<'a, C>>,
    otherwise: Option<SendPromptVariant<'a, C>>,
    condition: Option<U>,
}
impl<'a, C, U> SendIfPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool + Send + Sync + 'a,
{
    pub fn new() -> Self {
        SendIfPromptBuilder {
            then: None,
            otherwise: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<SendIfPrompt<'a, C>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.then.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(SendIfPrompt {
            then: self.then.unwrap(),
            otherwise: self.otherwise,
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn then(mut self, then: impl Into<SendPromptVariant<'a, C>>) -> Self {
        self.then = Some(then.into());
        self
    }
    pub fn otherwise(mut self, otherwise: impl Into<SendPromptVariant<'a, C>>) -> Self {
        self.otherwise = Some(otherwise.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}
#[derive(Default)]
pub struct SendLoopPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool,
{
    prompt: Option<SendPromptVariant<'a, C>>,
    condition: Option<U>,
}
impl<'a, C, U> SendLoopPromptBuilder<'a, C, U>
where
    C: Context + 'a,
    U: Fn(&C) -> bool + Send + Sync + 'a,
{
    pub fn new() -> Self {
        SendLoopPromptBuilder {
            prompt: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<SendLoopPrompt<'a, C>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.prompt.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(SendLoopPrompt {
            prompt: self.prompt.unwrap(),
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn prompt(mut self, prompt: impl Into<SendPromptVariant<'a, C>>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}

pub enum SendPromptVariant<'a, C>
where
    C: Context
{
    Naive(Cow<'a, str>),
    Template(PromptTemplate),
    If(Box<SendIfPrompt<'a, C>>),
    Loop(Box<SendLoopPrompt<'a, C>>),
}
impl<'a, C: Context> SendPromptVariant<'a, C> {
    pub fn naive(s: Cow<'a, str>) -> Self {
        SendPromptVariant::Naive(s)
    }
    pub fn if_prompt(p: SendIfPrompt<'a, C>) -> Self {
        SendPromptVariant::If(Box::new(p))
    }
    pub fn loop_prompt(p: SendLoopPrompt<'a, C>) -> Self {
        SendPromptVariant::Loop(Box::new(p))
    }
    pub fn template(t: PromptTemplate) -> Self {
        SendPromptVariant::Template(t)
    }
}
impl<'a, C: Context> From<String> for SendPromptVariant<'a, C> {
    fn from(s: String) -> Self {
        SendPromptVariant::naive(Cow::Owned(s))
    }
}
impl<'a, C: Context> From<&'a str> for SendPromptVariant<'a, C> {
    fn from(s: &'a str) -> Self {
        SendPromptVariant::naive(Cow::Borrowed(s))
    }
}
impl<'a, C: Context> From<Cow<'a, str>> for SendPromptVariant<'a, C> {
    fn from(s: Cow<'a, str>) -> Self {
        SendPromptVariant::naive(s)
    }
}
impl<'a, C: Context> From<PromptTemplate> for SendPromptVariant<'a, C> {
    fn from(t: PromptTemplate) -> Self {
        SendPromptVariant::template(t)
    }
}
impl<'a, C: Context> From<SendIfPrompt<'a, C>> for SendPromptVariant<'a, C> {
    fn from(p: SendIfPrompt<'a, C>) -> Self {
        SendPromptVariant::if_prompt(p)
    }
}
impl<'a, C: Context> From<SendLoopPrompt<'a, C>> for SendPromptVariant<'a, C> {
    fn from(p: SendLoopPrompt<'a, C>) -> Self {
        SendPromptVariant::loop_prompt(p)
    }
}
impl<C: Context> Prompt<C> for SendPromptVariant<'_, C> {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        match self {
            SendPromptVariant::Naive(s) => Ok(Some(Cow::Borrowed(s))), //Is this right?
            SendPromptVariant::Template(p) => p.prompt_str(context),
            SendPromptVariant::If(p) => p.prompt_str(context),
            SendPromptVariant::Loop(p) => p.prompt_str(context),
        }
    }
}