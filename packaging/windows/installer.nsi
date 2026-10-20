!include "MUI2.nsh"
!include "x64.nsh"

!define APP_NAME "LUMA"
!define COMPANY_NAME "LUMA"
!define EXE_NAME "luma.exe"

Name "${APP_NAME}"
OutFile "..\..\dist\luma_${VERSION}_x64-setup.exe"
InstallDir "$PROGRAMFILES64\Luma"
InstallDirRegKey HKLM "Software\${APP_NAME}" "InstallDir"
RequestExecutionLevel admin

; Installer / Uninstaller Icons
!define MUI_ICON "..\..\icons\icon.ico"
!define MUI_UNICON "..\..\icons\icon.ico"
!define MUI_ABORTWARNING

; Modern UI Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "$INSTDIR"
    File "..\..\target\x86_64-pc-windows-msvc\release\luma.exe"

    ; Create Uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Start Menu Shortcuts (App menu entry)
    CreateDirectory "$SMPROGRAMS\Luma"
    CreateShortcut "$SMPROGRAMS\Luma\Luma.lnk" "$INSTDIR\luma.exe" "" "$INSTDIR\luma.exe" 0
    CreateShortcut "$SMPROGRAMS\Luma\Uninstall Luma.lnk" "$INSTDIR\uninstall.exe"

    ; Add/Remove Programs Registry Keys
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma" "DisplayName" "LUMA"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma" "DisplayIcon" "$INSTDIR\luma.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma" "Publisher" "${COMPANY_NAME}"

    ; Add installation directory to System PATH
    ExecWait 'cmd.exe /c setx /M PATH "%PATH%;$INSTDIR"'
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\luma.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\Luma\Luma.lnk"
    Delete "$SMPROGRAMS\Luma\Uninstall Luma.lnk"
    RMDir "$SMPROGRAMS\Luma"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma"
    DeleteRegKey HKLM "Software\${APP_NAME}"
SectionEnd