mod models;
mod qr_service;
mod websocket;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State, Manager};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use models::{BarcodeMessage, ConnectionInfo, QRCodeData, ServerState};
use qr_service::{generate_qr_code, generate_token, get_local_ip};
use websocket::WebSocketServer;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct BarcodeEvent {
    token: String,
    barcode: String,
    timestamp: String,
}

struct AppState {
    server: Arc<Mutex<Option<WebSocketServer>>>,
    connection_info: Arc<Mutex<Option<ConnectionInfo>>>,
    server_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[tauri::command]
async fn start_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<QRCodeData, String> {
    // Stop existing server if running
    let should_wait = {
        let mut server_lock = state.server.lock().unwrap();
        let mut task_lock = state.server_task.lock().unwrap();

        if let Some(existing_server) = server_lock.take() {
            log::info!("Stopping existing server before starting new one");
            existing_server.shutdown();

            // Abort existing server task if it exists
            if let Some(task) = task_lock.take() {
                log::info!("Aborting existing server task");
                task.abort();
            }

            true
        } else {
            false
        }
    }; // Lock is dropped here

    if should_wait {
        log::info!("Waiting for port to be released...");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Generate token and get IP
    let token = generate_token();
    let ip = get_local_ip()?;
    let port = 8081;

    log::info!("=== STARTING NEW SERVER ===");
    log::info!("Generated token for server: {}", token);
    log::info!("===========================");

    let connection_info = ConnectionInfo {
        ip: ip.clone(),
        port,
        token: token.clone(),
    };

    // Generate QR code
    let qr_data = generate_qr_code(&connection_info)?;

    // Store connection info
    *state.connection_info.lock().unwrap() = Some(connection_info.clone());

    // Create WebSocket server
    let ws_server = WebSocketServer::new(token.clone(), port);

    // Store server instance
    *state.server.lock().unwrap() = Some(ws_server.clone());

    // Create channel for barcode messages
    let (barcode_tx, mut barcode_rx) = mpsc::unbounded_channel::<BarcodeMessage>();

    // Spawn task to handle barcode messages and emit to frontend
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(barcode_msg) = barcode_rx.recv().await {
            log::info!("Emitting barcode to frontend: {:?}", barcode_msg);

            // Convert timestamp to ISO 8601 string
            let timestamp_str = chrono::DateTime::from_timestamp(barcode_msg.timestamp, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            let event = BarcodeEvent {
                token: barcode_msg.token,
                barcode: barcode_msg.barcode,
                timestamp: timestamp_str,
            };

            if let Err(e) = app_handle_clone.emit("barcode-received", event) {
                log::error!("Failed to emit barcode event: {}", e);
            }
        }
    });

    // Start WebSocket server in background
    let server_handle = tokio::spawn(async move {
        log::info!("WebSocket server task started with token: {}", ws_server.token);
        if let Err(e) = ws_server.start(barcode_tx).await {
            log::error!("WebSocket server error: {}", e);
        }
        log::info!("WebSocket server task ended");
    });

    // Store the server task handle
    *state.server_task.lock().unwrap() = Some(server_handle);

    log::info!("Server started on {}:{}", ip, port);

    Ok(qr_data)
}

#[tauri::command]
async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    let mut server_lock = state.server.lock().unwrap();
    let mut task_lock = state.server_task.lock().unwrap();

    if server_lock.is_none() {
        return Err("Server is not running".to_string());
    }

    log::info!("Stopping server...");

    // Shutdown the WebSocket server gracefully
    if let Some(server) = server_lock.as_ref() {
        server.shutdown();
    }

    // Abort the server task
    if let Some(task) = task_lock.take() {
        log::info!("Aborting server task");
        task.abort();
    }

    *server_lock = None;
    *state.connection_info.lock().unwrap() = None;

    log::info!("Server stopped");
    Ok(())
}

#[tauri::command]
async fn get_server_state(state: State<'_, AppState>) -> Result<ServerState, String> {
    let server_lock = state.server.lock().unwrap();
    let is_running = server_lock.is_some();

    let connected_clients = if let Some(server) = server_lock.as_ref() {
        server.get_connected_count()
    } else {
        0
    };

    Ok(ServerState {
        is_running,
        connected_clients,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Create system tray menu
            let show_item = MenuItem::with_id(app, "show", "Abrir", true, None::<&str>)?;
            let separator1 = PredefinedMenuItem::separator(app)?;
            let start_item = MenuItem::with_id(app, "start", "▶️  Iniciar Servidor", true, None::<&str>)?;
            let stop_item = MenuItem::with_id(app, "stop", "⏹️  Parar Servidor", true, None::<&str>)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "❌ Sair", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&show_item, &separator1, &start_item, &stop_item, &separator2, &quit_item],
            )?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "start" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<AppState>();
                                if let Err(e) = start_server(state, app_handle.clone()).await {
                                    log::error!("Failed to start server from tray: {}", e);
                                }
                            });
                        }
                        "stop" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<AppState>();
                                if let Err(e) = stop_server(state).await {
                                    log::error!("Failed to stop server from tray: {}", e);
                                }
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .manage(AppState {
            server: Arc::new(Mutex::new(None)),
            connection_info: Arc::new(Mutex::new(None)),
            server_task: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_server_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
