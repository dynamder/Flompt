use std::iter::Peekable;
use crate::prompt::context::Context;
use crate::prompt::naive::{PromptVariant};

#[derive(Default)]
pub struct PromptChain<'a, C: Context> {
    prompts: Vec<PromptVariant<'a, C>>
}
impl<'a, C: Context> PromptChain<'a, C> {
    pub fn new() -> Self {
        PromptChain {
            prompts: Vec::new()
        }
    }
    pub fn push(&mut self, prompt: impl Into<PromptVariant<'a, C>>) {
        self.prompts.push(prompt.into());
    }
    pub fn flow(&'a self) -> Flow<'a, C> {
        Flow {
            prompts: self.prompts.iter().peekable()
        }
    }
}

pub struct Flow<'a, C>
where
    C: Context,
{
    prompts: Peekable<std::slice::Iter<'a, PromptVariant<'a, C>>>
}
impl<'a, C> Flow<'a, C>
where
    C: Context,
{
    pub fn next_with(&mut self, context: &C) -> Option<&PromptVariant<C>> {
        let mut to_next: bool = true;
        let mut loop_end: bool = false;
        let res = self.prompts.peek().and_then(|&prompt| {
            match prompt {
                PromptVariant::Naive(_) | PromptVariant::Template(_)=> Some(prompt),
                PromptVariant::If(if_prompt) => {
                    if if_prompt.get_condition()(context) {
                        Some(if_prompt.get_then())
                    } else {
                        if_prompt.get_otherwise()
                    }
                },
                PromptVariant::Loop(loop_prompt) => {
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