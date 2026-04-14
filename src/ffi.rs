// rust <-> swift FFI glue. callbacks go through OnceLock because
// the FFI boundary gives us no context pointer to work with

use std::sync::OnceLock;

pub type KeyCallback = Box<dyn Fn(u16) + Send + Sync>;
pub type DismissCallback = Box<dyn Fn() + Send + Sync>;

static KEY_CALLBACK: OnceLock<KeyCallback> = OnceLock::new();
static DISMISS_CALLBACK: OnceLock<DismissCallback> = OnceLock::new();

pub fn set_key_callback(cb: KeyCallback) {
    let _ = KEY_CALLBACK.set(cb);
}

pub fn set_dismiss_callback(cb: DismissCallback) {
    let _ = DISMISS_CALLBACK.set(cb);
}


// swift calls these via @_silgen_name

#[no_mangle]
pub extern "C" fn rust_on_key_pressed(keycode: u16) {
    if let Some(cb) = KEY_CALLBACK.get() {
        cb(keycode);
    }
}

#[no_mangle]
pub extern "C" fn rust_on_overlay_dismissed() {
    if let Some(cb) = DISMISS_CALLBACK.get() {
        cb();
    }
}


// flat C struct for passing appearance config across the FFI boundary.
// field order matters -- must match the Swift side exactly
#[repr(C)]
pub struct OverlayAppearance {
    pub background_opacity: f64,
    pub border_r: f64, pub border_g: f64, pub border_b: f64, pub border_a: f64,
    pub fill_r: f64, pub fill_g: f64, pub fill_b: f64, pub fill_a: f64,
    pub highlight_r: f64, pub highlight_g: f64, pub highlight_b: f64, pub highlight_a: f64,
    pub text_r: f64, pub text_g: f64, pub text_b: f64, pub text_a: f64,
    pub font_size_ratio: f64,
    pub border_width: f64,
    pub cell_gap: f64,
    pub corner_radius: f64,
}

// rust calls these, implemented in swift via @_cdecl

extern "C" {
    pub fn swift_show_overlay(x: f64, y: f64, width: f64, height: f64);
    pub fn swift_hide_overlay();
    pub fn swift_highlight_cell(col: i32, row: i32);
    pub fn swift_clear_highlight();
    pub fn swift_setup_status_item();
    pub fn swift_configure_appearance(appearance: *const std::ffi::c_void);
    pub fn swift_configure_grid_labels(labels: *const std::os::raw::c_char);
}
