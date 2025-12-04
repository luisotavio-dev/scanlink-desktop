# System Tray (Bandeja do Sistema)

## Visão Geral

A aplicação ScanLink Desktop agora possui um **ícone na bandeja do sistema** (System Tray) que permite controlar a aplicação mesmo quando a janela está minimizada ou oculta.

## Funcionalidades

### Menu de Contexto

Ao clicar com o botão direito no ícone da bandeja, você tem acesso às seguintes opções:

- **Abrir** - Exibe a janela principal da aplicação
- **Iniciar Servidor** - Inicia o servidor WebSocket
- **Parar Servidor** - Para o servidor WebSocket
- **Sair** - Fecha completamente a aplicação

### Comportamento do Ícone

- **Clique Esquerdo** - Abre/restaura a janela principal
- **Clique Direito** - Exibe o menu de contexto

## Benefícios para o Usuário

### 1. Trabalho em Background

- ✅ Aplicação pode rodar em segundo plano
- ✅ Servidor continua ativo mesmo com janela oculta
- ✅ Não ocupa espaço na barra de tarefas quando minimizado

### 2. Acesso Rápido

- ✅ Controle o servidor diretamente do tray
- ✅ Não precisa abrir a janela para iniciar/parar servidor
- ✅ Clique rápido para restaurar a janela

### 3. Produtividade

- ✅ Ideal para usuários que mantêm o servidor rodando o dia todo
- ✅ Interface sempre disponível sem poluir a área de trabalho
- ✅ Fácil acesso às funções principais

## Implementação Técnica

### Backend (Rust)

O system tray foi implementado usando a feature nativa do Tauri 2.x:

```rust
// Menu items
let show_item = MenuItem::with_id(app, "show", "Abrir", true, None::<&str>)?;
let start_item = MenuItem::with_id(app, "start", "Iniciar Servidor", true, None::<&str>)?;
let stop_item = MenuItem::with_id(app, "stop", "Parar Servidor", true, None::<&str>)?;
let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

// Build tray with icon and menu
TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .on_menu_event(|app, event| { /* ... */ })
    .build(app)?;
```

### Eventos do Menu

Cada item do menu dispara uma ação específica:

- **show** → Exibe e foca a janela principal
- **start** → Chama `start_server()` assincronamente
- **stop** → Chama `stop_server()` assincronamente
- **quit** → Encerra a aplicação completamente

### Frontend Integration

Comando disponível para minimizar para o tray:

```typescript
import { invoke } from "@tauri-apps/api/core"

// Minimizar janela para o tray
await invoke("minimize_to_tray")
```

## Configuração

### Cargo.toml

```toml
tauri = { version = "2.9.2", features = ["tray-icon"] }
```

### tauri.conf.json

```json
{
	"app": {
		"trayIcon": {
			"iconPath": "icons/icon.png",
			"iconAsTemplate": false
		}
	}
}
```

## Uso Recomendado

### Para Clientes que Trabalham o Dia Todo

1. **Abrir a aplicação pela manhã**
2. **Servidor inicia automaticamente**
3. **Minimizar para o tray** (janela fica oculta)
4. **Apps móveis continuam conectados**
5. **Restaurar janela quando necessário** (clique no ícone)
6. **Fechar aplicação ao final do dia** (Menu → Sair)

### Fluxo de Trabalho Sugerido

```
Manhã:
  └─ Abrir ScanLink
     └─ Servidor inicia automaticamente
        └─ Minimizar para tray

Durante o Dia:
  └─ Apps móveis escaneiam códigos
     └─ Servidor processa em background
        └─ Restaurar janela para ver histórico (se necessário)

Noite:
  └─ Clicar com direito no tray
     └─ Sair
```

## Plataformas Suportadas

- ✅ **Windows** - Ícone na bandeja do sistema (System Tray)
- ✅ **macOS** - Ícone na barra de menu (Menu Bar)
- ✅ **Linux** - Ícone na bandeja do sistema (System Tray)

## Ícones

O ícone do tray usa o mesmo ícone da aplicação definido em:

- `src-tauri/icons/icon.png`
- `src-tauri/icons/icon.ico` (Windows)
- `src-tauri/icons/icon.icns` (macOS)

## Troubleshooting

### Ícone não aparece no tray

1. Verifique se os ícones existem em `src-tauri/icons/`
2. Reconstrua a aplicação: `cargo build`
3. Reinicie a aplicação

### Menu não responde

1. Verifique os logs do Rust
2. Certifique-se que a feature `tray-icon` está habilitada
3. Teste os comandos manualmente

### Janela não restaura

1. Verifique se a janela está com ID "main"
2. Teste o comando `show` do menu
3. Reinicie a aplicação

## Melhorias Futuras

Possíveis adições ao system tray:

- [ ] Indicador visual de servidor ativo/inativo
- [ ] Contador de apps conectados no tooltip
- [ ] Notificações de novos códigos escaneados
- [ ] Histórico rápido dos últimos códigos
- [ ] Configurações rápidas (idioma, porta, etc)
