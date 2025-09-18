# Flompt

Flompt是一个轻量级的提示词工程库，它具有以下特点：

- 独立解耦，Flompt不依赖任何LLM推理引擎或API服务商，它仅操作提示词
- 迭代器样式，prompt的管道流使用类似迭代器的写法
- 流程控制，允许条件分支的prompt，循环某一prompt
- 提示词模板，轻松的创建提示词模板，提供上下文自动格式化

## 重要提示

Flompt暂时只支持**纯文字**提示词

## 快速开始

```rust
// examples/quick_start.rs
use flompt::prelude::*;

pub struct MyContext {
    pub name: String,
    pub a: i32,
    pub age: String
}
fn main() -> Result<(), Box<dyn std::error::Error> {
    //初始化Context
    let mut my_context = MyContext {
        name: "John".to_string(),
        a: 1,
        age: "18".to_string()
    };
    
    //新建一个PromptChain
    let mut chain = PromptChain::<MyContext>::new();
    //简单提示词
    chain.push("Hello");
    
    //提示词模板
    let template = PromptTemplate::new(
        "I'm {name}.\n{age} years old.\ncrazy {a}."
    ).unwrap();
    chain.push(template);
    
    //分支提示词
    let if_prompt = IfPromptBuilder::new()
        .then("If block here!")
        .otherwise("Else block here!")
        .condition(|my_context: &MyContext| my_context.a > 3)
        .build().unwrap();
    chain.push(if_prompt);
    
    //循环提示词
    let loop_prompt = LoopPromptBuilder::new()
        .prompt("Loop block here!")
        .condition(|my_context: &MyContext| my_context.a < 5)
        .build().unwrap();
    chain.push(loop_prompt);
	
    //使用chain.flow()转换为可以迭代的结构
    let mut flow = chain.flow();
    
    //迭代PromptChain
    while let Some(prompt) = flow.next_with(&my_context) {
        println!("{:?}", prompt.prompt_str(&my_context));
        println!("a: {}", my_context.a);
        my_context.a += 1;
    }
}

impl Context for MyContext {
    fn get<T: 'static>(&self, _: &str) -> Option<&T> {
        None
    }

    fn get_mut<T: 'static>(&mut self, _: &str) -> Option<&mut T> {
        None
    }
}

impl DisplayableContext for MyContext {
    fn get_displayable(&self, key: &str) -> Option<String> {
        match key {
            "name" => Some(self.name.clone()),
            "a" => Some(self.a.to_string()),
            "age" => Some(self.age.clone()),
            _ => None
        }
    }
}


```

