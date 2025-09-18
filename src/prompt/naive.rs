use std::borrow::Cow;
use crate::prompt::context::{Context, DisplayableContext};
use crate::prompt::control::{IfPrompt, LoopPrompt};
use crate::prompt::error::PromptError;
use crate::prompt::template::PromptTemplate;

pub trait Prompt<C: Context> {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError>;
}
pub trait PromptWithTemplate<C: DisplayableContext>: Prompt<C> {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError>;
}

pub enum PromptVariant<'a, C>
where
    C: Context
{
    Naive(Cow<'a, str>),
    Template(PromptTemplate),
    If(Box<IfPrompt<'a, C>>),
    Loop(Box<LoopPrompt<'a, C>>),
}
impl<'a, C: Context> PromptVariant<'a, C> {
    pub fn naive(s: Cow<'a, str>) -> Self {
        PromptVariant::Naive(s)
    }
    pub fn if_prompt(p: IfPrompt<'a, C>) -> Self {
        PromptVariant::If(Box::new(p))
    }
    pub fn loop_prompt(p: LoopPrompt<'a, C>) -> Self {
        PromptVariant::Loop(Box::new(p))
    }
}
impl<'a, C: DisplayableContext> PromptVariant<'a, C> {
    pub fn template(t: PromptTemplate) -> Self {
        PromptVariant::Template(t)
    }
}
impl<'a, C: Context> From<String> for PromptVariant<'a, C> {
    fn from(s: String) -> Self {
        PromptVariant::naive(Cow::Owned(s))
    }
}
impl<'a, C: Context> From<&'a str> for PromptVariant<'a, C> {
    fn from(s: &'a str) -> Self {
        PromptVariant::naive(Cow::Borrowed(s))
    }
}
impl<'a, C: Context> From<Cow<'a, str>> for PromptVariant<'a, C> {
    fn from(s: Cow<'a, str>) -> Self {
        PromptVariant::naive(s)
    }
}
impl<'a, C: DisplayableContext> From<PromptTemplate> for PromptVariant<'a, C> {
    fn from(t: PromptTemplate) -> Self {
        PromptVariant::template(t)
    }
}
impl<'a, C: Context> From<IfPrompt<'a, C>> for PromptVariant<'a, C> {
    fn from(p: IfPrompt<'a, C>) -> Self {
        PromptVariant::if_prompt(p)
    }
}
impl<'a, C: Context> From<LoopPrompt<'a, C>> for PromptVariant<'a, C> {
    fn from(p: LoopPrompt<'a, C>) -> Self {
        PromptVariant::loop_prompt(p)
    }
}
impl<C: Context> Prompt<C> for PromptVariant<'_, C> {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        match self {
            PromptVariant::Naive(s) => Ok(Some(Cow::Borrowed(s))), //Is this right?
            PromptVariant::Template(_) => unreachable!("Template cannot be pushed in a non Displayable Context."),
            PromptVariant::If(p) => p.prompt_str(context),
            PromptVariant::Loop(p) => p.prompt_str(context),
        }
    }
}
impl <C: DisplayableContext> PromptWithTemplate<C> for PromptVariant<'_, C> {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        match self {
            PromptVariant::Naive(s) => Ok(Some(Cow::Borrowed(s))),
            PromptVariant::Template(t) => t.prompt_str(context),
            PromptVariant::If(p) => p.prompt_str(context),
            PromptVariant::Loop(p) => p.prompt_str(context),
        }
    }
}
