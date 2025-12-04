import "react-i18next"
import common from "./locales/en-US/common.json"

declare module "react-i18next" {
	interface CustomTypeOptions {
		defaultNS: "common"
		resources: {
			common: typeof common
		}
	}
}
