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
        loop {
            let mut final_break = false;
            let res = self.prompts.peek().and_then(|&prompt| {
                let mut cur_prompt = prompt;
                loop {
                    match cur_prompt {
                        PromptVariant::Naive(_) | PromptVariant::Template(_)=> {
                            final_break = true;
                            break Some(cur_prompt)
                        },
                        PromptVariant::If(if_prompt) => {
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
                        PromptVariant::Loop(loop_prompt) => {
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