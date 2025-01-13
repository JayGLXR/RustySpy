use std::sync::{Arc, Mutex};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    UIA_PROPERTY_ID,
    UIA_EVENT_ID,
    UIA_ValueValuePropertyId,
    UIA_NamePropertyId,
    UIA_LegacyIAccessibleHelpPropertyId,
    UIA_LocalizedControlTypePropertyId,
};
use windows::core::{BSTR, Interface};

use crate::logger::Logger;
use crate::error::Result;

pub fn handle_generic_event(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    event_id: UIA_EVENT_ID,
    logger: &Arc<Mutex<Logger>>,
) -> Result<()> {
    let mut window_name = Vec::<u16>::with_capacity(256);
    let mut class_name = Vec::<u16>::with_capacity(256);
    let mut help_text = Vec::<u16>::with_capacity(256);
    let mut value = Vec::<u16>::with_capacity(1024);
    
    unsafe {
        element.get_CurrentName(&mut window_name).ok();
        element.get_CurrentClassName(&mut class_name).ok();
        element.GetCurrentPropertyValue(UIA_LegacyIAccessibleHelpPropertyId, &mut help_text.into()).ok();
        element.GetCurrentPropertyValue(UIA_ValueValuePropertyId, &mut value.into()).ok();
    }

    let window_name = String::from_utf16_lossy(&window_name);
    let class_name = String::from_utf16_lossy(&class_name);
    let help_text = String::from_utf16_lossy(&help_text);
    let value = String::from_utf16_lossy(&value);

    let mut log_message = format!("{} {} [Generic Event]\n", date, process_name);
    log_message.push_str(&format!("Window: {}\n", window_name));
    log_message.push_str(&format!("Class: {}\n", class_name));
    log_message.push_str(&format!("Help: {}\n", help_text));
    log_message.push_str(&format!("Value: {}\n", value));

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
}

pub fn handle_generic_property_change(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    property_id: UIA_PROPERTY_ID,
    new_value: BSTR,
    logger: &Arc<Mutex<Logger>>,
) -> Result<()> {
    let mut control_type = Vec::<u16>::with_capacity(256);
    unsafe {
        element.get_CurrentLocalizedControlType(&mut control_type).ok();
    }
    let control_type = String::from_utf16_lossy(&control_type);

    let property_name = match property_id {
        UIA_NamePropertyId => "Name",
        UIA_ValueValuePropertyId => "Value",
        _ => "Unknown Property",
    };

    let new_value = if new_value.is_null() {
        String::from("<null>")
    } else {
        unsafe { new_value.to_string().unwrap_or_else(|_| String::from("<error>")) }
    };

    let log_message = format!(
        "{} {} [{}]\nNew {}: {}\n",
        date, process_name, control_type, property_name, new_value
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
}

pub fn get_element_value(element: &IUIAutomationElement) -> Result<String> {
    let mut value = Vec::<u16>::with_capacity(1024);
    unsafe {
        element.GetCurrentPropertyValue(UIA_ValueValuePropertyId, &mut value.into()).ok();
    }
    Ok(String::from_utf16_lossy(&value))
}

pub fn get_element_name(element: &IUIAutomationElement) -> Result<String> {
    let mut name = Vec::<u16>::with_capacity(256);
    unsafe {
        element.get_CurrentName(&mut name).ok();
    }
    Ok(String::from_utf16_lossy(&name))
} 