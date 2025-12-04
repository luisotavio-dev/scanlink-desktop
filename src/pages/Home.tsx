import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { useAppStore } from '@/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Play, QrCode, Settings, Smartphone, Square, Trash2, Wifi } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

interface BarcodeMessage {
  barcode: string;
  timestamp: string;
  device_id: string;
  device_name?: string;
}

interface ConnectedDevice {
  deviceId: string;
  deviceName: string;
  deviceModel?: string;
  isConnected: boolean;
}

interface HomeProps {
  onOpenSettings: () => void;
}

export default function Home({ onOpenSettings }: HomeProps) {
  const { t } = useTranslation('common');
  const {
    barcodes,
    qrData,
    serverState,
    isLoading,
    error,
    addBarcode,
    setQRData,
    setServerState,
    setLoading,
    setError,
    clearBarcodes,
  } = useAppStore();

  const [statusCheckInterval, setStatusCheckInterval] = useState<number | null>(null);
  const [showDevicesSheet, setShowDevicesSheet] = useState(false);
  const [connectedDevices, setConnectedDevices] = useState<ConnectedDevice[]>([]);
  const [loadingDevices, setLoadingDevices] = useState(false);

  const fetchConnectedDevices = async () => {
    setLoadingDevices(true);
    try {
      const devices = await invoke<ConnectedDevice[]>('get_connected_devices');
      setConnectedDevices(devices);
    } catch (err) {
      console.error('Failed to fetch connected devices:', err);
    } finally {
      setLoadingDevices(false);
    }
  };

  const handleOpenDevicesSheet = () => {
    setShowDevicesSheet(true);
    fetchConnectedDevices();
  };

  const checkServerStatus = async () => {
    try {
      const state = await invoke<{ is_running: boolean; connected_clients: number }>(
        'get_server_state'
      );
      setServerState(state);
    } catch (err) {
      console.error('Failed to check server status:', err);
    }
  };

  useEffect(() => {
    // Listen for barcode events from backend
    const unlistenBarcodePromise = listen<BarcodeMessage>('barcode-received', (event) => {
      console.log('Barcode received:', event.payload);
      addBarcode(event.payload.barcode, event.payload.timestamp);
    });

    // Listen for server-started events to keep QR data in sync
    const unlistenServerPromise = listen<{
      qr_base64: string;
      connection_info: { ip: string; port: number; token: string };
    }>('server-started', (event) => {
      console.log('Server started event received:', event.payload);
      setQRData(event.payload);
    });

    // Check server status periodically
    const interval = window.setInterval(() => {
      checkServerStatus();
    }, 2000);
    setStatusCheckInterval(interval);

    // Initial status check and auto-start server
    const initializeServer = async () => {
      // First check real server state from backend
      const state = await invoke<{ is_running: boolean; connected_clients: number }>(
        'get_server_state'
      );
      setServerState(state);

      if (state.is_running) {
        // Server is already running, get existing QR data instead of starting new server
        console.log('Server already running, fetching existing QR data...');
        try {
          const existingQRData = await invoke<{
            qr_base64: string;
            connection_info: { ip: string; port: number; token: string };
          } | null>('get_current_qr_data');

          if (existingQRData) {
            console.log('Retrieved existing QR data with token:', existingQRData.connection_info.token);
            setQRData(existingQRData);
          }
        } catch (err) {
          console.error('Failed to get existing QR data:', err);
        }
      } else {
        // Server not running, start it
        handleStartServer();
      }
    };
    initializeServer();

    return () => {
      unlistenBarcodePromise.then((unlisten) => unlisten());
      unlistenServerPromise.then((unlisten) => unlisten());
      if (interval) clearInterval(interval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Cleanup interval on unmount
  useEffect(() => {
    return () => {
      if (statusCheckInterval) {
        clearInterval(statusCheckInterval);
      }
    };
  }, [statusCheckInterval]);

  const handleStartServer = async () => {
    setLoading(true);
    setError(null);
    try {
      const qrData = await invoke<{
        qr_base64: string;
        connection_info: { ip: string; port: number; token: string };
      }>('start_server');

      setQRData(qrData);
      await checkServerStatus();
    } catch (err) {
      setError(err as string);
      console.error('Failed to start server:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleStopServer = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke('stop_server');
      setQRData(null);
      await checkServerStatus();
    } catch (err) {
      setError(err as string);
      console.error('Failed to stop server:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  };

  return (
    <div className="h-screen bg-[#0f1419] text-white flex flex-col overflow-hidden">
      {/* Header */}
      <div className="border-b border-slate-800/50 bg-[#1a1f2e]/80 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500 to-blue-600 flex items-center justify-center shadow-lg shadow-blue-500/20">
                <QrCode className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-semibold text-white">
                  {t('app.title')}
                </h1>
                <p className="text-xs text-slate-400">
                  {t('app.subtitle')}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-800/40 border border-slate-700/50">
                {serverState.is_running ? (
                  <>
                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                    <span className="text-xs font-medium text-slate-300">{t('status.online')}</span>
                  </>
                ) : (
                  <>
                    <div className="w-2 h-2 rounded-full bg-slate-500" />
                    <span className="text-xs font-medium text-slate-400">{t('status.offline')}</span>
                  </>
                )}
              </div>
              {serverState.is_running && serverState.connected_clients > 0 && (
                <button
                  onClick={handleOpenDevicesSheet}
                  className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-800/40 border border-slate-700/50 hover:bg-slate-700/50 hover:border-slate-600/50 transition-colors cursor-pointer"
                >
                  <Wifi className="w-3.5 h-3.5 text-blue-400" />
                  <span className="text-xs font-medium text-slate-300">
                    {serverState.connected_clients} {t(`status.clients${serverState.connected_clients === 1 ? '' : '_plural'}`)}
                  </span>
                </button>
              )}
              {serverState.is_running && serverState.connected_clients === 0 && (
                <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-800/40 border border-slate-700/50">
                  <Wifi className="w-3.5 h-3.5 text-slate-500" />
                  <span className="text-xs font-medium text-slate-400">
                    {serverState.connected_clients} {t(`status.clients_plural`)}
                  </span>
                </div>
              )}
              <Button
                variant="ghost"
                size="icon"
                onClick={onOpenSettings}
                className="h-9 w-9 hover:bg-slate-800/60 text-slate-400 hover:text-slate-200 rounded-lg"
              >
                <Settings className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        <div className="max-w-7xl mx-auto px-6 py-6 h-full">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 h-full">
            {/* Left Column - QR Code & Controls */}
            <div className="lg:col-span-1 space-y-4">
              {/* Server Control Card */}
              <Card className="bg-[#1a1f2e]/60 border-slate-800/50 shadow-xl">
                <CardHeader className="pb-3">
                  <CardTitle className="text-base font-semibold text-white">{t('connection.title')}</CardTitle>
                  <CardDescription className="text-xs text-slate-400">
                    {serverState.is_running ? t('connection.serverActive') : t('connection.startToReceive')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {error && (
                    <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-red-400 text-xs">
                      {error}
                    </div>
                  )}

                  {!serverState.is_running ? (
                    <Button
                      onClick={handleStartServer}
                      disabled={isLoading}
                      className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium shadow-lg shadow-blue-600/20"
                      size="default"
                    >
                      <Play className="w-4 h-4 mr-2" />
                      {isLoading ? t('connection.starting') : t('connection.startServer')}
                    </Button>
                  ) : (
                    <Button
                      onClick={handleStopServer}
                      disabled={isLoading}
                      className="w-full bg-slate-700 hover:bg-slate-600 text-white font-medium"
                      size="default"
                    >
                      <Square className="w-4 h-4 mr-2" />
                      {isLoading ? t('connection.stopping') : t('connection.stopServer')}
                    </Button>
                  )}
                </CardContent>
              </Card>

              {/* QR Code Card */}
              {qrData && (
                <Card className="bg-[#1a1f2e]/60 border-slate-800/50 shadow-xl">
                  <CardHeader className="pb-3">
                    <CardTitle className="text-base font-semibold flex items-center gap-2 text-white">
                      <div className="w-6 h-6 rounded bg-blue-500/10 flex items-center justify-center">
                        <QrCode className="w-4 h-4 text-blue-400" />
                      </div>
                      {t('qrCode.title')}
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="relative">
                      <div className="bg-white p-4 rounded-xl">
                        <img
                          src={qrData.qr_base64}
                          alt="QR Code"
                          className="w-full h-auto"
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-xs bg-slate-800/30 px-3 py-2.5 rounded-lg border border-slate-700/30">
                        <span className="text-slate-400">{t('qrCode.ipAddress')}</span>
                        <span className="font-mono text-slate-200 font-medium">{qrData.connection_info.ip}</span>
                      </div>
                      <div className="flex items-center justify-between text-xs bg-slate-800/30 px-3 py-2.5 rounded-lg border border-slate-700/30">
                        <span className="text-slate-400">{t('qrCode.port')}</span>
                        <span className="font-mono text-slate-200 font-medium">{qrData.connection_info.port}</span>
                      </div>
                      <div className="flex flex-col gap-1 text-xs bg-amber-500/10 px-3 py-2.5 rounded-lg border border-amber-500/30">
                        <span className="text-amber-400 font-medium">Token (Debug)</span>
                        <span className="font-mono text-amber-200 text-[10px] break-all">{qrData.connection_info.token}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>

            {/* Right Column - Barcode List */}
            <div className="lg:col-span-2 flex flex-col overflow-hidden">
              <Card className="bg-[#1a1f2e]/60 border-slate-800/50 shadow-xl flex flex-col h-full overflow-hidden">
                <CardHeader className="pb-4 border-b border-slate-800/50">
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-base font-semibold text-white">{t('barcodes.title')}</CardTitle>
                      <CardDescription className="text-xs text-slate-400 mt-1">
                        {t(`barcodes.count${barcodes.length === 1 ? '' : '_plural'}`, { count: barcodes.length })}
                      </CardDescription>
                    </div>
                    {barcodes.length > 0 && (
                      <Button
                        onClick={clearBarcodes}
                        variant="outline"
                        size="sm"
                        className="border-slate-700/50 hover:bg-slate-800/60 text-slate-300 hover:text-white h-8 text-xs"
                      >
                        <Trash2 className="w-3.5 h-3.5 mr-1.5" />
                        {t('barcodes.clear')}
                      </Button>
                    )}
                  </div>
                </CardHeader>
                <CardContent className="flex-1 overflow-hidden flex flex-col p-0">
                  {barcodes.length === 0 ? (
                    <div className="text-center flex-1 flex flex-col items-center justify-center p-8">
                      <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-slate-800/40 mb-4">
                        <QrCode className="w-8 h-8 text-slate-600" />
                      </div>
                      <p className="text-slate-400 text-sm font-medium mb-1">
                        {t('barcodes.empty.title')}
                      </p>
                      <p className="text-slate-600 text-xs">
                        {t('barcodes.empty.subtitle')}
                      </p>
                    </div>
                  ) : (
                    <div className="flex-1 overflow-y-auto custom-scrollbar">
                      <div className="p-4 space-y-2">
                        {barcodes.map((item: { id: string; barcode: string; timestamp: string }, index: number) => (
                          <div
                            key={item.id}
                            className="group relative bg-slate-800/30 hover:bg-slate-800/50 border border-slate-700/30 hover:border-slate-600/50 rounded-lg p-3.5 transition-all duration-150"
                          >
                            <div className="flex items-center gap-3">
                              <div className="flex-shrink-0 w-8 h-8 rounded-lg bg-blue-500/10 border border-blue-500/20 flex items-center justify-center">
                                <span className="text-xs font-semibold text-blue-400">
                                  {barcodes.length - index}
                                </span>
                              </div>
                              <div className="flex-1 min-w-0">
                                <p className="font-mono text-sm font-medium text-slate-200 truncate">
                                  {item.barcode}
                                </p>
                                <p className="text-xs text-slate-500 mt-0.5">
                                  {formatTimestamp(item.timestamp)}
                                </p>
                              </div>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="opacity-0 group-hover:opacity-100 h-7 w-7 p-0 hover:bg-slate-700/50 transition-opacity"
                                onClick={() => navigator.clipboard.writeText(item.barcode)}
                              >
                                <svg
                                  xmlns="http://www.w3.org/2000/svg"
                                  width="14"
                                  height="14"
                                  viewBox="0 0 24 24"
                                  fill="none"
                                  stroke="currentColor"
                                  strokeWidth="2"
                                  strokeLinecap="round"
                                  strokeLinejoin="round"
                                  className="text-slate-400"
                                >
                                  <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                                  <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
                                </svg>
                              </Button>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </div>

      {/* Connected Devices Sheet */}
      <Sheet open={showDevicesSheet} onOpenChange={setShowDevicesSheet}>
        <SheetContent side="right" className="w-full max-w-md">
          <SheetHeader>
            <SheetTitle className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center">
                <Smartphone className="w-4 h-4 text-blue-400" />
              </div>
              {t('devices.title', 'Dispositivos Conectados')}
            </SheetTitle>
            <SheetDescription>
              {t('devices.description', 'Dispositivos atualmente conectados ao servidor')}
            </SheetDescription>
          </SheetHeader>
          
          <div className="px-6 pb-6">
            {loadingDevices ? (
              <div className="flex items-center justify-center py-12">
                <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
              </div>
            ) : connectedDevices.length === 0 ? (
              <div className="text-center py-12">
                <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-slate-800/40 mb-4">
                  <Smartphone className="w-8 h-8 text-slate-600" />
                </div>
                <p className="text-slate-400 text-sm font-medium mb-1">
                  {t('devices.empty.title', 'Nenhum dispositivo conectado')}
                </p>
                <p className="text-slate-600 text-xs">
                  {t('devices.empty.subtitle', 'Escaneie o QR Code com seu celular')}
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {connectedDevices.map((device) => (
                  <div
                    key={device.deviceId}
                    className="bg-slate-800/30 border border-slate-700/30 rounded-xl p-4 hover:bg-slate-800/50 transition-colors"
                  >
                    <div className="flex items-start gap-3">
                      <div className="flex-shrink-0 w-10 h-10 rounded-xl bg-green-500/10 border border-green-500/20 flex items-center justify-center">
                        <Smartphone className="w-5 h-5 text-green-400" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <p className="font-medium text-slate-200 truncate">
                            {device.deviceName}
                          </p>
                          <div className="flex items-center gap-1">
                            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                            <span className="text-xs text-green-400">
                              {t('devices.status.connected', 'Conectado')}
                            </span>
                          </div>
                        </div>
                        {device.deviceModel && (
                          <p className="text-xs text-slate-500 mt-0.5">
                            {device.deviceModel}
                          </p>
                        )}
                        <p className="text-xs text-slate-600 mt-1 font-mono truncate">
                          ID: {device.deviceId.substring(0, 8)}...
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
            
            {connectedDevices.length > 0 && (
              <div className="mt-6 pt-4 border-t border-slate-800/50">
                <Button
                  onClick={fetchConnectedDevices}
                  variant="outline"
                  size="sm"
                  className="w-full border-slate-700/50 hover:bg-slate-800/60 text-slate-300 hover:text-white"
                >
                  {t('devices.refresh', 'Atualizar lista')}
                </Button>
              </div>
            )}
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
