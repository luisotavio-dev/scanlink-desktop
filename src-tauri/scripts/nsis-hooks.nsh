; ScanLink NSIS Installer Hooks
; Configures Windows Firewall to allow port 47592

!define APP_NAME "ScanLink"
!define APP_PORT "47592"

; Hook executed after installation
!macro NSIS_HOOK_POSTINSTALL
    ; Add Windows Firewall rule for inbound connections
    DetailPrint "Configuring Windows Firewall for ${APP_NAME}..."

    ; Remove existing rule if any (to avoid duplicates)
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${APP_NAME} WebSocket Server"'

    ; Add new firewall rule for TCP port
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${APP_NAME} WebSocket Server" dir=in action=allow protocol=TCP localport=${APP_PORT} profile=private,public description="Allow ${APP_NAME} to receive connections from mobile devices"'

    Pop $0
    ${If} $0 == 0
        DetailPrint "Windows Firewall configured successfully"
    ${Else}
        DetailPrint "Warning: Could not configure Windows Firewall (error: $0)"
    ${EndIf}
!macroend

; Hook executed before uninstallation
!macro NSIS_HOOK_PREUNINSTALL
    ; Remove Windows Firewall rule
    DetailPrint "Removing Windows Firewall rule for ${APP_NAME}..."
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${APP_NAME} WebSocket Server"'

    Pop $0
    ${If} $0 == 0
        DetailPrint "Windows Firewall rule removed successfully"
    ${Else}
        DetailPrint "Warning: Could not remove Windows Firewall rule (error: $0)"
    ${EndIf}
!macroend

; Empty hooks for completeness
!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
!macroend
