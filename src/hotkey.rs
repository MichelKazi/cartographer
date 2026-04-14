// CGEventTap on a background thread, intercepts configured hotkey globally

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};
use std::sync::mpsc;

#[derive(Debug)]
pub enum HotkeyEvent {
    Activate,
    TapDisabled,
}

pub fn start_listener(
    trigger_keycode: i64,
    required_flags: CGEventFlags,
) -> mpsc::Receiver<HotkeyEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || run_event_tap(tx, trigger_keycode, required_flags));
    rx
}

fn run_event_tap(tx: mpsc::Sender<HotkeyEvent>, trigger_keycode: i64, required_flags: CGEventFlags) {
    // only KeyDown in the mask. disabled-by-timeout events show up
    // automatically and would cause a shift overflow if you tried
    // to include them. learned that one the hard way
    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        move |_proxy, event_type, event| {
            match event_type {
                CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                    let _ = tx.send(HotkeyEvent::TapDisabled);
                    return Some(event.clone());
                }
                CGEventType::KeyDown => {}
                _ => return Some(event.clone()),
            }

            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let flags = event.get_flags();

            let modifier_mask = CGEventFlags::CGEventFlagAlternate
                | CGEventFlags::CGEventFlagCommand
                | CGEventFlags::CGEventFlagShift
                | CGEventFlags::CGEventFlagControl;
            let active_modifiers = flags & modifier_mask;

            if keycode == trigger_keycode && active_modifiers == required_flags {
                let _ = tx.send(HotkeyEvent::Activate);
                return None; // eat it
            }

            Some(event.clone())
        },
    );

    match tap {
        Ok(tap) => {
            unsafe {
                let loop_source = tap
                    .mach_port
                    .create_runloop_source(0)
                    .expect("failed to create run loop source");
                CFRunLoop::get_current().add_source(&loop_source, kCFRunLoopCommonModes);
                tap.enable();
            }
            CFRunLoop::run_current();
        }
        Err(()) => {
            eprintln!(
                "[cartographer] couldn't create event tap. \
                 go to System Settings > Privacy & Security > Accessibility \
                 and grant permission, then restart"
            );
        }
    }
}
