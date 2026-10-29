//! Nur Accessibility-Metadaten lesen, niemals fremde Feldinhalte.
#[derive(Clone, Copy, Debug, Default)]
pub struct Target {
    pub text_only: bool,
    pub password: bool,
    pub anchor: Option<(f64, f64)>,
}

#[cfg(windows)]
mod platform {
    use super::Target;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows::Win32::{
        Foundation::HWND,
        System::{Com::*, Ole::*},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    };
    static PREVIOUS: AtomicIsize = AtomicIsize::new(0);

    pub fn remember() {
        unsafe {
            PREVIOUS.store(GetForegroundWindow().0 as isize, Ordering::SeqCst);
        }
    }
    pub fn restore() -> bool {
        let hwnd = PREVIOUS.load(Ordering::SeqCst);
        hwnd != 0 && unsafe { SetForegroundWindow(HWND(hwnd as *mut _)).as_bool() }
    }

    pub fn inspect() -> Target {
        unsafe {
            let initialized = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
            let result = (|| -> windows::core::Result<Target> {
                let automation: IUIAutomation =
                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
                let focused = automation.GetFocusedElement()?;
                let class = focused
                    .CurrentClassName()
                    .unwrap_or_default()
                    .to_string()
                    .to_ascii_lowercase();
                let edit = focused.CurrentControlType()? == UIA_EditControlTypeId;
                // RichEdit/Document-Ziele können Bilder annehmen; einfache Edit-Felder nicht.
                let text_only = edit && !class.contains("richedit");
                let password = focused.CurrentIsPassword()?.as_bool();
                let anchor = if password {
                    None
                } else {
                    selection_anchor(&focused)
                };
                Ok(Target {
                    text_only,
                    password,
                    anchor,
                })
            })()
            .unwrap_or_default();
            if initialized {
                CoUninitialize();
            }
            result
        }
    }

    unsafe fn selection_anchor(focused: &IUIAutomationElement) -> Option<(f64, f64)> {
        let pattern: IUIAutomationTextPattern =
            focused.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
        let range = pattern.GetSelection().ok()?.GetElement(0).ok()?;
        let array = range.GetBoundingRectangles().ok()?;
        if array.is_null() {
            return None;
        }
        let mut data = std::ptr::null_mut();
        let mut anchor = None;
        if SafeArrayGetUBound(array, 1).unwrap_or(-1) >= 3
            && SafeArrayAccessData(array, &mut data).is_ok()
        {
            if !data.is_null() {
                let rect = std::slice::from_raw_parts(data.cast::<f64>(), 4);
                if rect.iter().all(|v| v.is_finite()) && rect[2] > 0.0 && rect[3] > 0.0 {
                    anchor = Some((rect[0], rect[1] + rect[3]));
                }
            }
            let _ = SafeArrayUnaccessData(array);
        }
        let _ = SafeArrayDestroy(array);
        anchor
    }
}

#[cfg(windows)]
pub use platform::{inspect, remember, restore};

#[cfg(not(windows))]
pub fn remember() {}
#[cfg(not(windows))]
pub fn restore() -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn inspect() -> Target {
    let output = std::process::Command::new("osascript").args(["-e", "tell application \"System Events\" to tell first application process whose frontmost is true to get role of value of attribute \"AXFocusedUIElement\""]).output();
    let role = output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    Target {
        text_only: role == "AXTextField",
        password: role == "AXSecureTextField",
        anchor: None,
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn inspect() -> Target {
    // shortcut: Ohne verlässliche Accessibility-Metadaten bleibt die Stapelreihenfolge; bei AT-SPI-Unterstützung erweitern.
    Target::default()
}
