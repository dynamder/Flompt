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
        loop {
            let mut final_break = false;
            let res = self.prompts.peek().and_then(|&prompt| {
                let mut cur_prompt = prompt;
                loop {
                    match cur_prompt {
                        SendPromptVariant::Naive(_) | SendPromptVariant::Template(_)=> {
                            final_break = true;
                            break Some(cur_prompt)
                        },
                        SendPromptVariant::If(if_prompt) => {
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
                        SendPromptVariant::Loop(loop_prompt) => {
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
                break res
            }
            if res.is_none() {
                if self.prompts.next().is_none() {
                    break None
                }
            }
        }
    }
}