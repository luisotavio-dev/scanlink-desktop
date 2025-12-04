import i18n from "i18next"
import { initReactI18next } from "react-i18next"
import enUS from "./locales/en-US/common.json"
import ptBR from "./locales/pt-BR/common.json"

// Detect system language
const getSystemLanguage = (): string => {
	// Get browser/system language
	const systemLang = navigator.language || "en-US"

	// Map common language codes to our supported languages
	if (systemLang.startsWith("pt")) {
		return "pt-BR"
	}

	// Default to English for any other language
	return "en-US"
}

const resources = {
	"en-US": {
		common: enUS,
	},
	"pt-BR": {
		common: ptBR,
	},
}

i18n.use(initReactI18next).init({
	resources,
	lng: getSystemLanguage(),
	fallbackLng: "en-US",
	defaultNS: "common",
	interpolation: {
		escapeValue: false, // React already escapes by default
	},
	react: {
		useSuspense: false,
	},
})

export default i18n
