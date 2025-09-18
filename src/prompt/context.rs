use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;

pub trait Context {
    fn get<T: 'static>(&self, key: &str) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>;
}
pub trait DisplayableContext: Context {
    fn get_displayable(&self, key: &str) -> Option<String>;
}
impl Context for HashMap<String, Box<dyn Any>> {
    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.get(key).and_then(|v| v.downcast_ref())
    }
    fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.get_mut(key).and_then(|v| v.downcast_mut())
    }
}
impl Context for () {
    fn get<T: 'static>(&self, _key: &str) -> Option<&T> {
        None
    }
    fn get_mut<T: 'static>(&mut self, _key: &str) -> Option<&mut T> {
        None
    }
}
#[derive(Default)]
pub struct DefaultContext {
    data: HashMap<String, Box<dyn Any>>,
}
impl DefaultContext {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert<T: 'static>(&mut self, key: String, value: T) {
        self.data.insert(key, Box::new(value));
    }
}

impl Context for DefaultContext {
    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref())
    }
    fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.data.get_mut(key).and_then(|v| v.downcast_mut())
    }
}
#[derive(Default)]
pub struct DefaultDisplayableContext {
    data: HashMap<String, Box<dyn Display>>,
}
impl DefaultDisplayableContext {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert<T: 'static + Display>(&mut self, key: String, value: Box<T>) {
        self.data.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<&dyn Display> {
        self.data.get(key).map(|v| v as &dyn Display)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut dyn Display> {
        self.data.get_mut(key).map(|v| v as &mut dyn Display)
    }
}

impl Context for DefaultDisplayableContext {
    /// we don't support directly get in DefaultDisplayContext for Context Trait, only None will return
    fn get<T: 'static>(&self, _: &str) -> Option<&T> {
        None
    }
    /// we don't support directly get_mut in DefaultDisplayContext for Context Trait, only None will return
    fn get_mut<T: 'static>(&mut self, _: &str) -> Option<&mut T> {
        None
    }
}

impl DisplayableContext for DefaultDisplayableContext {
    fn get_displayable(&self, key: &str) -> Option<String> {
        self.data.get(key).map(|v| v.to_string())
    }
}