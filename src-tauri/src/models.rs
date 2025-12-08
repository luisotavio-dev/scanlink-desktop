use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub ip: String,
    pub port: u16,
    pub token: String,
    /// Secret key for encryption (only included in QR for initial pairing)
    #[serde(rename = "secretKey", skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
}

// Barcode message (internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeMessage {
    pub barcode: String,
    pub timestamp: i64,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: Option<String>,
}

// Scan payload from mobile app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPayload {
    pub barcode: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub barcode_type: Option<String>,
}

// Scan message from mobile app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMessage {
    pub action: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName", skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(rename = "deviceModel", skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<ScanPayload>,
    /// Plain token (for initial pairing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Encrypted auth token (for reconnection)
    #[serde(rename = "authToken", skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

// Pair request from mobile app (first-time connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequest {
    pub action: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "deviceModel", skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    #[serde(rename = "masterToken")]
    pub master_token: String,
}

// Reconnect request from mobile app (returning device)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectRequest {
    pub action: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "authToken")]
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeData {
    pub qr_base64: String,
    pub connection_info: ConnectionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub is_running: bool,
    pub connected_clients: usize,
}

// Device info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "deviceModel", skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    #[serde(rename = "pairedAt", skip_serializing_if = "Option::is_none")]
    pub paired_at: Option<String>,
    #[serde(rename = "lastSeen", skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<String>,
    #[serde(rename = "isConnected", default)]
    pub is_connected: bool,
}

// App settings for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "autoStart")]
    pub auto_start: bool,
    #[serde(rename = "minimizeToTray")]
    pub minimize_to_tray: bool,
    #[serde(rename = "startMinimized")]
    pub start_minimized: bool,
}

// WebSocket response messages
#[allow(dead_code)] // Reserved for future WebSocket response handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(rename = "authToken", skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(rename = "deviceId", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}
