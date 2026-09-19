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
!macroend

