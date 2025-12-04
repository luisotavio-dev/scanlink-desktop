# ScanLink Mobile Integration Guide

Este documento descreve como integrar o aplicativo mobile com o novo sistema de pareamento persistente do ScanLink Desktop.

## Visão Geral

O novo sistema permite que, após a primeira conexão via QR Code, o dispositivo móvel se reconecte automaticamente sem precisar escanear novamente. Isso é feito através de:

1. **Pareamento inicial**: O app mobile escaneia o QR Code e envia uma requisição de pareamento
2. **Token criptografado**: O servidor responde com um `auth_token` criptografado que o mobile armazena
3. **Reconexão automática**: Nas próximas vezes, o mobile usa o `auth_token` para se reconectar diretamente

## Protocolo WebSocket

### 1. Pareamento Inicial (action: "pair")

Quando o usuário escaneia o QR Code pela primeira vez:

```json
// Requisição do Mobile → Desktop
{
	"action": "pair",
	"deviceId": "uuid-do-dispositivo", // UUID único do dispositivo
	"deviceName": "iPhone do João", // Nome amigável do dispositivo
	"deviceModel": "iPhone 15 Pro", // Modelo do dispositivo (opcional)
	"masterToken": "token-do-qrcode" // Token extraído do QR Code
}
```

```json
// Resposta do Desktop → Mobile
{
	"action": "pair_ack",
	"status": "paired",
	"auth_token": "encrypted-token...", // Token criptografado para reconexão
	"device_id": "uuid-do-dispositivo",
	"timestamp": 1733346460
}
```

**Importante**: O mobile deve armazenar o `auth_token` de forma segura (Keychain no iOS, EncryptedSharedPreferences no Android).

### 2. Reconexão (action: "reconnect")

Quando o app mobile inicia e já tem um `auth_token` armazenado:

```json
// Requisição do Mobile → Desktop
{
	"action": "reconnect",
	"deviceId": "uuid-do-dispositivo",
	"authToken": "encrypted-token..." // Token armazenado anteriormente
}
```

```json
// Resposta do Desktop → Mobile (sucesso)
{
	"action": "reconnect_ack",
	"status": "connected",
	"device_id": "uuid-do-dispositivo",
	"timestamp": 1733346460
}
```

```json
// Resposta do Desktop → Mobile (erro - dispositivo revogado)
{
	"action": "reconnect_ack",
	"status": "unauthorized",
	"message": "Device not authorized. Please pair again."
}
```

```json
// Resposta do Desktop → Mobile (erro - token inválido)
{
	"action": "reconnect_ack",
	"status": "invalid_token",
	"message": "Invalid auth token. Please pair again."
}
```

### 3. Envio de Código de Barras (action: "scan")

Após pareamento ou reconexão:

```json
// Requisição do Mobile → Desktop
{
	"action": "scan",
	"deviceId": "uuid-do-dispositivo",
	"deviceName": "iPhone do João",
	"timestamp": 1733346460,
	"payload": {
		"barcode": "7891234567890",
		"type": "EAN_13"
	},
	"authToken": "encrypted-token..." // Token para validação (opcional se já autenticado)
}
```

```json
// Resposta do Desktop → Mobile
{
	"action": "scan_ack",
	"status": "received",
	"barcode": "7891234567890"
}
```

### 4. Mensagens de Erro

```json
{
	"action": "error",
	"message": "Invalid token"
}
```

## Fluxo Recomendado no App Mobile

### Inicialização

```swift
// Pseudocódigo Swift/Kotlin
func onAppStart() {
    // 1. Verificar se tem dados salvos (IP, porta, authToken)
    guard let savedConnection = loadSavedConnection() else {
        // Primeiro uso - aguardar QR Code
        showQRScannerPrompt()
        return
    }

    // 2. Tentar reconectar automaticamente
    tryReconnect(savedConnection)
}

func tryReconnect(connection: SavedConnection) {
    websocket.connect(ip: connection.ip, port: connection.port)

    websocket.onOpen {
        let request = ReconnectRequest(
            action: "reconnect",
            deviceId: getDeviceId(),
            authToken: connection.authToken
        )
        websocket.send(request)
    }

    websocket.onMessage { message in
        if message.action == "reconnect_ack" {
            if message.status == "connected" {
                // Sucesso! Pronto para escanear
                showScannerScreen()
            } else {
                // Token inválido ou dispositivo revogado
                clearSavedConnection()
                showQRScannerPrompt()
            }
        }
    }
}
```

### Pareamento via QR Code

```swift
func onQRCodeScanned(qrData: String) {
    // 1. Parsear dados do QR Code
    // Formato: {"ip":"192.168.1.100","port":8081,"token":"abc123..."}
    guard let connection = parseQRCode(qrData) else { return }

    // 2. Conectar ao WebSocket
    websocket.connect(ip: connection.ip, port: connection.port)

    websocket.onOpen {
        // 3. Enviar requisição de pareamento
        let request = PairRequest(
            action: "pair",
            deviceId: getDeviceId(),
            deviceName: getDeviceName(),
            deviceModel: getDeviceModel(),
            masterToken: connection.token
        )
        websocket.send(request)
    }

    websocket.onMessage { message in
        if message.action == "pair_ack" && message.status == "paired" {
            // 4. Salvar dados para reconexão futura
            saveConnection(
                ip: connection.ip,
                port: connection.port,
                authToken: message.auth_token
            )

            // 5. Pronto para escanear
            showScannerScreen()
        }
    }
}
```

### Envio de Código de Barras

```swift
func onBarcodeScanned(barcode: String, type: String) {
    let message = ScanMessage(
        action: "scan",
        deviceId: getDeviceId(),
        deviceName: getDeviceName(),
        timestamp: Date().timeIntervalSince1970,
        payload: ScanPayload(barcode: barcode, type: type)
    )
    websocket.send(message)
}
```

## Armazenamento Seguro

### iOS (Keychain)

```swift
import Security

class SecureStorage {
    static func save(key: String, value: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecValueData as String: value.data(using: .utf8)!
        ]
        SecItemAdd(query as CFDictionary, nil)
    }

    static func load(key: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true
        ]

        var result: AnyObject?
        SecItemCopyMatching(query as CFDictionary, &result)

        guard let data = result as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }
}
```

### Android (EncryptedSharedPreferences)

```kotlin
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

class SecureStorage(context: Context) {
    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    private val sharedPrefs = EncryptedSharedPreferences.create(
        context,
        "scanlink_secure_prefs",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    fun save(key: String, value: String) {
        sharedPrefs.edit().putString(key, value).apply()
    }

    fun load(key: String): String? {
        return sharedPrefs.getString(key, null)
    }
}
```

## Descoberta de Rede (Futura Implementação)

Para descoberta automática do servidor na rede local via mDNS/Bonjour:

- Tipo de serviço: `_scanlink._tcp.local.`
- TXT Record: `version=1.0`

## Estrutura de Dados para Salvar

```json
{
	"serverIP": "192.168.1.100",
	"serverPort": 8081,
	"deviceId": "uuid-do-dispositivo",
	"authToken": "encrypted-token...",
	"pairedAt": "2024-12-04T18:30:00Z",
	"deviceName": "iPhone do João"
}
```

## Tratamento de Erros

| Erro               | Causa                                            | Ação                                            |
| ------------------ | ------------------------------------------------ | ----------------------------------------------- |
| `unauthorized`     | Dispositivo foi revogado pelo usuário no desktop | Limpar dados salvos, mostrar tela de QR Code    |
| `invalid_token`    | Token expirado ou inválido                       | Limpar dados salvos, mostrar tela de QR Code    |
| WebSocket closed   | Servidor desligado ou conexão perdida            | Tentar reconectar com backoff exponencial       |
| Connection refused | Servidor não está rodando                        | Mostrar mensagem para usuário iniciar o desktop |

## Notas de Segurança

1. **Nunca** armazene o `masterToken` (do QR Code) - ele é apenas para pareamento inicial
2. **Sempre** use armazenamento seguro (Keychain/EncryptedSharedPreferences) para o `authToken`
3. O `authToken` é criptografado com AES-256-GCM no servidor
4. O `deviceId` deve ser um UUID persistente gerado no primeiro uso do app
