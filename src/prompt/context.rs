use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;

pub trait Context {
    fn get<T: 'static>(&self, key: &str) -> Option<&T>;
    fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T>;

    fn template_var(&self, key: &str) -> Option<String>;
}

impl Context for HashMap<String, Box<dyn Any>> {
    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.get(key).and_then(|v| v.downcast_ref())
    }
    fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.get_mut(key).and_then(|v| v.downcast_mut())
    }
    fn template_var(&self, key: &str) -> Option<String> {
        None
    }
}
impl Context for () {
    fn get<T: 'static>(&self, _key: &str) -> Option<&T> {
        None
    }
    fn get_mut<T: 'static>(&mut self, _key: &str) -> Option<&mut T> {
        None
    }
    fn template_var(&self, _key: &str) -> Option<String> {
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
    fn template_var(&self, key: &str) -> Option<String> {
        None
    }
}