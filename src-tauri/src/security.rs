use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use serde::{Deserialize, Serialize};

const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedDevice {
    pub device_id: String,
    pub device_name: String,
    #[serde(default)]
    pub device_model: Option<String>,
    pub paired_at: String,
    pub last_seen: String,
}

impl AuthorizedDevice {
    #[allow(dead_code)] // Reserved for future device management feature
    pub fn new(device_id: String, device_name: String, device_model: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            device_id,
            device_name,
            device_model,
            paired_at: now.clone(),
            last_seen: now,
        }
    }
}

/// Generates a new 256-bit secret key for AES-GCM encryption
pub fn generate_secret_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    BASE64.encode(key)
}

/// Generates a random 32-character alphanumeric token
#[allow(dead_code)] // Reserved for future master token feature
pub fn generate_master_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const TOKEN_LEN: usize = 32;
    let mut rng = rand::thread_rng();

    (0..TOKEN_LEN)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Encrypts a message using AES-256-GCM
pub fn encrypt(secret_key: &str, plaintext: &str) -> Result<String, String> {
    let key_bytes = BASE64.decode(secret_key)
        .map_err(|e| format!("Invalid secret key: {}", e))?;

    if key_bytes.len() != 32 {
        return Err("Secret key must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);

    Ok(BASE64.encode(result))
}

/// Decrypts a message using AES-256-GCM
pub fn decrypt(secret_key: &str, encrypted: &str) -> Result<String, String> {
    let key_bytes = BASE64.decode(secret_key)
        .map_err(|e| format!("Invalid secret key: {}", e))?;

    if key_bytes.len() != 32 {
        return Err("Secret key must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let data = BASE64.decode(encrypted)
        .map_err(|e| format!("Invalid encrypted data: {}", e))?;

    if data.len() < NONCE_SIZE {
        return Err("Encrypted data too short".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed - invalid token or tampered data".to_string())?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// Creates an authentication token for a device
pub fn create_auth_token(device_id: &str, secret_key: &str) -> String {
    let payload = format!("scanlink:{}:{}", device_id, chrono::Utc::now().timestamp());
    match encrypt(secret_key, &payload) {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to create auth token: {}", e);
            String::new()
        }
    }
}

/// Validates an authentication token and returns true if valid
pub fn validate_auth_token(token: &str, expected_device_id: &str, secret_key: &str) -> bool {
    match decrypt(secret_key, token) {
        Ok(payload) => {
            // Format: "scanlink:device_id:timestamp"
            let parts: Vec<&str> = payload.splitn(3, ':').collect();
            if parts.len() >= 2 && parts[0] == "scanlink" && parts[1] == expected_device_id {
                true
            } else {
                log::warn!("Auth token validation failed: invalid format or device_id mismatch");
                false
            }
        }
        Err(e) => {
            log::warn!("Auth token validation failed: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_secret_key();
        let plaintext = "Hello, World!";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_auth_token() {
        let key = generate_secret_key();
        let device_id = "test-device-123";
        let master_token = generate_master_token();

        let auth_token = create_auth_token(&key, device_id, &master_token).unwrap();
        let validated_device = validate_auth_token(&key, &auth_token, &master_token).unwrap();

        assert_eq!(device_id, validated_device);
    }

    #[test]
    fn test_invalid_master_token() {
        let key = generate_secret_key();
        let device_id = "test-device-123";
        let master_token = generate_master_token();
        let wrong_token = generate_master_token();

        let auth_token = create_auth_token(&key, device_id, &master_token).unwrap();
        let result = validate_auth_token(&key, &auth_token, &wrong_token);

        assert!(result.is_err());
    }
}
