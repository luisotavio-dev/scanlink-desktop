# ScanLink Desktop

Uma aplicação desktop moderna construída com Tauri 2, React e Rust que recebe códigos de barras de dispositivos móveis via WebSocket em tempo real.

## 🚀 Funcionalidades

- ✅ Servidor WebSocket local para receber códigos de barras
- ✅ Geração automática de QR Code para emparelhamento
- ✅ Validação de token para conexões seguras
- ✅ Interface moderna com ShadCN UI
- ✅ Listagem em tempo real dos códigos recebidos
- ✅ Suporte a múltiplos clientes conectados
- ✅ Detecção automática do IP local

## 🏗️ Arquitetura

### Backend (Rust)

- **WebSocket Server**: Servidor WebSocket construído com `warp` rodando na porta 8081
- **QR Service**: Geração de QR codes com informações de conexão (IP, porta, token)
- **Security**: Autenticação via token aleatório gerado a cada sessão
- **Real-time Communication**: Eventos Tauri para comunicação com o frontend

### Frontend (React)

- **React 19** + **Vite** para desenvolvimento rápido
- **ShadCN UI** para componentes modernos e acessíveis
- **Zustand** para gerenciamento de estado
- **TailwindCSS** para estilização
- **Tauri API** para comunicação com o backend

## 📋 Pré-requisitos

- Node.js 18+ e pnpm
- Rust 1.77.2+
- Windows/macOS/Linux

## 🛠️ Instalação

1. Clone o repositório:

```bash
git clone <repository-url>
cd scanlink_desktop
```

2. Instale as dependências do frontend:

```bash
pnpm install
```

3. As dependências do Rust serão instaladas automaticamente ao compilar

## 🎯 Como Usar

### Desenvolvimento

Execute a aplicação em modo desenvolvimento:

```bash
pnpm tauri dev
```

Isso iniciará:

1. O servidor Vite na porta 5173
2. A aplicação desktop Tauri
3. Hot-reload automático para mudanças no código

### Produção

Para compilar a aplicação para produção:

```bash
pnpm tauri build
```

O executável será gerado em `src-tauri/target/release/`

## 📱 Conexão Mobile

1. **Inicie o servidor**: Clique em "Start Server" na aplicação desktop
2. **Escaneie o QR Code**: Use seu aplicativo mobile para escanear o QR code exibido
3. **Envie códigos**: O app mobile deve enviar mensagens no formato JSON:

```json
{
	"token": "<token-do-qr-code>",
	"barcode": "1234567890",
	"timestamp": "2025-11-18T12:00:00Z"
}
```

### Exemplo de Conexão WebSocket

```javascript
// Conectar ao WebSocket
const ws = new WebSocket("ws://192.168.0.12:8081/ws")

// Enviar código de barras
ws.send(
	JSON.stringify({
		token: "abc123...",
		barcode: "7891234567890",
		timestamp: new Date().toISOString(),
	}),
)
```

## 🏃 Fluxo de Funcionamento

1. **Desktop inicia**: Aplicação é iniciada sem servidor ativo
2. **Usuário clica "Start Server"**:
   - Gera token aleatório de 32 caracteres
   - Detecta IP local da máquina
   - Inicia servidor WebSocket na porta 8081
   - Gera QR code com {ip, port, token}
   - Exibe QR code na tela
3. **Mobile escaneia QR**:
   - Obtém IP, porta e token
   - Conecta ao WebSocket
4. **Mobile envia código**:
   - Envia JSON com token, barcode e timestamp
   - Servidor valida o token
   - Se válido, broadcasta evento para frontend
5. **Desktop exibe**: Código aparece instantaneamente na lista

## 🔧 Estrutura do Projeto

```
scanlink_desktop/
├── src/                        # Frontend React
│   ├── components/
│   │   └── ui/                 # Componentes ShadCN
│   ├── pages/
│   │   └── Home.tsx            # Página principal
│   ├── store/
│   │   └── index.ts            # Estado global (Zustand)
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                  # Backend Rust
│   ├── src/
│   │   ├── lib.rs              # Configuração Tauri e commands
│   │   ├── main.rs             # Entry point
│   │   ├── models.rs           # Estruturas de dados
│   │   ├── websocket.rs        # Servidor WebSocket
│   │   └── qr_service.rs       # Geração de QR codes
│   ├── Cargo.toml              # Dependências Rust
│   └── tauri.conf.json         # Configuração Tauri
└── package.json                # Dependências Node.js
```

## 📦 Dependências Principais

### Rust

- `tauri` - Framework para apps desktop
- `tokio` - Runtime assíncrono
- `warp` - Framework WebSocket
- `qrcode` - Geração de QR codes
- `serde` - Serialização JSON
- `rand` - Geração de tokens

### Frontend

- `react` - UI framework
- `@tauri-apps/api` - Comunicação com Tauri
- `zustand` - Gerenciamento de estado
- `lucide-react` - Ícones
- `tailwindcss` - CSS utility-first

## 🔒 Segurança

- Token aleatório gerado a cada sessão
- Validação obrigatória do token em cada mensagem
- Conexões não autenticadas são rejeitadas
- WebSocket local (sem exposição externa por padrão)

## 🐛 Troubleshooting

### Servidor não inicia

- Verifique se a porta 8081 está disponível
- Verifique permissões de firewall

### QR Code não aparece

- Verifique se o IP local foi detectado corretamente
- Tente reiniciar o servidor

### Mobile não conecta

- Certifique-se de estar na mesma rede
- Verifique se o firewall não está bloqueando a porta 8081
- Confirme que está usando o token correto do QR code

## 📝 Licença

Este projeto está sob licença MIT.

## 👨‍💻 Desenvolvimento

Para contribuir:

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## 🎉 Resultado

Uma aplicação desktop completa que:

- ✅ Abre um WebSocket para receber códigos do mobile
- ✅ Gera um QR Code de emparelhamento
- ✅ Exibe os códigos instantaneamente
- ✅ Tem UI moderna via ShadCN + React
- ✅ Está rodando e testada!
