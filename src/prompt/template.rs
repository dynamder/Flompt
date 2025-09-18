use std::borrow::Cow;
use crate::prompt::context::{Context};
use crate::prompt::error::{PromptError, PromptTemplateError};
use crate::prompt::naive::{Prompt};

pub enum TemplatePart {
    Text(String),
    Var(String),
}
pub struct PromptTemplate {
    parts: Vec<TemplatePart>,
}
impl PromptTemplate {
    pub fn new(template: &str) -> Result<Self, PromptTemplateError> {
        #[derive(Eq, PartialEq)]
        enum State {
            Text,
            WaitingBraceClose
        }
        let mut state = State::Text;

        let mut parts = Vec::new();
        let mut chars = template.chars().peekable();
        let mut buffer = String::new();

        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    if state == State::WaitingBraceClose {
                        return Err(PromptTemplateError::BraceMismatch);
                    }
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        buffer.push('{');
                    }else {
                        state = State::WaitingBraceClose;
                        parts.push(TemplatePart::Text(
                            std::mem::take(&mut buffer)
                        ));
                    }
                }
                '}' => {
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        buffer.push('}');
                    }else {
                        if state == State::WaitingBraceClose {
                            let var_name = std::mem::take(&mut buffer);
                            if var_name.is_empty() {
                                return Err(PromptTemplateError::EmptyVariable);
                            }
                            parts.push(TemplatePart::Var(var_name));
                        } else {
                            return Err(PromptTemplateError::BraceMismatch);
                        }
                        state = State::Text;
                    }
                }
                _ => buffer.push(c)
            }
        }
        if state == State::WaitingBraceClose {
            return Err(PromptTemplateError::BraceMismatch);
        }
        Ok(PromptTemplate {
            parts,
        })
    }
}
impl<C: Context> Prompt<C> for PromptTemplate {
    fn prompt_str(&self, context: &C) -> Result<Option<Cow<str>>, PromptError> {
        if self.parts.is_empty() {
            return Ok(None);
        }
        if self.parts.len() == 1 && let TemplatePart::Text(text) = &self.parts[0] {
            return Ok(Some(Cow::Borrowed(text)));
        }
        let rendered_str = self.parts.iter()
            .try_fold(String::new(), |mut acc, part| {
                match part {
                    TemplatePart::Text(text) => acc.push_str(text),
                    TemplatePart::Var(var_name) => {
                        if let Some(var_value) = context.template_var(var_name) {
                            acc.push_str(&var_value);
                        } else {
                            return Err(PromptError::MissingContextVar(var_name.clone()));
                        }
                    }
                }
                Ok(acc)
            })
            .map(Cow::Owned)?;
        Ok(Some(rendered_str))
    }
}