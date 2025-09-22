pub mod prompt;
pub mod flow;
pub mod prelude {
    pub use crate::flow::{Flow, PromptChain};
    pub use crate::prompt::control::{IfPrompt, IfPromptBuilder, LoopPrompt, LoopPromptBuilder};
    pub use crate::prompt::naive::{Prompt, PromptVariant};
    pub use crate::prompt::template::PromptTemplate;
    pub use crate::prompt::error::PromptError;
    pub use crate::prompt::context::Context;
    #[cfg(feature = "async_oai")]
    pub use crate::feature::async_openai::prompt_result::{PromptExecutableError, PromptResult};
    #[cfg(feature = "retry")]
    pub use crate::feature::retry::RetryStrategy;
    #[cfg(feature = "retry")]
    pub use crate::feature::retry::result::RetryableExecuteError;
    #[cfg(feature = "retry")]
    pub use crate::feature::retry::result::RetryablePromptResult;
    #[cfg(feature = "send")]
    pub use crate::feature::send::{control::*, flow::*};
    #[cfg(feature = "send")]
    #[cfg(feature = "async_oai")]
    pub use crate::feature::send::result::*;
}

pub mod feature;

#[cfg(test)]
mod tests {
    use async_openai::Client;
    use async_openai::config::OpenAIConfig;
    use crate::flow::PromptChain;
    use crate::prelude::*;
    pub struct MyContext {
        pub name: String,
        pub a: i32,
        pub age: String
    }


    impl Context for MyContext {
        fn get<T: 'static>(&self, _: &str) -> Option<&T> {
            None
        }

        fn get_mut<T: 'static>(&mut self, _: &str) -> Option<&mut T> {
            None
        }
        fn template_var(&self, key: &str) -> Option<String> {
            match key {
                "name" => Some(self.name.clone()),
                "a" => Some(self.a.to_string()),
                "age" => Some(self.age.clone()),
                _ => None
            }
        }
    }


    #[test]
    fn plain_chain() {

        let mut my_context = MyContext {
            name: "John".to_string(),
            a: 1,
            age: "18".to_string()
        };
        let mut chain = PromptChain::<MyContext>::new();
        chain.push("Hello");
        let template = PromptTemplate::new(
            "I'm {name}.\n{age} years old.\ncrazy {a}."
        ).unwrap();
        chain.push(template);
        let if_prompt = IfPromptBuilder::new()
            .then("If block here!")
            .otherwise("Else block here!")
            .condition(|my_context: &MyContext| my_context.a > 3)
            .build().unwrap();
        chain.push(if_prompt);
        let loop_prompt = LoopPromptBuilder::new()
            .prompt("Loop block here!")
            .condition(|my_context: &MyContext| my_context.a < 5)
            .build().unwrap();
        chain.push(loop_prompt);
        chain.push("Can I be reached?");
        let mut flow = chain.flow();
        while let Some(prompt) = flow.next_with(&my_context) {
            println!("{:?}", prompt.prompt_str(&my_context));
            println!("a: {}", my_context.a);
            my_context.a += 1;
        }
    }
    #[cfg(feature = "send")]
    #[test]
    fn send_chain() {
        let mut my_context = MyContext {
            name: "John".to_string(),
            a: 1,
            age: "18".to_string()
        };
        let mut chain = SendPromptChain::<MyContext>::new();
        chain.push("Hello");
        let template = PromptTemplate::new(
            "I'm {name}.\n{age} years old.\ncrazy {a}."
        ).unwrap();
        chain.push(template);
        let if_prompt = SendIfPromptBuilder::new()
            .then("If block here!")
            .otherwise("Else block here!")
            .condition(|my_context: &MyContext| my_context.a > 3)
            .build().unwrap();
        chain.push(if_prompt);
        let loop_prompt = SendLoopPromptBuilder::new()
            .prompt("Loop block here!")
            .condition(|my_context: &MyContext| my_context.a < 5)
            .build().unwrap();
        chain.push(loop_prompt);
        chain.push("Can I be reached?");
        let mut flow = chain.flow();
        while let Some(prompt) = flow.next_with(&my_context) {
            println!("{:?}", prompt.prompt_str(&my_context));
            println!("a: {}", my_context.a);
            my_context.a += 1;
        }
    }
    #[cfg(feature = "async_oai")]
    #[tokio::test]
    async fn plain_execute_chain() {
        let mut my_context = MyContext {
            name: "John".to_string(),
            a: 1,
            age: "18".to_string()
        };
        dotenvy::dotenv().ok();
        let api_key = dotenvy::var("TEST_API_KEY")
            .expect("TEST_API_KEY must be set");
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base("https://api.siliconflow.cn/v1")
                .with_api_key(api_key)

        );

        let mut chain = PromptChain::<MyContext>::new();
        chain.push("Hello");
        let template = PromptTemplate::new(
            "I'm {name}.\n{age} years old.\ncrazy {a}."
        ).unwrap();
        chain.push(template);
        let if_prompt = IfPromptBuilder::new()
            .then("If block here!")
            .otherwise("Else block here!")
            .condition(|my_context: &MyContext| my_context.a > 3)
            .build().unwrap();
        chain.push(if_prompt);
        let loop_prompt = LoopPromptBuilder::new()
            .prompt("Loop block here!")
            .condition(|my_context: &MyContext| my_context.a < 5)
            .build().unwrap();
        chain.push(loop_prompt);
        chain.push("Can I be reached?");
        let mut flow = chain.flow();
        while let Some(prompt) = flow.next_with(&my_context) {
            println!("{:?}", prompt.prompt_str(&my_context));
            println!("a: {}", my_context.a);
            prompt
                .to_executable(|_, _| {Ok(None::<String>)})
                .models(
                    vec![
                        "Qwen/Qwen3-8B"
                    ]
                )
                .execute(&mut my_context, &client, None)
                .await
                .unwrap();
            my_context.a += 1;
        }
    }
    #[cfg(feature = "async_oai")]
    #[cfg(feature = "send")]
    #[tokio::test]
    async fn send_execute_chain() {
        let mut my_context = MyContext {
            name: "John".to_string(),
            a: 1,
            age: "18".to_string()
        };
        dotenvy::dotenv().ok();
        let api_key = dotenvy::var("TEST_API_KEY")
            .expect("TEST_API_KEY must be set");
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base("https://api.siliconflow.cn/v1")
                .with_api_key(api_key)

        );

        let mut chain = SendPromptChain::<MyContext>::new();
        chain.push("Hello");
        let template = PromptTemplate::new(
            "I'm {name}.\n{age} years old.\ncrazy {a}."
        ).unwrap();
        chain.push(template);
        let if_prompt = SendIfPromptBuilder::new()
            .then("If block here!")
            .otherwise("Else block here!")
            .condition(|my_context: &MyContext| my_context.a > 3)
            .build().unwrap();
        chain.push(if_prompt);
        let loop_prompt = SendLoopPromptBuilder::new()
            .prompt("Loop block here!")
            .condition(|my_context: &MyContext| my_context.a < 5)
            .build().unwrap();
        chain.push(loop_prompt);
        chain.push("Can I be reached?");
        let mut flow = chain.flow();
        while let Some(prompt) = flow.next_with(&my_context) {
            println!("{:?}", prompt.prompt_str(&my_context));
            println!("a: {}", my_context.a);
            prompt
                .to_executable(|_, _| {Ok(None::<String>)})
                .models(
                    vec![
                        "Qwen/Qwen3-8B"
                    ]
                )
                .execute(&mut my_context, &client, None)
                .await
                .unwrap();
            my_context.a += 1;
        }
    }
    #[cfg(feature = "retry")]
    #[tokio::test]
    async fn retry_execute_chain() {
        let mut my_context = MyContext {
            name: "John".to_string(),
            a: 1,
            age: "18".to_string()
        };
        dotenvy::dotenv().ok();
        let api_key = dotenvy::var("TEST_API_KEY")
            .expect("TEST_API_KEY must be set");
        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base("https://api.siliconflow.cn/v1")
                .with_api_key(api_key)

        );

        let mut chain = SendPromptChain::<MyContext>::new();
        chain.push("Hello");
        let template = PromptTemplate::new(
            "I'm {name}.\n{age} years old.\ncrazy {a}."
        ).unwrap();
        chain.push(template);
        let if_prompt = SendIfPromptBuilder::new()
            .then("If block here!")
            .otherwise("Else block here!")
            .condition(|my_context: &MyContext| my_context.a > 3)
            .build().unwrap();
        chain.push(if_prompt);
        let loop_prompt = SendLoopPromptBuilder::new()
            .prompt("Loop block here!")
            .condition(|my_context: &MyContext| my_context.a < 5)
            .build().unwrap();
        chain.push(loop_prompt);
        chain.push("Can I be reached?");
        let mut flow = chain.flow();
        while let Some(prompt) = flow.next_with(&my_context) {
            println!("{:?}", prompt.prompt_str(&my_context));
            println!("a: {}", my_context.a);
            prompt
                .to_retry_executable(|_, _| {Ok(None::<String>)})
                .models(
                    vec![
                        "Qwen/Qwen3-8B"
                    ]
                )
                .execute_with_retry(&mut my_context, &client, None)
                .await
                .retry(3)
                .await
                .unwrap();

            my_context.a += 1;
        }
    }
}
