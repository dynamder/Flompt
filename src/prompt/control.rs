use std::borrow::Cow;
use crate::prompt::context::Context;
use crate::prompt::error::{IfPromptBuilderError, PromptError};
use crate::prompt::naive::{Prompt, PromptVariant};

pub struct IfPrompt<'a, C>
where
    C: Context
{
    then: PromptVariant<'a, C>,
    otherwise: Option<PromptVariant<'a, C>>,
    condition: Box<dyn Fn(&C) -> bool>,
}
impl<'a, C> IfPrompt<'a, C>
where
    C: Context
{
    pub fn get_then(&self) -> &PromptVariant<C> {
        &self.then
    }
    pub fn get_otherwise(&self) -> Option<&PromptVariant<C>> {
        self.otherwise.as_ref()
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
    pub fn new(then: PromptVariant<'a, C>, otherwise: Option<PromptVariant<'a, C>>, condition: impl Fn(&C) -> bool + 'static) -> Self {
        IfPrompt {
            then,
            otherwise,
            condition: Box::new(condition),
        }
    }
}
impl<C> Prompt<C> for IfPrompt<'_, C>
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
pub struct LoopPrompt<'a,  C>
where
    C: Context,
{
    prompt: PromptVariant<'a, C>,
    condition: Box<dyn Fn(&C) -> bool>,
}
impl<'a, C> LoopPrompt<'a, C>
where
    C: Context,
{
    pub fn get_prompt(&self) -> &PromptVariant<C> {
        &self.prompt
    }
    pub fn get_condition(&self) -> &dyn Fn(&C) -> bool {
        &self.condition
    }
    pub fn new(prompt: PromptVariant<'a, C>, condition: impl Fn(&C) -> bool + 'static) -> Self {
        LoopPrompt {
            prompt,
            condition: Box::new(condition),
        }
    }
}
impl<'a, C> Prompt<C> for LoopPrompt<'a, C>
where
    C: Context,
{
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        self.prompt.prompt_str(context)
    }
}

//Builders
#[derive(Default)]
pub struct IfPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool,
{
    then: Option<PromptVariant<'a, C>>,
    otherwise: Option<PromptVariant<'a, C>>,
    condition: Option<U>,
}
impl<'a, C, U> IfPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool + 'static
{
    pub fn new() -> Self {
        IfPromptBuilder {
            then: None,
            otherwise: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<IfPrompt<'a, C>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.then.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(IfPrompt {
            then: self.then.unwrap(),
            otherwise: self.otherwise,
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn then(mut self, then: impl Into<PromptVariant<'a, C>>) -> Self {
        self.then = Some(then.into());
        self
    }
    pub fn otherwise(mut self, otherwise: impl Into<PromptVariant<'a, C>>) -> Self {
        self.otherwise = Some(otherwise.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}
#[derive(Default)]
pub struct LoopPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool,
{
    prompt: Option<PromptVariant<'a, C>>,
    condition: Option<U>,
}
impl<'a, C, U> LoopPromptBuilder<'a, C, U>
where
    C: Context,
    U: Fn(&C) -> bool + 'static,
{
    pub fn new() -> Self {
        LoopPromptBuilder {
            prompt: None,
            condition: None,
        }
    }
    pub fn build(self) -> Result<LoopPrompt<'a, C>, IfPromptBuilderError> {
        if self.condition.is_none() {
            return Err(IfPromptBuilderError::MissingCondition);
        }
        if self.prompt.is_none() {
            return Err(IfPromptBuilderError::MissingThen);
        }
        Ok(LoopPrompt {
            prompt: self.prompt.unwrap(),
            condition: Box::new(self.condition.unwrap()),
        })
    }
    pub fn prompt(mut self, prompt: impl Into<PromptVariant<'a, C>>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }
    pub fn condition(mut self, condition: U) -> Self {
        self.condition = Some(condition);
        self
    }
}