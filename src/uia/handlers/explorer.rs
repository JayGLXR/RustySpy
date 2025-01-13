use std::sync::{Arc, Mutex};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    UIA_EVENT_ID,
    UIA_Text_TextSelectionChangedEventId,
    UIA_Text_TextChangedEventId,
    UIA_Invoke_InvokedEventId,
    UIA_Window_WindowOpenedEventId,
    UIA_NamePropertyId,
    UIA_ClassNamePropertyId,
};
use windows::core::Interface;
use crate::logger::Logger;
use crate::error::Result;
use super::common::{handle_generic_event, get_element_name};

pub fn create_explorer_handler(logger: Arc<Mutex<Logger>>) -> Box<dyn Fn(&IUIAutomationElement, &str, &str, UIA_EVENT_ID) -> Result<()> + Send + Sync> {
    Box::new(move |element, process_name, date, event_id| {
        match event_id {
            UIA_Window_WindowOpenedEventId => {
                handle_explorer_window_event(element, process_name, date, logger.clone())
            }
            UIA_Invoke_InvokedEventId => {
                handle_explorer_action_event(element, process_name, date, logger.clone())
            }
            _ => handle_generic_event(element, process_name, date, event_id, &logger),
        }
    })
}

fn handle_explorer_window_event(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    let window_name = get_element_name(element)?;
    
    let mut class_name = Vec::<u16>::with_capacity(256);
    unsafe {
        element.GetCurrentPropertyValue(UIA_ClassNamePropertyId, &mut class_name.into())?;
    }
    let class_name = String::from_utf16_lossy(&class_name);

    let log_message = format!(
        "{} {} [Explorer Window]\nPath: {}\nClass: {}\n",
        date, process_name, window_name, class_name
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
}

fn handle_explorer_action_event(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    let action_name = get_element_name(element)?;
    
    let mut class_name = Vec::<u16>::with_capacity(256);
    unsafe {
        element.GetCurrentPropertyValue(UIA_ClassNamePropertyId, &mut class_name.into())?;
    }
    let class_name = String::from_utf16_lossy(&class_name);

    let log_message = format!(
        "{} {} [Explorer Action]\nAction: {}\nClass: {}\n",
        date, process_name, action_name, class_name
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
} 