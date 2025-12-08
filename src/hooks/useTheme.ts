import { useAppStore } from "@/store"
import { useEffect } from "react"

export type Theme = "light" | "dark" | "system"

export function useTheme() {
	const { settings, updateSettings } = useAppStore()
	const theme = settings.theme || "dark"

	useEffect(() => {
		const root = window.document.documentElement

		const applyTheme = (resolvedTheme: "light" | "dark") => {
			root.classList.remove("light", "dark")
			root.classList.add(resolvedTheme)
		}

		if (theme === "system") {
			const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
				.matches
				? "dark"
				: "light"
			applyTheme(systemTheme)

			const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
			const handleChange = (e: MediaQueryListEvent) => {
				applyTheme(e.matches ? "dark" : "light")
			}

			mediaQuery.addEventListener("change", handleChange)
			return () => mediaQuery.removeEventListener("change", handleChange)
		} else {
			applyTheme(theme)
		}
	}, [theme])

	const setTheme = (newTheme: Theme) => {
		updateSettings({ theme: newTheme })
	}

	const resolvedTheme =
		theme === "system"
			? window.matchMedia("(prefers-color-scheme: dark)").matches
				? "dark"
				: "light"
			: theme

	return { theme, setTheme, resolvedTheme }
}
