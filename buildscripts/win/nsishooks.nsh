!macro NSIS_HOOK_POSTINSTALL
  ; Safely append to the Current User PATH if it doesn't already exist
  nsExec::Exec `powershell -NoProfile -Command "$$current = [Environment]::GetEnvironmentVariable('Path', 'User'); if (!$$current.Split(';').Contains('$INSTDIR')) { [Environment]::SetEnvironmentVariable('Path', $$current + ';$INSTDIR', 'User') }"`
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Safely remove from the Current User PATH on uninstall
  nsExec::Exec `powershell -NoProfile -Command "$$current = [Environment]::GetEnvironmentVariable('Path', 'User'); $$paths = $$current.Split(';') | Where-Object { $$_ -ne '$INSTDIR' }; [Environment]::SetEnvironmentVariable('Path', [string]::Join(';', $$paths), 'User')"`
!macroend