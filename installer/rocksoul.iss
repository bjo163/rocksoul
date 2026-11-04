#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

#define AppName "Rocksoul"
#define AppPublisher "Rocksoul Project"
#define AppExeName "rocksoul.exe"

[Setup]
AppId={{DAB5FBB4-8F57-4D1D-90CF-0C5015000001}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\Rocksoul
DefaultGroupName=Rocksoul
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Rocksoul-Setup-{#AppVersion}-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
UninstallDisplayIcon={app}\{#AppExeName}
CloseApplications=yes
RestartApplications=no

[Dirs]
Name: "{app}\config"; Permissions: users-modify
Name: "{app}\data"; Permissions: users-modify
Name: "{app}\cache"; Permissions: users-modify
Name: "{app}\workspace"; Permissions: users-modify
Name: "{app}\evolution"; Permissions: users-modify
Name: "{app}\logs"; Permissions: users-modify

[Files]
Source: "..\target\release\rocksoul.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\.env.example"; DestDir: "{app}"; DestName: ".env"; Flags: onlyifdoesntexist uninsneveruninstall; Permissions: users-modify
Source: "..\config\config.example.toml"; DestDir: "{app}\config"; DestName: "config.toml"; Flags: onlyifdoesntexist uninsneveruninstall; Permissions: users-modify
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Rocksoul"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\Rocksoul"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Run]
Filename: "{app}\{#AppExeName}"; Parameters: "init"; Description: "Initialize Rocksoul portable data folders"; Flags: postinstall nowait skipifsilent

; User configuration and runtime data are intentionally preserved on uninstall.
