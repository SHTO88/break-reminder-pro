; Break Reminder Pro - NSIS Installer Lifecycle Hooks
; Injected into Tauri's default NSIS installer template

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Closing running instances of Break Reminder Pro..."
  nsExec::Exec 'taskkill /F /IM "Break Reminder Pro.exe"'
  nsExec::Exec 'taskkill /F /IM "break-reminder-pro-app.exe"'
  Sleep 500
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Closing running instances of Break Reminder Pro..."
  nsExec::Exec 'taskkill /F /IM "Break Reminder Pro.exe"'
  nsExec::Exec 'taskkill /F /IM "break-reminder-pro-app.exe"'
  Sleep 500

  ; Backup Autostart registry key so it can be preserved across upgrade uninstalls
  ReadRegStr $R9 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; If app data was preserved (user did NOT check 'Delete app data'), restore the autostart key
  ${If} $DeleteAppDataCheckboxState = 0
  ${AndIf} $R9 != ""
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}" "$R9"
  ${EndIf}
!macroend

