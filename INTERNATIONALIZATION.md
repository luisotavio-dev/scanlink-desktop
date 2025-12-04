# Internacionalização (i18n)

## Visão Geral

A aplicação ScanLink Desktop foi implementada com suporte a internacionalização usando **i18next** e **react-i18next**, seguindo as melhores práticas do mercado.

## Idiomas Suportados

- 🇺🇸 **Inglês Americano (en-US)** - Idioma padrão
- 🇧🇷 **Português Brasileiro (pt-BR)**

## Estrutura de Arquivos

```
src/
├── i18n.ts                          # Configuração principal do i18next
├── i18next.d.ts                     # Type definitions para TypeScript
├── locales/
│   ├── en-US/
│   │   └── common.json              # Traduções em inglês
│   └── pt-BR/
│       └── common.json              # Traduções em português
└── components/
    └── LanguageSwitcher.tsx         # Componente de troca de idioma
```

## Detecção Automática de Idioma

O sistema detecta automaticamente o idioma do sistema operacional do usuário:

- Se o idioma do sistema começa com `pt` → Português Brasileiro
- Qualquer outro idioma → Inglês (padrão)

### Implementação

```typescript
const getSystemLanguage = (): string => {
	const systemLang = navigator.language || "en-US"

	if (systemLang.startsWith("pt")) {
		return "pt-BR"
	}

	return "en-US"
}
```

## Como Usar

### 1. Hook useTranslation

Em qualquer componente React, importe e use o hook:

```tsx
import { useTranslation } from "react-i18next"

function MyComponent() {
	const { t } = useTranslation("common")

	return <h1>{t("app.title")}</h1>
}
```

### 2. Traduções com Interpolação

Para textos com variáveis dinâmicas:

```tsx
// Arquivo de tradução
{
  "barcodes": {
    "count": "{{count}} barcode scanned",
    "count_plural": "{{count}} barcodes scanned"
  }
}

// Uso no componente
{t(`barcodes.count${barcodes.length === 1 ? '' : '_plural'}`, { count: barcodes.length })}
```

### 3. Trocar Idioma Programaticamente

```tsx
import { useTranslation } from "react-i18next"

function MyComponent() {
	const { i18n } = useTranslation()

	const changeLanguage = (lang: string) => {
		i18n.changeLanguage(lang)
	}

	return (
		<button onClick={() => changeLanguage("pt-BR")}>
			Mudar para Português
		</button>
	)
}
```

## Componente LanguageSwitcher

O componente `LanguageSwitcher` fornece uma interface visual para troca de idioma usando o **Dropdown Menu do shadcn/ui** e **flag-icons** para as bandeiras.

### Características

- **Ícone de bandeira** do país atual usando flag-icons
- **Dropdown Menu** do shadcn/ui com animações suaves
- **Ícone Languages** da biblioteca Lucide para indicar funcionalidade
- **Check visual** mostrando o idioma selecionado
- **Design responsivo** e consistente com a UI

### Dependências

- `@radix-ui/react-dropdown-menu` - Base do dropdown menu
- `flag-icons` - Bandeiras dos países em CSS
- `lucide-react` - Ícones (Languages, Check)

### Uso

```tsx
import { LanguageSwitcher } from "@/components/LanguageSwitcher"

function Header() {
	return (
		<div className="header">
			<LanguageSwitcher />
		</div>
	)
}
```

### Estrutura Visual

```
Button: [🌐 🇺🇸]
  ↓ (on click)
Dropdown:
  🇺🇸 English    ✓
  🇧🇷 Português
```

## Adicionar Novos Idiomas

### 1. Criar arquivo de tradução

Crie um novo arquivo JSON em `src/locales/{codigo-idioma}/common.json`:

```json
{
	"app": {
		"title": "ScanLink Desktop",
		"subtitle": "..."
	}
}
```

### 2. Atualizar configuração i18n

Edite `src/i18n.ts`:

```typescript
import newLang from "./locales/new-lang/common.json"

const resources = {
	"en-US": { common: enUS },
	"pt-BR": { common: ptBR },
	"new-lang": { common: newLang }, // Adicione aqui
}
```

### 3. Atualizar LanguageSwitcher

Adicione a nova opção em `src/components/LanguageSwitcher.tsx`:

```typescript
const languages = [
	{ code: "en-US", name: "English", flag: "us" },
	{ code: "pt-BR", name: "Português", flag: "br" },
	{ code: "es-ES", name: "Español", flag: "es" }, // Adicione aqui
]
```

**Nota**: Use os códigos de país ISO 3166-1 alpha-2 (minúsculas) do [flag-icons](https://github.com/lipis/flag-icons).

## Boas Práticas

### 1. Organização das Chaves

Use uma estrutura hierárquica clara:

```json
{
	"feature": {
		"section": {
			"element": "Tradução"
		}
	}
}
```

### 2. Pluralização

Use sufixos `_plural` para formas plurais:

```json
{
	"items": "{{count}} item",
	"items_plural": "{{count}} items"
}
```

### 3. Namespace

Organize traduções em namespaces quando o projeto crescer:

```typescript
// common.json - textos gerais
// errors.json - mensagens de erro
// validation.json - mensagens de validação
```

### 4. Type Safety

O arquivo `i18next.d.ts` fornece type safety para as chaves de tradução:

```typescript
// TypeScript saberá quais chaves existem
t("app.title") // ✅ OK
t("app.wrongKey") // ❌ Erro de tipo
```

## Clean Code

### Princípios Aplicados

1. **Separação de Responsabilidades**: Traduções separadas da lógica de negócio
2. **DRY (Don't Repeat Yourself)**: Traduções centralizadas e reutilizáveis
3. **Single Source of Truth**: Arquivos JSON como fonte única
4. **Extensibilidade**: Fácil adicionar novos idiomas
5. **Type Safety**: TypeScript garante uso correto das chaves

### Benefícios

- ✅ Manutenção simplificada
- ✅ Escalabilidade
- ✅ Testabilidade
- ✅ Consistência na UI
- ✅ Experiência do usuário localizada

## Performance

### Otimizações Implementadas

- **Code Splitting**: Traduções carregadas sob demanda
- **No Suspense**: Configurado `useSuspense: false` para evitar flickers
- **Cache**: i18next faz cache automático das traduções

## Troubleshooting

### Tradução não aparece

1. Verifique se a chave existe no arquivo JSON
2. Verifique se o namespace está correto
3. Limpe o cache do navegador

### Idioma não muda

1. Verifique se o código do idioma está correto
2. Confirme que o arquivo de tradução existe
3. Verifique o console para erros

### Tipos não funcionam

1. Verifique se `i18next.d.ts` existe
2. Reinicie o TypeScript server (VS Code: Cmd/Ctrl + Shift + P → "TypeScript: Restart TS Server")

## Recursos Adicionais

- [Documentação i18next](https://www.i18next.com/)
- [react-i18next](https://react.i18next.com/)
- [Guia de Pluralização](https://www.i18next.com/translation-function/plurals)
