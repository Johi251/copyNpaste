//! System-Tray: copyNpaste lebt als schlanker Hintergrund-Dienst im Tray.
//! Linksklick öffnet/schließt das Magazin, Rechtsklick zeigt das Menü.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::{logger, paste};

/// Feste Tray-ID (u. a. für Tooltip-Aktualisierungen aus `state.rs`).
pub const TRAY_ID: &str = "copynpaste-tray";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Magazin öffnen / schließen", true, None::<&str>)?;
    let reverse = MenuItem::with_id(app, "reverse", "Reverse Paste — Spitze einfügen", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &reverse, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("copyNpaste — Clipboard-Magazin")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => crate::toggle_window(app),
            "reverse" => paste::reverse_paste_async(app.clone()),
            "quit" => {
                logger::log(app, "Beenden ueber Tray-Menue — copyNpaste stoppt");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::toggle_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    logger::log(app, "Tray-Icon eingerichtet");
    Ok(())
}
