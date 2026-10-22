#define AppVersion "0.1.0"
#define Exe GetEnv("LOCALAPPDATA") + "\monitor-uso-target\release\tally.exe"

[Setup]
AppId={{0079ABDF-7825-4119-BF63-83859738B6E5}
AppName=Tally
AppVersion={#AppVersion}
AppVerName=Tally {#AppVersion}
AppPublisher=wllmvg
AppPublisherURL=https://github.com/wllmvg/usage-for-claude
AppSupportURL=https://github.com/wllmvg/usage-for-claude/issues
DefaultDirName={localappdata}\Programs\Tally
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
AppMutex=Local\TallyForClaude
SetupIconFile=tally.ico
UninstallDisplayIcon={app}\tally.ico
UninstallDisplayName=Tally
WizardStyle=modern
WizardImageFile=wizard-1.bmp,wizard-2.bmp
WizardSmallImageFile=small-1.bmp,small-2.bmp
InfoAfterFile=despues.txt
OutputDir=..\dist
OutputBaseFilename=Tally-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "startup"; Description: "Abrir Tally al iniciar Windows"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
Source: "{#Exe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "tally.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Tally"; Filename: "{app}\tally.exe"; IconFilename: "{app}\tally.ico"
Name: "{autodesktop}\Tally"; Filename: "{app}\tally.exe"; IconFilename: "{app}\tally.ico"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Tally"; ValueData: """{app}\tally.exe"""; Tasks: startup
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Tally"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Tally"; Flags: deletevalue; Tasks: not startup

[Run]
Filename: "{app}\tally.exe"; Description: "Abrir Tally ahora"; Flags: postinstall nowait skipifsilent
