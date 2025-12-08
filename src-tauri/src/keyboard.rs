use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Key, Direction, Settings};
use std::thread;
use std::time::Duration;
use std::process::Command;

/// Check if we're running on Wayland
fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|s| s.to_lowercase() == "wayland")
        .unwrap_or(false)
}

/// Check if ydotool daemon is running and ydotool is available
fn is_ydotool_available() -> bool {
    Command::new("which")
        .arg("ydotool")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if wl-copy/wl-paste are available for Wayland clipboard
fn is_wl_clipboard_available() -> bool {
    Command::new("which")
        .arg("wl-copy")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if xdotool is available for X11
fn is_xdotool_available() -> bool {
    Command::new("which")
        .arg("xdotool")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Type barcode using ydotool (works on Wayland with GNOME)
fn type_barcode_ydotool(barcode: &str) -> Result<(), String> {
    log::info!("Using ydotool for Wayland input simulation");

    // Use wl-copy for clipboard if available, otherwise try arboard
    let clipboard_set = if is_wl_clipboard_available() {
        log::debug!("Using wl-copy for clipboard");
        Command::new("wl-copy")
            .arg(barcode)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        log::debug!("Using arboard for clipboard");
        // Fallback to arboard
        Clipboard::new()
            .and_then(|mut cb| cb.set_text(barcode))
            .is_ok()
    };

    if !clipboard_set {
        return Err("Failed to set clipboard content".to_string());
    }

    // Wait for clipboard to be ready
    thread::sleep(Duration::from_millis(200));

    // Simulate Ctrl+V using ydotool (v0.1.x syntax uses key names like xdotool)
    let paste_result = Command::new("ydotool")
        .args(["key", "ctrl+v"])
        .output();

    match paste_result {
        Ok(output) if output.status.success() => {
            log::debug!("ydotool paste command executed successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::warn!("ydotool paste output - stdout: {}, stderr: {}", stdout, stderr);
        }
        Err(e) => {
            return Err(format!("Failed to execute ydotool paste: {}", e));
        }
    }

    // Wait a bit before Enter
    thread::sleep(Duration::from_millis(100));

    // Simulate Enter key
    let enter_result = Command::new("ydotool")
        .args(["key", "enter"])
        .output();

    match enter_result {
        Ok(output) if output.status.success() => {
            log::debug!("ydotool enter command executed successfully");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("ydotool enter returned non-zero: {}", stderr);
        }
        Err(e) => {
            return Err(format!("Failed to execute ydotool enter: {}", e));
        }
    }

    log::info!("Successfully simulated barcode input via ydotool: {}", barcode);
    Ok(())
}

/// Type barcode using xdotool (works on X11)
fn type_barcode_xdotool(barcode: &str) -> Result<(), String> {
    log::info!("Using xdotool for X11 input simulation");

    // Set clipboard using xclip or arboard
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(barcode)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Wait for clipboard
    thread::sleep(Duration::from_millis(100));

    // Simulate Ctrl+V
    let paste_result = Command::new("xdotool")
        .args(["key", "ctrl+v"])
        .output();

    match paste_result {
        Ok(output) if output.status.success() => {
            log::debug!("xdotool paste command executed successfully");
        }
        Err(e) => {
            return Err(format!("Failed to execute xdotool paste: {}", e));
        }
        _ => {}
    }

    thread::sleep(Duration::from_millis(50));

    // Simulate Enter
    let enter_result = Command::new("xdotool")
        .args(["key", "Return"])
        .output();

    match enter_result {
        Ok(output) if output.status.success() => {
            log::debug!("xdotool enter command executed successfully");
        }
        Err(e) => {
            return Err(format!("Failed to execute xdotool enter: {}", e));
        }
        _ => {}
    }

    log::info!("Successfully simulated barcode input via xdotool: {}", barcode);
    Ok(())
}

/// Type barcode using enigo library (cross-platform, but may not work on all Wayland compositors)
fn type_barcode_enigo(barcode: &str) -> Result<(), String> {
    log::info!("Using enigo for input simulation");

    // Save current clipboard content
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    let previous_content = clipboard.get_text().ok();

    // Set barcode to clipboard
    clipboard
        .set_text(barcode)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Delay to ensure clipboard is ready
    thread::sleep(Duration::from_millis(150));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize keyboard simulator: {}", e))?;

    // Paste with Ctrl+V
    enigo.key(Key::Control, Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {}", e))?;
    thread::sleep(Duration::from_millis(30));
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Failed to press V: {}", e))?;
    thread::sleep(Duration::from_millis(30));
    enigo.key(Key::Control, Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {}", e))?;

    // Delay before pressing Enter
    thread::sleep(Duration::from_millis(100));

    // Press Enter key
    enigo.key(Key::Return, Direction::Click)
        .map_err(|e| format!("Failed to press Enter: {}", e))?;

    // Restore previous clipboard content after a short delay
    thread::sleep(Duration::from_millis(150));
    if let Some(prev) = previous_content {
        let _ = clipboard.set_text(&prev);
    }

    log::info!("Successfully simulated barcode input via enigo: {}", barcode);
    Ok(())
}

/// Simulates typing a barcode followed by Enter key, just like a physical barcode scanner.
/// Uses clipboard paste (Ctrl+V) for instant input, then presses Enter.
/// Automatically selects the best method based on the environment.
pub fn type_barcode(barcode: &str) -> Result<(), String> {
    log::debug!("Starting barcode input simulation for: {}", barcode);
    log::debug!("Environment - Wayland: {}, ydotool: {}, xdotool: {}",
        is_wayland(), is_ydotool_available(), is_xdotool_available());

    // On Wayland (especially GNOME), try ydotool first
    if is_wayland() {
        if is_ydotool_available() {
            log::info!("Wayland detected, using ydotool");
            return type_barcode_ydotool(barcode);
        } else {
            log::warn!("Wayland detected but ydotool not available. Trying enigo as fallback...");
            log::warn!("For best Wayland support, install ydotool: sudo apt install ydotool");
            // Try enigo anyway, it might work with some compositors
            let enigo_result = type_barcode_enigo(barcode);
            if enigo_result.is_ok() {
                return enigo_result;
            }
            log::error!("enigo failed on Wayland. Please install ydotool for Wayland support.");
            return Err("Input simulation not available on Wayland without ydotool. Install with: sudo apt install ydotool".to_string());
        }
    }

    // On X11, try xdotool first, then enigo
    if is_xdotool_available() {
        log::info!("X11 detected, using xdotool");
        return type_barcode_xdotool(barcode);
    }

    // Fallback to enigo (works on X11 and Windows/macOS)
    log::info!("Using enigo as default input simulator");
    type_barcode_enigo(barcode)
}
