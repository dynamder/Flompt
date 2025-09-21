use std::iter::Peekable;
use crate::prelude::SendPromptVariant;
use crate::prompt::context::Context;
use crate::prompt::naive::{PromptVariant};

#[derive(Default)]
pub struct SendPromptChain<'a, C: Context> {
    prompts: Vec<SendPromptVariant<'a, C>>
}
impl<'a, C: Context> SendPromptChain<'a, C> {
    pub fn new() -> Self {
        SendPromptChain {
            prompts: Vec::new()
        }
    }
    pub fn push(&mut self, prompt: impl Into<SendPromptVariant<'a, C>>) {
        self.prompts.push(prompt.into());
    }
    pub fn flow(&'a self) -> SendFlow<'a, C> {
        SendFlow {
            prompts: self.prompts.iter().peekable()
        }
    }
}

pub struct SendFlow<'a, C>
where
    C: Context,
{
    prompts: Peekable<std::slice::Iter<'a, SendPromptVariant<'a, C>>>
}
impl<'a, C> SendFlow<'a, C>
where
    C: Context,
{
    pub fn next_with(&mut self, context: &C) -> Option<&SendPromptVariant<C>> {
        let mut to_next: bool = true;
        let mut loop_end: bool = false;
        let res = self.prompts.peek().and_then(|&prompt| {
            match prompt {
                SendPromptVariant::Naive(_) | SendPromptVariant::Template(_)=> Some(prompt),
                SendPromptVariant::If(if_prompt) => {
                    if if_prompt.get_condition()(context) {
                        Some(if_prompt.get_then())
                    } else {
                        if_prompt.get_otherwise()
                    }
                },
                SendPromptVariant::Loop(loop_prompt) => {
                    if loop_prompt.get_condition()(context) {
                        to_next = false;
                        Some(loop_prompt.get_prompt())
                    } else {
                        loop_end = true;
                        None
                    }
                }
            }
        });
        if loop_end {
            self.prompts.next()
        }else if to_next{
            self.prompts.next();
            res
        }else {
            res
        }
    }
}