; Inno Setup 6. Defines are passed by CI (/D...): AppName AppExeName AppVersion AppPublisher SourceExe OutputName

[Setup]
; Generate your own GUID once and never change it
AppId={{B3E5A1C2-7D44-4F0A-8E19-5C6D2A9F1B07}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE.txt
SetupIconFile=..\..\icons\icon.ico
UninstallDisplayIcon={app}\{#AppExeName}
OutputDir=..\..\dist
OutputBaseFilename={#OutputName}
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
ChangesEnvironment=yes
CloseApplications=yes

[Tasks]
Name: "addtopath"; Description: "Add {#AppName} to PATH (use it from the command line)"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a &desktop shortcut"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: IsAdminInstallMode and NeedsAddPath
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: (not IsAdminInstallMode) and NeedsAddPath

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
; In-app updates run this installer with /SILENT: relaunch afterwards
Filename: "{app}\{#AppExeName}"; Flags: nowait; Check: WizardSilent

[Code]
const
  SysEnvKey  = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
  UserEnvKey = 'Environment';

function PathContains(Root: Integer; Key, Dir: string): Boolean;
var
  Paths: string;
begin
  if not RegQueryStringValue(Root, Key, 'Path', Paths) then
  begin
    Result := False;
    exit;
  end;
  Result := Pos(';' + Lowercase(Dir) + ';', ';' + Lowercase(Paths) + ';') > 0;
end;

function NeedsAddPath: Boolean;
begin
  if IsAdminInstallMode then
    Result := not PathContains(HKLM, SysEnvKey, ExpandConstant('{app}'))
  else
    Result := not PathContains(HKCU, UserEnvKey, ExpandConstant('{app}'));
end;

procedure RemoveFromPath(Root: Integer; Key: string);
var
  Paths, Dir: string;
  P: Integer;
begin
  Dir := ExpandConstant('{app}');
  if not RegQueryStringValue(Root, Key, 'Path', Paths) then exit;
  P := Pos(';' + Lowercase(Dir), Lowercase(Paths));
  if P = 0 then exit;
  Delete(Paths, P, Length(Dir) + 1);
  RegWriteExpandStringValue(Root, Key, 'Path', Paths);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if IsAdminInstallMode then RemoveFromPath(HKLM, SysEnvKey)
    else RemoveFromPath(HKCU, UserEnvKey);
  end;
end;
