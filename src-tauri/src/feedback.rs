//! Fokusfreies, durchklickbares Feedback am ausgewählten Link oder Mauszeiger.
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn init(app: &tauri::AppHandle) -> tauri::Result<()> {
    let win = WebviewWindowBuilder::new(app, "feedback", WebviewUrl::App("feedback.html".into()))
        .title("copyNpaste Feedback")
        .inner_size(32.0, 28.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .focusable(false)
        .focused(false)
        .visible(false)
        .build()?;
    win.set_ignore_cursor_events(true)?;
    Ok(())
}

pub fn show(app: tauri::AppHandle, anchor: Option<(f64, f64)>) {
    let anchor = anchor.or_else(|| app.cursor_position().ok().map(|p| (p.x, p.y + 16.0)));
    let Some((x, y)) = anchor else { return };
    let Some(win) = app.get_webview_window("feedback") else {
        return;
    };
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
    let _ = win.eval("document.body.classList.remove('draw');void document.body.offsetWidth;document.body.classList.add('draw')");
    let _ = win.show();
    // Aufrufer ist bereits der eigene Webhook-Thread.
    std::thread::sleep(std::time::Duration::from_millis(280));
    if GENERATION.load(Ordering::SeqCst) == generation {
        let _ = win.hide();
    }
}
