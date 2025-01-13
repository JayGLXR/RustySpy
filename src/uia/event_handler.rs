use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows::core::{implement, Interface, Result as WindowsResult};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    IUIAutomationEventHandler,
    UIA_EVENT_ID,
};

use crate::logger::Logger;
use crate::error::Result;
use super::handlers::{HandlerRegistry, common::handle_generic_event};

#[implement(IUIAutomationEventHandler)]
pub struct AutomationEventHandler {
    logger: Arc<Mutex<Logger>>,
    handlers: HandlerRegistry,
    last_event_time: Instant,
    event_timeout: Duration,
    last_value: String,
}

impl AutomationEventHandler {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        Self {
            logger: logger.clone(),
            handlers: HandlerRegistry::new(logger),
            last_event_time: Instant::now(),
            event_timeout: Duration::from_secs(1),
            last_value: String::new(),
        }
    }

    pub fn set_event_timeout(&mut self, seconds: u64) {
        self.event_timeout = Duration::from_secs(seconds);
    }
}

impl IUIAutomationEventHandler_Impl for AutomationEventHandler {
    fn HandleAutomationEvent(
        &self,
        sender: Option<&IUIAutomationElement>,
        event_id: UIA_EVENT_ID,
    ) -> WindowsResult<()> {
        let now = Instant::now();
        if now - self.last_event_time < self.event_timeout {
            return Ok(());
        }

        let element = match sender {
            Some(element) => element,
            None => return Ok(()),
        };

        // Get process name
        let process_id = unsafe {
            let mut pid = 0u32;
            windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                element.GetCurrentNativeWindowHandle()?,
                Some(&mut pid),
            );
            pid
        };

        let process_name = crate::finder::Finder::get_process_name(process_id)?;
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        if let Some(handler) = self.handlers.get_event_handler(&process_name) {
            handler(element, &process_name, &date, event_id)?;
        } else {
            handle_generic_event(element, &process_name, &date, event_id, &self.logger)?;
        }

        Ok(())
    }
} 