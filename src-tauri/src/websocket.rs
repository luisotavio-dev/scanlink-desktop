use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc;
use serde_json;
use crate::models::{BarcodeMessage, ScanMessage, PairRequest, ReconnectRequest, DeviceInfo};
use crate::storage::{AppConfig, self};
use crate::security::{self, AuthorizedDevice};

type Clients = Arc<Mutex<HashMap<usize, ClientInfo>>>;

#[derive(Clone)]
pub struct ClientInfo {
    pub sender: mpsc::UnboundedSender<Message>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub authenticated: bool,
}

#[derive(Clone)]
pub struct WebSocketServer {
    pub token: String,
    pub port: u16,
    clients: Clients,
    next_client_id: Arc<Mutex<usize>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::UnboundedSender<()>>>>,
    config: Arc<Mutex<AppConfig>>,
}

impl WebSocketServer {
    pub fn new(token: String, port: u16, config: AppConfig) -> Self {
        Self {
            token,
            port,
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(0)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn shutdown(&self) {
        log::info!("Shutting down WebSocket server...");
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
        self.clients.lock().unwrap().clear();
    }

    pub fn get_connected_count(&self) -> usize {
        // Count only authenticated clients with unique device_ids
        let clients = self.clients.lock().unwrap();
        let mut unique_devices: std::collections::HashSet<String> = std::collections::HashSet::new();

        for client in clients.values() {
            if client.authenticated {
                if let Some(ref device_id) = client.device_id {
                    unique_devices.insert(device_id.clone());
                }
            }
        }

        unique_devices.len()
    }

    pub fn get_connected_devices(&self) -> Vec<DeviceInfo> {
        self.clients
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.authenticated && c.device_id.is_some())
            .map(|c| DeviceInfo {
                device_id: c.device_id.clone().unwrap_or_default(),
                device_name: c.device_name.clone().unwrap_or_else(|| "Unknown".to_string()),
                device_model: None,
                paired_at: None,
                last_seen: None,
                is_connected: true,
            })
            .collect()
    }

    pub async fn start(
        self,
        barcode_sender: mpsc::UnboundedSender<BarcodeMessage>,
    ) -> Result<(), String> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let clients = self.clients.clone();
        let token = self.token.clone();
        let next_client_id = self.next_client_id.clone();
        let config = self.config.clone();

        // Accept WebSocket connections on root path
        let ws_route = warp::ws()
            .map(move |ws: warp::ws::Ws| {
                let clients = clients.clone();
                let token = token.clone();
                let barcode_sender = barcode_sender.clone();
                let next_client_id = next_client_id.clone();
                let config = config.clone();

                ws.on_upgrade(move |socket| {
                    handle_connection(socket, clients, token, barcode_sender, next_client_id, config)
                })
            });

        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type", "Upgrade", "Connection", "Sec-WebSocket-Key", "Sec-WebSocket-Version"]);

        let routes = ws_route.with(cors);

        log::info!("WebSocket server starting on port {}", self.port);

        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([0, 0, 0, 0], self.port), async move {
                shutdown_rx.recv().await;
                log::info!("WebSocket server received shutdown signal");
            });

        server.await;

        log::info!("WebSocket server stopped");
        Ok(())
    }
}

async fn handle_connection(
    ws: WebSocket,
    clients: Clients,
    master_token: String,
    barcode_sender: mpsc::UnboundedSender<BarcodeMessage>,
    next_client_id: Arc<Mutex<usize>>,
    config: Arc<Mutex<AppConfig>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Generate client ID
    let client_id = {
        let mut id = next_client_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    };

    // Add client to the map
    {
        let client_info = ClientInfo {
            sender: tx,
            device_id: None,
            device_name: None,
            authenticated: false,
        };
        clients.lock().unwrap().insert(client_id, client_info);
    }
    log::info!("Client {} connected", client_id);

    // Spawn task to send messages to this client
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_tx.send(message).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let clients_for_send = clients.clone();

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    log::info!("Received message from client {}: {}", client_id, text);

                    // Try to parse as JSON to check message type
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
                            match action {
                                // Handle handshake (simple connection check)
                                "handshake" => {
                                    log::info!("Client {} sent handshake", client_id);
                                    let response = serde_json::json!({
                                        "action": "handshake_ack",
                                        "status": "connected",
                                        "clientId": client_id,
                                        "timestamp": chrono::Utc::now().timestamp()
                                    });
                                    send_to_client(&clients_for_send, client_id, &response);
                                    continue;
                                }

                                // Handle pairing request (first-time connection via QR code)
                                "pair" => {
                                    if let Ok(pair_request) = serde_json::from_str::<PairRequest>(text) {
                                        handle_pair_request(
                                            &clients_for_send,
                                            client_id,
                                            &pair_request,
                                            &master_token,
                                            &config,
                                        );
                                    } else {
                                        send_error(&clients_for_send, client_id, "Invalid pair request format");
                                    }
                                    continue;
                                }

                                // Handle reconnection (returning device with auth token)
                                "reconnect" => {
                                    if let Ok(reconnect_request) = serde_json::from_str::<ReconnectRequest>(text) {
                                        handle_reconnect_request(
                                            &clients_for_send,
                                            client_id,
                                            &reconnect_request,
                                            &config,
                                        );
                                    } else {
                                        send_error(&clients_for_send, client_id, "Invalid reconnect request format");
                                    }
                                    continue;
                                }

                                // Handle scan (barcode received)
                                "scan" => {
                                    if let Ok(scan_msg) = serde_json::from_str::<ScanMessage>(text) {
                                        handle_scan_message(
                                            &clients_for_send,
                                            client_id,
                                            &scan_msg,
                                            &master_token,
                                            &config,
                                            &barcode_sender,
                                        );
                                    } else {
                                        send_error(&clients_for_send, client_id, "Invalid scan message format");
                                    }
                                    continue;
                                }

                                _ => {
                                    log::warn!("Unknown action '{}' from client {}", action, client_id);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
        }
    }

    // Client disconnected
    let was_authenticated = clients.lock().unwrap().get(&client_id).map(|c| c.authenticated).unwrap_or(false);
    clients.lock().unwrap().remove(&client_id);
    log::info!("Client {} disconnected (authenticated: {})", client_id, was_authenticated);
}

fn send_to_client(clients: &Clients, client_id: usize, message: &serde_json::Value) {
    if let Some(client) = clients.lock().unwrap().get(&client_id) {
        let _ = client.sender.send(Message::text(message.to_string()));
    }
}

fn send_error(clients: &Clients, client_id: usize, message: &str) {
    let error = serde_json::json!({
        "action": "error",
        "message": message
    });
    send_to_client(clients, client_id, &error);
}

/// Remove any existing connection from the same device to avoid duplicates
fn remove_previous_device_connection(clients: &Clients, device_id: &str, current_client_id: usize) {
    let mut clients_guard = clients.lock().unwrap();
    let old_client_ids: Vec<usize> = clients_guard
        .iter()
        .filter(|(id, info)| {
            **id != current_client_id &&
            info.device_id.as_deref() == Some(device_id)
        })
        .map(|(id, _)| *id)
        .collect();

    for old_id in old_client_ids {
        log::info!("Removing old connection {} for device {}", old_id, device_id);
        clients_guard.remove(&old_id);
    }
}

fn handle_pair_request(
    clients: &Clients,
    client_id: usize,
    request: &PairRequest,
    master_token: &str,
    config: &Arc<Mutex<AppConfig>>,
) {
    log::info!("Pair request from device {} ({})", request.device_id, request.device_name);

    // Validate master token from QR code
    if request.master_token != master_token {
        log::warn!("Invalid master token from device {}", request.device_id);
        send_error(clients, client_id, "Invalid pairing token");
        return;
    }

    // Get or create secret key
    let mut cfg = config.lock().unwrap();
    if cfg.secret_key.is_none() {
        cfg.secret_key = Some(security::generate_secret_key());
    }
    let secret_key = cfg.secret_key.clone().unwrap();

    // Create auth token for this device
    let auth_token = security::create_auth_token(&request.device_id, &secret_key);

    // Add device to authorized list
    let device = AuthorizedDevice {
        device_id: request.device_id.clone(),
        device_name: request.device_name.clone(),
        device_model: request.device_model.clone(),
        paired_at: chrono::Utc::now().to_rfc3339(),
        last_seen: chrono::Utc::now().to_rfc3339(),
    };
    cfg.add_device(device);

    // Save config
    drop(cfg);
    if let Ok(cfg) = config.lock() {
        let _ = storage::save(&cfg);
    }

    // Remove any old connection from this device
    remove_previous_device_connection(clients, &request.device_id, client_id);

    // Update client info
    if let Some(client) = clients.lock().unwrap().get_mut(&client_id) {
        client.authenticated = true;
        client.device_id = Some(request.device_id.clone());
        client.device_name = Some(request.device_name.clone());
    }

    log::info!("Device {} paired successfully", request.device_id);

    // Send success response with auth token
    let response = serde_json::json!({
        "action": "pair_ack",
        "status": "paired",
        "auth_token": auth_token,
        "device_id": request.device_id,
        "timestamp": chrono::Utc::now().timestamp()
    });
    send_to_client(clients, client_id, &response);
}

fn handle_reconnect_request(
    clients: &Clients,
    client_id: usize,
    request: &ReconnectRequest,
    config: &Arc<Mutex<AppConfig>>,
) {
    log::info!("Reconnect request from device {}", request.device_id);

    let mut cfg = config.lock().unwrap();

    // Check if device is authorized
    if !cfg.is_device_authorized(&request.device_id) {
        log::warn!("Device {} is not authorized", request.device_id);
        let error = serde_json::json!({
            "action": "reconnect_ack",
            "status": "unauthorized",
            "message": "Device not authorized. Please pair again."
        });
        send_to_client(clients, client_id, &error);
        return;
    }

    // Validate auth token
    let secret_key = match &cfg.secret_key {
        Some(key) => key.clone(),
        None => {
            log::error!("No secret key configured");
            send_error(clients, client_id, "Server configuration error");
            return;
        }
    };

    if !security::validate_auth_token(&request.auth_token, &request.device_id, &secret_key) {
        log::warn!("Invalid auth token from device {}", request.device_id);
        let error = serde_json::json!({
            "action": "reconnect_ack",
            "status": "invalid_token",
            "message": "Invalid auth token. Please pair again."
        });
        send_to_client(clients, client_id, &error);
        return;
    }

    // Update last seen
    if let Some(device) = cfg.authorized_devices.get_mut(&request.device_id) {
        device.last_seen = chrono::Utc::now().to_rfc3339();
    }

    // Save config
    drop(cfg);
    if let Ok(cfg) = config.lock() {
        let _ = storage::save(&cfg);
    }

    // Remove any old connection from this device
    remove_previous_device_connection(clients, &request.device_id, client_id);

    // Update client info
    let device_name = {
        let cfg = config.lock().unwrap();
        cfg.authorized_devices
            .get(&request.device_id)
            .map(|d| d.device_name.clone())
    };

    if let Some(client) = clients.lock().unwrap().get_mut(&client_id) {
        client.authenticated = true;
        client.device_id = Some(request.device_id.clone());
        client.device_name = device_name;
    }

    log::info!("Device {} reconnected successfully", request.device_id);

    // Send success response
    let response = serde_json::json!({
        "action": "reconnect_ack",
        "status": "connected",
        "device_id": request.device_id,
        "timestamp": chrono::Utc::now().timestamp()
    });
    send_to_client(clients, client_id, &response);
}

fn handle_scan_message(
    clients: &Clients,
    client_id: usize,
    scan_msg: &ScanMessage,
    master_token: &str,
    config: &Arc<Mutex<AppConfig>>,
    barcode_sender: &mpsc::UnboundedSender<BarcodeMessage>,
) {
    // Get the payload - if missing, we can't process
    let payload = match &scan_msg.payload {
        Some(p) => p,
        None => {
            log::warn!("Client {} sent scan without payload", client_id);
            send_error(clients, client_id, "Missing payload");
            return;
        }
    };

    // Check if client is already authenticated
    let is_authenticated = clients
        .lock()
        .unwrap()
        .get(&client_id)
        .map(|c| c.authenticated)
        .unwrap_or(false);

    // Validate token - either master token or auth token
    let valid = if is_authenticated {
        // Client already authenticated, just verify device is still authorized
        let cfg = config.lock().unwrap();
        cfg.is_device_authorized(&scan_msg.device_id)
    } else if let Some(ref auth_token) = scan_msg.auth_token {
        // Validate via encrypted auth token
        let cfg = config.lock().unwrap();
        if let Some(ref secret_key) = cfg.secret_key {
            cfg.is_device_authorized(&scan_msg.device_id)
                && security::validate_auth_token(auth_token, &scan_msg.device_id, secret_key)
        } else {
            false
        }
    } else if let Some(ref token) = scan_msg.token {
        // Fallback: validate via master token (backward compatibility / initial connection)
        token == master_token
    } else {
        false
    };

    if !valid {
        log::warn!("Client {} sent invalid token for scan", client_id);
        send_error(clients, client_id, "Invalid token");
        return;
    }

    // Update client as authenticated
    if let Some(client) = clients.lock().unwrap().get_mut(&client_id) {
        client.authenticated = true;
        client.device_id = Some(scan_msg.device_id.clone());
        client.device_name = scan_msg.device_name.clone();
    }

    log::info!(
        "Barcode received from device {}: {}",
        scan_msg.device_id,
        payload.barcode
    );

    // Send acknowledgment
    let ack = serde_json::json!({
        "action": "scan_ack",
        "status": "received",
        "barcode": payload.barcode
    });
    send_to_client(clients, client_id, &ack);

    // Convert to BarcodeMessage for frontend
    let barcode_msg = BarcodeMessage {
        barcode: payload.barcode.clone(),
        timestamp: scan_msg.timestamp,
        device_id: scan_msg.device_id.clone(),
        device_name: scan_msg.device_name.clone(),
    };

    // Forward barcode to Tauri frontend
    if let Err(e) = barcode_sender.send(barcode_msg) {
        log::error!("Failed to send barcode to frontend: {}", e);
    }
}
