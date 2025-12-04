use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Key, Direction, Settings};
use std::thread;
use std::time::Duration;

/// Simulates typing a barcode followed by Enter key, just like a physical barcode scanner.
/// Uses clipboard paste (Ctrl+V) for instant input, then presses Enter.
pub fn type_barcode(barcode: &str) -> Result<(), String> {
    // Save current clipboard content
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    let previous_content = clipboard.get_text().ok();

    // Set barcode to clipboard
    clipboard
        .set_text(barcode)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Small delay to ensure clipboard is ready
    thread::sleep(Duration::from_millis(30));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize keyboard simulator: {}", e))?;

    // Paste with Ctrl+V (instant)
    enigo.key(Key::Control, Direction::Press)
        .map_err(|e| format!("Failed to press Ctrl: {}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Failed to press V: {}", e))?;
    enigo.key(Key::Control, Direction::Release)
        .map_err(|e| format!("Failed to release Ctrl: {}", e))?;

    // Small delay before pressing Enter
    thread::sleep(Duration::from_millis(20));

    // Press Enter key
    enigo.key(Key::Return, Direction::Click)
        .map_err(|e| format!("Failed to press Enter: {}", e))?;

    // Restore previous clipboard content after a short delay
    thread::sleep(Duration::from_millis(50));
    if let Some(prev) = previous_content {
        let _ = clipboard.set_text(&prev);
    }

    Ok(())
}
