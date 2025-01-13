mod event_handler;
mod property_handler;
mod tree_walker;
pub mod handlers;

pub use event_handler::AutomationEventHandler;
pub use property_handler::PropertyChangedEventHandler;
pub use tree_walker::TreeWalker;

use windows::Win32::UI::Accessibility::{
    IUIAutomation,
    CUIAutomation,
};
use windows::core::Interface;

use crate::error::Result;

pub struct UiaContext {
    automation: IUIAutomation,
}

impl UiaContext {
    pub fn new() -> Result<Self> {
        let automation: IUIAutomation = unsafe {
            CUIAutomation::new()?
        };
        
        Ok(Self { automation })
    }

    pub fn automation(&self) -> &IUIAutomation {
        &self.automation
    }
} 