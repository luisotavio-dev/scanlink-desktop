use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub ip: String,
    pub port: u16,
    pub token: String,
}

// Old format support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeMessage {
    pub token: String,
    pub barcode: String,
    pub timestamp: i64,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

// New format support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPayload {
    pub barcode: String,
    #[serde(rename = "type")]
    pub barcode_type: Option<String>,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMessage {
    pub action: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub timestamp: i64,
    pub payload: ScanPayload,
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
