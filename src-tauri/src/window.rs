#[cfg(target_os = "macos")]
use tauri::LogicalPosition;
#[cfg(target_os = "linux")]
use gtk::prelude::WidgetExt;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{App, AppHandle, Emitter, Manager, Runtime, WebviewWindow, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

// The offset from the top of the screen to the window
const TOP_OFFSET: i32 = 54;

pub struct PassiveModeState {
    enabled: AtomicBool,
}

impl Default for PassiveModeState {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(cfg!(target_os = "macos")),
        }
    }
}

impl PassiveModeState {
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
}

/// Sets up the main window with custom positioning
pub fn setup_main_window(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Try different possible window labels
    let window = app
        .get_webview_window("main")
        .or_else(|| app.get_webview_window("pluely"))
        .or_else(|| {
            // Get the first window if specific labels don't work
            app.webview_windows().values().next().cloned()
        })
        .ok_or("No window found")?;

    position_window_top_center(&window, TOP_OFFSET)?;

    let passive_state = app.state::<PassiveModeState>();
    passive_state.set(cfg!(target_os = "macos"));

    #[cfg(target_os = "macos")]
    {
        apply_main_window_passive_mode(&window, true)
            .map_err(|error| format!("Failed to apply passive mode: {}", error))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_main_window_passive_mode<R: Runtime>(
    _window: &WebviewWindow<R>,
    _passive: bool,
) -> Result<(), String> {
    // The main overlay is panelized on macOS, so runtime passive-mode toggling
    // stays disabled here until we have a panel-safe implementation.
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
fn apply_main_window_passive_mode<R: Runtime>(
    window: &WebviewWindow<R>,
    passive: bool,
) -> Result<(), String> {
    window
        .set_focusable(!passive)
        .map_err(|e| format!("Failed to set window focusable state: {}", e))?;

    window
        .set_ignore_cursor_events(passive)
        .map_err(|e| format!("Failed to set window click-through state: {}", e))?;

    Ok(())
}

/// Positions a window at the top center of the screen with a specified Y offset
pub fn position_window_top_center(
    window: &WebviewWindow,
    y_offset: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the primary monitor
    if let Some(monitor) = window.primary_monitor()? {
        let monitor_size = monitor.size();
        let window_size = window.outer_size()?;

        // Calculate center X position
        let center_x = (monitor_size.width as i32 - window_size.width as i32) / 2;

        // Set the window position
        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: center_x,
            y: y_offset,
        }))?;
    }

    Ok(())
}

/// Future function for centering window completely (both X and Y)
#[allow(dead_code)]
pub fn center_window_completely(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(monitor) = window.primary_monitor()? {
        let monitor_size = monitor.size();
        let window_size = window.outer_size()?;

        let center_x = (monitor_size.width as i32 - window_size.width as i32) / 2;
        let center_y = (monitor_size.height as i32 - window_size.height as i32) / 2;

        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: center_x,
            y: center_y,
        }))?;
    }

    Ok(())
}

#[tauri::command]
pub fn set_window_height(window: tauri::WebviewWindow, height: u32) -> Result<(), String> {
    use tauri::{LogicalSize, Size};

    // Simply set the window size with fixed width and new height
    let new_size = LogicalSize::new(600.0, height as f64);
    window
        .set_size(Size::Logical(new_size))
        .map_err(|e| format!("Failed to resize window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn set_window_opacity(app: tauri::AppHandle, opacity: f64) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let clamped = opacity.clamp(0.4, 1.0);

    apply_window_opacity(&window, clamped)
}

#[tauri::command]
pub fn set_main_window_passive(app: tauri::AppHandle, passive: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    #[cfg(target_os = "macos")]
    {
        apply_main_window_passive_mode(&window, passive)?;

        let state = app.state::<PassiveModeState>();
        state.set(passive);
        let _ = app.emit("passive-mode-changed", passive);

        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = passive;
        let state = app.state::<PassiveModeState>();
        state.set(false);
        let _ = app.emit("passive-mode-changed", false);
        let _ = window;
        Ok(())
    }
}

#[tauri::command]
pub fn get_main_window_passive(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<PassiveModeState>();
    Ok(state.is_enabled())
}

#[cfg(target_os = "linux")]
fn apply_window_opacity<R: Runtime>(window: &WebviewWindow<R>, opacity: f64) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let window_for_main_thread = window.clone();

    window
        .run_on_main_thread(move || {
            let result = window_for_main_thread
                .gtk_window()
                .map_err(|e| format!("Failed to get GTK window: {}", e))
                .map(|gtk_window| gtk_window.set_opacity(opacity));

            let _ = tx.send(result);
        })
        .map_err(|e| format!("Failed to schedule opacity update: {}", e))?;

    rx.recv()
        .map_err(|e| format!("Failed to apply window opacity: {}", e))?
}

#[cfg(target_os = "macos")]
fn apply_window_opacity<R: Runtime>(window: &WebviewWindow<R>, opacity: f64) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let window_for_main_thread = window.clone();
    let window_label = window.label().to_string();

    window
        .run_on_main_thread(move || {
            let result = window_for_main_thread
                .get_webview_panel(&window_label)
                .map_err(|e| format!("Failed to access main panel: {:?}", e))
                .map(|panel| panel.set_alpha_value(opacity));

            let _ = tx.send(result);
        })
        .map_err(|e| format!("Failed to schedule opacity update: {}", e))?;

    rx.recv()
        .map_err(|e| format!("Failed to apply window opacity: {}", e))?
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn apply_window_opacity<R: Runtime>(
    _window: &WebviewWindow<R>,
    _opacity: f64,
) -> Result<(), String> {
    Err("Window opacity is not supported on this platform".to_string())
}

#[tauri::command]
pub fn open_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    show_dashboard_window(&app)
}

#[tauri::command]
pub fn toggle_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(dashboard_window) = app.get_webview_window("dashboard") {
        match dashboard_window.is_visible() {
            Ok(true) => {
                // Window is visible, hide it
                dashboard_window
                    .hide()
                    .map_err(|e| format!("Failed to hide dashboard window: {}", e))?;
            }
            Ok(false) => {
                // Window is hidden, show and focus it
                dashboard_window
                    .show()
                    .map_err(|e| format!("Failed to show dashboard window: {}", e))?;
                dashboard_window
                    .set_focus()
                    .map_err(|e| format!("Failed to focus dashboard window: {}", e))?;
            }
            Err(e) => {
                return Err(format!("Failed to check dashboard visibility: {}", e));
            }
        }
    } else {
        // Window doesn't exist, create and show it
        show_dashboard_window(&app)?;
    }

    Ok(())
}

#[tauri::command]
pub fn move_window(app: tauri::AppHandle, direction: String, step: i32) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let current_pos = window
            .outer_position()
            .map_err(|e| format!("Failed to get window position: {}", e))?;

        let (new_x, new_y) = match direction.as_str() {
            "up" => (current_pos.x, current_pos.y - step),
            "down" => (current_pos.x, current_pos.y + step),
            "left" => (current_pos.x - step, current_pos.y),
            "right" => (current_pos.x + step, current_pos.y),
            _ => return Err(format!("Invalid direction: {}", direction)),
        };

        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: new_x,
                y: new_y,
            }))
            .map_err(|e| format!("Failed to set window position: {}", e))?;
    } else {
        return Err("Main window not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn move_window_by(app: tauri::AppHandle, dx: i32, dy: i32) -> Result<(), String> {
    move_main_window_by(&app, dx, dy)
}

pub(crate) fn move_main_window_by<R: Runtime>(
    app: &AppHandle<R>,
    dx: i32,
    dy: i32,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let current_pos = window
        .outer_position()
        .map_err(|e| format!("Failed to get window position: {}", e))?;

    let new_x = current_pos.x.saturating_add(dx).max(0);
    let new_y = current_pos.y.saturating_add(dy).max(0);

    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        }))
        .map_err(|e| format!("Failed to set window position: {}", e))?;

    Ok(())
}

pub fn create_dashboard_window<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<WebviewWindow<R>, tauri::Error> {
    let base_builder =
        WebviewWindowBuilder::new(app, "dashboard", tauri::WebviewUrl::App("/chats".into()));

    #[cfg(target_os = "macos")]
    let base_builder = base_builder
        .title("Pluely - Dashboard")
        .center()
        .decorations(true)
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .hidden_title(true)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .content_protected(true)
        .visible(true)
        .traffic_light_position(LogicalPosition::new(14.0, 18.0));

    #[cfg(not(target_os = "macos"))]
    let base_builder = base_builder
        .title("Pluely - Dashboard")
        .center()
        .decorations(true)
        .inner_size(800.0, 600.0)
        .min_inner_size(800.0, 600.0)
        .content_protected(true)
        .visible(false);

    let window = base_builder.build()?;

    // Set up close event handler - hide window instead of destroying it
    setup_dashboard_close_handler(&window);

    Ok(window)
}

/// Sets up the close event handler for the dashboard window
fn setup_dashboard_close_handler<R: Runtime>(window: &WebviewWindow<R>) {
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            // Prevent the window from being destroyed
            api.prevent_close();
            // Hide the window instead
            if let Err(e) = window_clone.hide() {
                eprintln!("Failed to hide dashboard window on close: {}", e);
            }
        }
    });
}

/// Shows the dashboard window and brings it to focus
pub fn show_dashboard_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(dashboard_window) = app.get_webview_window("dashboard") {
        // Window exists, show and focus it
        dashboard_window
            .show()
            .map_err(|e| format!("Failed to show dashboard window: {}", e))?;
        dashboard_window
            .set_focus()
            .map_err(|e| format!("Failed to focus dashboard window: {}", e))?;
    } else {
        // Window doesn't exist, create it and then show it
        let window = create_dashboard_window(app)
            .map_err(|e| format!("Failed to create dashboard window: {}", e))?;
        window
            .show()
            .map_err(|e| format!("Failed to show new dashboard window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus new dashboard window: {}", e))?;
    }
    Ok(())
}
