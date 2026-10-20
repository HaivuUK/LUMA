!define APP_NAME "Luma"
!define EXE_NAME "luma.exe"

OutFile "..\..\dist\luma_${VERSION}_x64-setup.exe"
InstallDir "$PROGRAMFILES64\Luma"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Install"
    SetOutPath "$INSTDIR"
    File "..\..\target\x86_64-pc-windows-msvc\release\luma.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"

    # Add installation directory to System PATH
    ExecWait 'cmd.exe /c setx /M PATH "%PATH%;$INSTDIR"'
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\luma.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
SectionEnd