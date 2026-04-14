mod app;
mod config;
mod ffi;
mod grid;
mod hotkey;
mod window_manager;

use core_foundation::date::CFAbsoluteTimeGetCurrent;
use core_foundation::runloop::{
    kCFRunLoopCommonModes, CFRunLoop, CFRunLoopTimer, CFRunLoopTimerContext,
};
use std::ffi::c_void;
use std::sync::mpsc;

const POLL_INTERVAL: f64 = 0.05; // 50ms

fn main() {
    if !window_manager::check_accessibility(true) {
        eprintln!(
            "[cartographer] no accessibility permission. \
             go grant it in System Settings and restart"
        );
    }

    let cfg = config::load();

    let (trigger_keycode, trigger_flags) = cfg.hotkey.resolve().unwrap_or_else(|e| {
        eprintln!("[cartographer] hotkey config broken ({e}), using defaults");
        config::HotkeyConfig::default().resolve().unwrap()
    });

    app::initialize(&cfg);
    let rx = hotkey::start_listener(trigger_keycode, trigger_flags);

    // claude code says poll the hotkey channel from the main thread's run loop.
    let rx_ptr = Box::into_raw(Box::new(rx));

    let mut context = CFRunLoopTimerContext {
        version: 0,
        info: rx_ptr as *mut c_void,
        retain: None,
        release: None,
        copyDescription: None,
    };

    unsafe {
        let timer = CFRunLoopTimer::new(
            CFAbsoluteTimeGetCurrent() + POLL_INTERVAL,
            POLL_INTERVAL,
            0,
            0,
            timer_callback,
            &mut context,
        );

        CFRunLoop::get_current().add_timer(&timer, kCFRunLoopCommonModes);
    }

    // fire up NSApp. status item needs NSApp to exist before it can be created
    unsafe {
        use objc::{class, msg_send, sel, sel_impl};
        let app: *mut objc::runtime::Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, setActivationPolicy: 1i64]; // Accessory (no dock icon)

        ffi::swift_setup_status_item();

        let _: () = msg_send![app, run];
    }
}

extern "C" fn timer_callback(
    _timer: core_foundation::runloop::CFRunLoopTimerRef,
    info: *mut c_void,
) {
    let rx = unsafe { &*(info as *const mpsc::Receiver<hotkey::HotkeyEvent>) };
    app::process_hotkey_events(rx);
}
