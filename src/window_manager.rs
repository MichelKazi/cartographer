// wraps macOS Accessibility API for window manipulation.
// needs the Accessibility permission or everything returns errors

use accessibility::{AXAttribute, AXUIElement, AXUIElementAttributes};
use accessibility_sys::{
    kAXFocusedApplicationAttribute,
    kAXPositionAttribute, kAXSizeAttribute,
    kAXValueTypeCGPoint, kAXValueTypeCGSize,
    AXValueCreate, AXValueGetValue, AXValueRef,
};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::geometry::{CGPoint, CGSize};
use std::ffi::c_void;

use crate::grid::Rect;

pub fn check_accessibility(prompt: bool) -> bool {
    unsafe {
        if prompt {
            let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
            let value = core_foundation::boolean::CFBoolean::true_value();
            let options = core_foundation::dictionary::CFDictionary::from_CFType_pairs(&[
                (key.clone(), value.as_CFType()),
            ]);
            accessibility_sys::AXIsProcessTrustedWithOptions(
                options.as_concrete_TypeRef(),
            )
        } else {
            accessibility_sys::AXIsProcessTrusted()
        }
    }
}

pub fn get_focused_window() -> Option<AXUIElement> {
    let system = AXUIElement::system_wide();

    // the high-level crate doesn't expose focused_application, so raw attribute it is
    let attr_name = CFString::from_static_string(kAXFocusedApplicationAttribute);
    let attr = AXAttribute::new(&attr_name);
    let cf_value: core_foundation::base::CFType = system.attribute(&attr).ok()?;

    let app: AXUIElement = unsafe {
        AXUIElement::wrap_under_get_rule(cf_value.as_concrete_TypeRef() as *mut _)
    };

    app.focused_window().ok()
}

pub fn tile_window(window: &AXUIElement, rect: &Rect) -> Result<(), String> {
    set_window_size(window, rect.width, rect.height)?;
    set_window_position(window, rect.x, rect.y)?;
    Ok(())
}

pub fn get_window_size(window: &AXUIElement) -> Result<(f64, f64), String> {
    unsafe {
        let attr_name = CFString::from_static_string(kAXSizeAttribute);
        let attr = AXAttribute::new(&attr_name);
        let value: core_foundation::base::CFType = window
            .attribute(&attr)
            .map_err(|e| format!("couldn't get size: {e:?}"))?;

        let ax_value = value.as_concrete_TypeRef() as AXValueRef;
        let mut size = CGSize::new(0.0, 0.0);
        let ok = AXValueGetValue(
            ax_value,
            kAXValueTypeCGSize,
            &mut size as *mut CGSize as *mut c_void,
        );
        if ok {
            Ok((size.width, size.height))
        } else {
            Err("AXValueGetValue failed for size".into())
        }
    }
}

fn set_window_position(window: &AXUIElement, x: f64, y: f64) -> Result<(), String> {
    unsafe {
        let point = CGPoint::new(x, y);
        let ax_value = AXValueCreate(
            kAXValueTypeCGPoint,
            &point as *const CGPoint as *const c_void,
        );
        if ax_value.is_null() {
            return Err("AXValueCreate failed for position".into());
        }

        let attr_name = CFString::from_static_string(kAXPositionAttribute);
        let attr = AXAttribute::new(&attr_name);
        let cf_value = core_foundation::base::CFType::wrap_under_create_rule(ax_value as *const _);
        window
            .set_attribute(&attr, cf_value)
            .map_err(|e| format!("couldn't set position: {e:?}"))
    }
}

fn set_window_size(window: &AXUIElement, width: f64, height: f64) -> Result<(), String> {
    unsafe {
        let size = CGSize::new(width, height);
        let ax_value = AXValueCreate(
            kAXValueTypeCGSize,
            &size as *const CGSize as *const c_void,
        );
        if ax_value.is_null() {
            return Err("AXValueCreate failed for size".into());
        }

        let attr_name = CFString::from_static_string(kAXSizeAttribute);
        let attr = AXAttribute::new(&attr_name);
        let cf_value = core_foundation::base::CFType::wrap_under_create_rule(ax_value as *const _);
        window
            .set_attribute(&attr, cf_value)
            .map_err(|e| format!("couldn't set size: {e:?}"))
    }
}
