use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Missing variable in context: {0}.")]
    MissingContextVar(String),
    #[error("Fail to format.")]
    FailToFormatTemplate(#[from] PromptTemplateError)
}
#[derive(Debug, Error)]
pub enum PromptTemplateError {
    #[error("Brace Mismatch in prompt template.")]
    BraceMismatch,
    #[error("Empty variable in prompt template.")]
    EmptyVariable
}
#[derive(Debug, Error)]
pub enum ControlPromptBuilderError {
    #[error("{0}")]
    If(#[from] IfPromptBuilderError),
    #[error("{0}")]
    Loop(#[from] LoopPromptBuilderError),
}
#[derive(Debug, Error)]
pub enum IfPromptBuilderError {
    #[error("Missing If Condition in IfPrompt.")]
    MissingCondition,
    #[error("Missing Then Block in IfPrompt")]
    MissingThen,
}
#[derive(Debug, Error)]
pub enum LoopPromptBuilderError {
    #[error("Missing Loop Condition in LoopPrompt.")]
    MissingCondition,
    #[error("Missing Prompt in LoopPrompt")]
    MissingPrompt
}