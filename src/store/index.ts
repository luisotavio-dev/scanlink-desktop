import { create } from "zustand"
import { persist } from "zustand/middleware"

export interface BarcodeItem {
	barcode: string
	timestamp: string
	id: string
}

export interface ConnectionInfo {
	ip: string
	port: number
	token: string
}

export interface QRCodeData {
	qr_base64: string
	connection_info: ConnectionInfo
}

export interface ServerState {
	is_running: boolean
	connected_clients: number
}

export interface AppSettings {
	minimizeToTray: boolean
}

interface AppStore {
	barcodes: BarcodeItem[]
	qrData: QRCodeData | null
	serverState: ServerState
	isLoading: boolean
	error: string | null
	settings: AppSettings

	addBarcode: (barcode: string, timestamp: string) => void
	setQRData: (data: QRCodeData | null) => void
	setServerState: (state: ServerState) => void
	setLoading: (loading: boolean) => void
	setError: (error: string | null) => void
	clearBarcodes: () => void
	updateSettings: (settings: Partial<AppSettings>) => void
}

export const useAppStore = create<AppStore>()(
	persist(
		set => ({
			barcodes: [],
			qrData: null,
			serverState: {
				is_running: false,
				connected_clients: 0,
			},
			isLoading: false,
			error: null,
			settings: {
				minimizeToTray: false,
			},

			addBarcode: (barcode, timestamp) =>
				set(state => ({
					barcodes: [
						{
							barcode,
							timestamp,
							id: `${Date.now()}-${Math.random()}`,
						},
						...state.barcodes,
					],
				})),

			setQRData: data => set({ qrData: data }),

			setServerState: serverState => set({ serverState }),

			setLoading: loading => set({ isLoading: loading }),

			setError: error => set({ error }),

			clearBarcodes: () => set({ barcodes: [] }),

			updateSettings: settings =>
				set(state => ({
					settings: { ...state.settings, ...settings },
				})),
		}),
		{
			name: "scanlink-settings",
			partialize: state => ({ settings: state.settings }),
		},
	),
)
