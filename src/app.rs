use accessibility::AXUIElement;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Instant;

use crate::ffi;
use crate::grid::{Grid, Rect, SelectionAction, SelectionState};
use crate::hotkey::HotkeyEvent;
use crate::window_manager;

const KEYCODE_ESCAPE: u16 = 53;

// global state because FFI callbacks give us no context pointer. sorry
static APP: OnceLock<Mutex<App>> = OnceLock::new();

pub struct App {
    grid: Grid,
    selection: SelectionState,
    overlay_visible: bool,
    // grabbed before overlay steals focus
    target_window: Option<AXUIElement>,
    aerospace_window_id: Option<String>,
    screen_rect: Option<Rect>,
}

// only touched from main thread, the raw pointer in AXUIElement is refcounted
unsafe impl Send for App {}

impl App {
    fn new() -> Self {
        Self {
            grid: Grid::default_4x3(),
            selection: SelectionState::new(),
            overlay_visible: false,
            target_window: None,
            aerospace_window_id: None,
            screen_rect: None,
        }
    }

    fn show_overlay(&mut self) {
        let window = match window_manager::get_focused_window() {
            Some(w) => w,
            None => {
                eprintln!("[cartographer] no focused window, bailing");
                return;
            }
        };

        self.aerospace_window_id = get_aerospace_focused_window_id();

        let (x, y, w, h) = get_main_screen_visible_frame();
        let screen = Rect { x, y, width: w, height: h };

        self.target_window = Some(window);
        self.screen_rect = Some(screen);
        self.selection.reset();
        self.overlay_visible = true;

        unsafe {
            ffi::swift_show_overlay(screen.x, screen.y, screen.width, screen.height);
        }
    }

    fn hide_overlay(&mut self) {
        self.overlay_visible = false;
        self.aerospace_window_id = None;
        self.target_window = None;
        self.screen_rect = None;
        self.selection.reset();

        unsafe {
            ffi::swift_hide_overlay();
        }
    }

    fn handle_key(&mut self, keycode: u16) {
        if !self.overlay_visible {
            return;
        }

        if keycode == KEYCODE_ESCAPE {
            self.hide_overlay();
            return;
        }

        let now = Instant::now();
        match self.selection.advance(keycode, &self.grid, now) {
            SelectionAction::FirstSelected(cell) => {
                unsafe {
                    ffi::swift_highlight_cell(cell.col as i32, cell.row as i32);
                }
            }
            SelectionAction::Tile(a, b) => {
                if let (Some(window), Some(screen)) = (&self.target_window, &self.screen_rect) {
                    let rect = self.grid.bounding_rect(a, b, *screen);
                    if let Some(ref wid) = self.aerospace_window_id {
                        if let Err(e) = resize_via_aerospace(wid, rect.width, rect.height, window) {
                            eprintln!("[cartographer] aerospace resize failed ({e}), trying AX API");
                            if let Err(e) = window_manager::tile_window(window, &rect) {
                                eprintln!("[cartographer] AX API also failed, we're cooked: {e}");
                            }
                        }
                    } else {
                        if let Err(e) = window_manager::tile_window(window, &rect) {
                            eprintln!("[cartographer] tile failed: {e}");
                        }
                    }
                }
                self.hide_overlay();
            }
            SelectionAction::Ignored => {}
        }
    }

    fn check_timeout(&mut self) {
        if self.overlay_visible && self.selection.check_timeout(Instant::now()) {
            unsafe {
                ffi::swift_clear_highlight();
            }
        }
    }
}

pub fn initialize() {
    let _ = APP.set(Mutex::new(App::new()));

    ffi::set_key_callback(Box::new(|keycode| {
        if let Some(app) = APP.get() {
            if let Ok(mut app) = app.lock() {
                app.handle_key(keycode);
            }
        }
    }));

    ffi::set_dismiss_callback(Box::new(|| {
        if let Some(app) = APP.get() {
            if let Ok(mut app) = app.lock() {
                app.hide_overlay();
            }
        }
    }));
}

/// Polled from the main thread's run loop timer.
pub fn process_hotkey_events(rx: &mpsc::Receiver<HotkeyEvent>) {
    while let Ok(event) = rx.try_recv() {
        match event {
            HotkeyEvent::Activate => {
                if let Some(app) = APP.get() {
                    if let Ok(mut app) = app.lock() {
                        if app.overlay_visible {
                            app.hide_overlay();
                        } else {
                            app.show_overlay();
                        }
                    }
                }
            }
            HotkeyEvent::TapDisabled => {
                eprintln!("[cartographer] event tap got disabled, hotkey might be dead");
            }
        }
    }

    if let Some(app) = APP.get() {
        if let Ok(mut app) = app.lock() {
            app.check_timeout();
        }
    }
}

fn get_main_screen_visible_frame() -> (f64, f64, f64, f64) {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut w: f64 = 0.0;
    let mut h: f64 = 0.0;
    unsafe {
        swift_get_screen_visible_frame(&mut x, &mut y, &mut w, &mut h);
    }
    (x, y, w, h)
}

fn get_aerospace_focused_window_id() -> Option<String> {
    use std::process::Command;
    let output = Command::new("aerospace")
        .args(["list-windows", "--focused", "--format", "%{window-id}"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

/// asks aerospace nicely to resize a window by computing the delta
/// from current size. tries both axes independently because one will
/// fail if there's no sibling in that direction (that's fine, not our problem)
fn resize_via_aerospace(
    window_id: &str,
    target_width: f64,
    target_height: f64,
    current_window: &AXUIElement,
) -> Result<(), String> {
    use std::process::Command;

    let (cur_w, cur_h) = window_manager::get_window_size(current_window)?;
    let dw = (target_width - cur_w).round() as i64;
    let dh = (target_height - cur_h).round() as i64;

    let mut any_succeeded = false;

    if dw != 0 {
        let arg = format!("{:+}", dw);
        match Command::new("aerospace")
            .args(["resize", "width", &arg, "--window-id", window_id])
            .output()
        {
            Ok(output) if output.status.success() => any_succeeded = true,
            Ok(output) => eprintln!(
                "[cartographer] width resize skipped: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(e) => return Err(format!("couldn't even run aerospace: {e}")),
        }
    }

    if dh != 0 {
        let arg = format!("{:+}", dh);
        match Command::new("aerospace")
            .args(["resize", "height", &arg, "--window-id", window_id])
            .output()
        {
            Ok(output) if output.status.success() => any_succeeded = true,
            Ok(output) => eprintln!(
                "[cartographer] height resize skipped: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(e) => return Err(format!("couldn't even run aerospace: {e}")),
        }
    }

    if dw == 0 && dh == 0 {
        any_succeeded = true;
    }

    if any_succeeded { Ok(()) } else { Err("neither axis budged, what the hell".into()) }
}

extern "C" {
    fn swift_get_screen_visible_frame(x: *mut f64, y: *mut f64, w: *mut f64, h: *mut f64);
}
