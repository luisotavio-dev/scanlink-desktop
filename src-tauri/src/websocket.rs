use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc;
use serde_json;
use crate::models::{BarcodeMessage, ScanMessage};

type Clients = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone)]
pub struct WebSocketServer {
    pub token: String,
    pub port: u16,
    clients: Clients,
    next_client_id: Arc<Mutex<usize>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::UnboundedSender<()>>>>,
}

impl WebSocketServer {
    pub fn new(token: String, port: u16) -> Self {
        Self {
            token,
            port,
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(0)),
            shutdown_tx: Arc::new(Mutex::new(None)),
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
        self.clients.lock().unwrap().len()
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

        // Accept WebSocket connections on root path
        let ws_route = warp::ws()
            .map(move |ws: warp::ws::Ws| {
                let clients = clients.clone();
                let token = token.clone();
                let barcode_sender = barcode_sender.clone();
                let next_client_id = next_client_id.clone();

                ws.on_upgrade(move |socket| {
                    handle_connection(socket, clients, token, barcode_sender, next_client_id)
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
    expected_token: String,
    barcode_sender: mpsc::UnboundedSender<BarcodeMessage>,
    next_client_id: Arc<Mutex<usize>>,
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
    clients.lock().unwrap().insert(client_id, tx);
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
    let mut authenticated = false;
    let clients_for_send = clients.clone();

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    log::info!("Received message from client {}: {}", client_id, text);

                    // Try to parse as JSON to check message type
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        // Check if it's a handshake message
                        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
                            if action == "handshake" {
                                log::info!("Client {} sent handshake", client_id);

                                // Send handshake response
                                let response = serde_json::json!({
                                    "action": "handshake_ack",
                                    "status": "connected",
                                    "clientId": client_id,
                                    "timestamp": chrono::Utc::now().timestamp()
                                });

                                if let Some(client_tx) = clients_for_send.lock().unwrap().get(&client_id) {
                                    let _ = client_tx.send(Message::text(response.to_string()));
                                }
                                continue;
                            }
                        }

                        // Try to parse as new format (ScanMessage)
                        if let Ok(scan_msg) = serde_json::from_str::<ScanMessage>(text) {
                            if scan_msg.action == "scan" {
                                // Validate token
                                log::info!("=== TOKEN COMPARISON (NEW FORMAT) ===");
                                log::info!("Expected token: '{}' (len: {})", expected_token, expected_token.len());
                                log::info!("Received token: '{}' (len: {})", scan_msg.payload.token, scan_msg.payload.token.len());
                                log::info!("Expected bytes: {:?}", expected_token.as_bytes());
                                log::info!("Received bytes: {:?}", scan_msg.payload.token.as_bytes());
                                log::info!("Tokens match: {}", scan_msg.payload.token == expected_token);
                                log::info!("=====================================");

                                if scan_msg.payload.token == expected_token {
                                    authenticated = true;
                                    log::info!("Client {} authenticated with valid token (new format)", client_id);

                                    // Send acknowledgment
                                    let ack = serde_json::json!({
                                        "action": "scan_ack",
                                        "status": "received",
                                        "barcode": scan_msg.payload.barcode
                                    });

                                    if let Some(client_tx) = clients_for_send.lock().unwrap().get(&client_id) {
                                        let _ = client_tx.send(Message::text(ack.to_string()));
                                    }

                                    // Convert to BarcodeMessage for frontend
                                    let barcode_msg = BarcodeMessage {
                                        token: scan_msg.payload.token,
                                        barcode: scan_msg.payload.barcode,
                                        timestamp: scan_msg.timestamp,
                                        device_id: Some(scan_msg.device_id),
                                    };

                                    // Forward barcode to Tauri frontend
                                    if let Err(e) = barcode_sender.send(barcode_msg) {
                                        log::error!("Failed to send barcode to frontend: {}", e);
                                    }
                                } else {
                                    log::warn!("Client {} sent invalid token (new format)", client_id);

                                    // Send error response
                                    let error = serde_json::json!({
                                        "action": "error",
                                        "message": "Invalid token"
                                    });

                                    if let Some(client_tx) = clients_for_send.lock().unwrap().get(&client_id) {
                                        let _ = client_tx.send(Message::text(error.to_string()));
                                    }
                                    break;
                                }
                                continue;
                            }
                        }

                        // Try to parse as old format (BarcodeMessage) for backward compatibility
                        if let Ok(barcode_msg) = serde_json::from_str::<BarcodeMessage>(text) {
                            // Validate token
                            log::info!("=== TOKEN COMPARISON (OLD FORMAT) ===");
                            log::info!("Expected token: '{}' (len: {})", expected_token, expected_token.len());
                            log::info!("Received token: '{}' (len: {})", barcode_msg.token, barcode_msg.token.len());
                            log::info!("Expected bytes: {:?}", expected_token.as_bytes());
                            log::info!("Received bytes: {:?}", barcode_msg.token.as_bytes());
                            log::info!("Tokens match: {}", barcode_msg.token == expected_token);
                            log::info!("=====================================");

                            if barcode_msg.token == expected_token {
                                authenticated = true;
                                log::info!("Client {} authenticated with valid token (old format)", client_id);

                                // Send acknowledgment
                                let ack = serde_json::json!({
                                    "action": "barcode_ack",
                                    "status": "received",
                                    "barcode": barcode_msg.barcode
                                });

                                if let Some(client_tx) = clients_for_send.lock().unwrap().get(&client_id) {
                                    let _ = client_tx.send(Message::text(ack.to_string()));
                                }

                                // Forward barcode to Tauri frontend
                                if let Err(e) = barcode_sender.send(barcode_msg) {
                                    log::error!("Failed to send barcode to frontend: {}", e);
                                }
                            } else {
                                log::warn!("Client {} sent invalid token (old format)", client_id);

                                // Send error response
                                let error = serde_json::json!({
                                    "action": "error",
                                    "message": "Invalid token"
                                });

                                if let Some(client_tx) = clients_for_send.lock().unwrap().get(&client_id) {
                                    let _ = client_tx.send(Message::text(error.to_string()));
                                }
                                break;
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
    clients.lock().unwrap().remove(&client_id);
    log::info!("Client {} disconnected (authenticated: {})", client_id, authenticated);
}
