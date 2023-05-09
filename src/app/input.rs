use core_foundation::base::{CFTypeRef, TCFType, CFType};
use core_foundation::dictionary::{CFDictionaryRef, CFDictionary, CFDictionaryGetValueIfPresent};
use core_foundation::number::{CFNumber, CFNumberGetValue, CFNumberRef, CFNumberType, kCFNumberFloat64Type};
use core_foundation::string::CFString;
use core_graphics::base::CGFloat;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
use core_graphics::event_source::CGEventSource;
use core_graphics::event_source::CGEventSourceStateID;
use core_graphics::geometry::CGPoint;
use core_graphics::window::CGWindowID;

use core_foundation::string::CFStringRef;
use std::os::raw::c_void;

pub type AXUIElementRef = *mut c_void;

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXUIElementCreateWithWindowID(window_id: CGWindowID) -> AXUIElementRef;
    pub fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
}

pub fn send_mouse_event(window_id: u32, x: f64, y: f64) {
    let ax_element = unsafe { AXUIElementCreateWithWindowID(window_id) };
    let position: CGPoint = unsafe {
        let mut position_raw: CFTypeRef = std::ptr::null_mut();
        let position_key = CFString::new("AXPosition");
        let position_key_raw = position_key.as_concrete_TypeRef();
        AXUIElementCopyAttributeValue(ax_element, position_key_raw, &mut position_raw);
        let position_dict = position_raw as CFDictionaryRef;
        let x_key = CFString::new("X");
        let y_key = CFString::new("Y");
        let mut x_raw: CFTypeRef = std::ptr::null_mut();
        let mut y_raw: CFTypeRef = std::ptr::null_mut();
        CFDictionaryGetValueIfPresent(position_dict, x_key.as_CFTypeRef(), &mut x_raw);
        CFDictionaryGetValueIfPresent(position_dict, y_key.as_CFTypeRef(), &mut y_raw);
        let mut x_value: f64 = 0.0;
        let mut y_value: f64 = 0.0;
        CFNumberGetValue(x_raw as CFNumberRef, kCFNumberFloat64Type, &mut x_value as *mut _ as *mut c_void);
        CFNumberGetValue(y_raw as CFNumberRef, kCFNumberFloat64Type, &mut y_value as *mut _ as *mut c_void);
        CGPoint::new(x_value, y_value)
    };
    let mouse_position = CGPoint::new(position.x + x as CGFloat, position.y + y as CGFloat);

    let event = CGEvent::new_mouse_event(
        CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
        CGEventType::MouseMoved,
        mouse_position,
        CGMouseButton::Left,
    )
    .unwrap();

    event.post(CGEventTapLocation::HID);
}
