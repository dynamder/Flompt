pub mod prompt;
pub mod flow;
pub mod prelude {
    pub use crate::flow::{Flow, PromptChain};
    pub use crate::prompt::control::{IfPrompt, IfPromptBuilder, LoopPrompt, LoopPromptBuilder};
    pub use crate::prompt::naive::{Prompt, PromptVariant};
    pub use crate::prompt::template::PromptTemplate;
    pub use crate::prompt::error::PromptError;
    pub use crate::prompt::context::Context;
    #[cfg(feature = "async-openai")]
    pub use crate::feature::async_openai::{
        send_control::*,
        retry::RetryStrategy,
        prompt_result::{PromptResult, RetryablePromptResult, PromptExecutableError, RetryableExecuteError},
    };
}

pub mod feature;

#[cfg(test)]
mod tests {
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
    fn it_works() {

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
}
