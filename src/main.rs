extern crate cocoa;
extern crate core_graphics;
extern crate objc;

use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyProhibited};
use cocoa::base::{nil, YES};
use cocoa::foundation::NSAutoreleasePool;
use core_foundation::array::CFArray;
use core_foundation::attributed_string::CFAttributedString;
use core_foundation::base::TCFType;
use core_graphics::display::CGMainDisplayID;
use core_graphics::window::{kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo};

fn main() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyProhibited);

        let window_list_ref = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, 0);
        let window_list: CFArray = TCFType::wrap_under_create_rule(window_list_ref);

        let window_count = window_list.len();
        println!("Found {} windows on the screen.", window_count);

        for i in 0..window_count {
            let window_info = window_list.get(i).unwrap();
            println!("Window: {}: {:?}", i, window_info)
        }

        app.run()
    }
}
