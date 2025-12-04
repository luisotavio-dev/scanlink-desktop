use qrcode::QrCode;
use image::Luma;
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use local_ip_address::local_ip;
use crate::models::{ConnectionInfo, QRCodeData};

pub fn generate_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const TOKEN_LEN: usize = 32;
    let mut rng = rand::thread_rng();

    (0..TOKEN_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn get_local_ip() -> Result<String, String> {
    match local_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(e) => Err(format!("Failed to get local IP: {}", e)),
    }
}

pub fn generate_qr_code(connection_info: &ConnectionInfo) -> Result<QRCodeData, String> {
    // Serialize connection info to JSON
    let json_data = serde_json::to_string(connection_info)
        .map_err(|e| format!("Failed to serialize connection info: {}", e))?;

    // Generate QR code
    let code = QrCode::new(json_data.as_bytes())
        .map_err(|e| format!("Failed to generate QR code: {}", e))?;

    // Render to image
    let image = code.render::<Luma<u8>>()
        .min_dimensions(300, 300)
        .max_dimensions(500, 500)
        .build();

    // Convert to PNG bytes
    let mut png_bytes: Vec<u8> = Vec::new();
    image.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    // Encode to base64
    let base64_string = general_purpose::STANDARD.encode(&png_bytes);
    let qr_base64 = format!("data:image/png;base64,{}", base64_string);

    Ok(QRCodeData {
        qr_base64,
        connection_info: connection_info.clone(),
    })
}
