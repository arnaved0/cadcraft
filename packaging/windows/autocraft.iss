; AutoCraft for Windows: a single-file setup program built with Inno Setup 6.
;
;   iscc /DAppVersion=0.17.0 /DStageDir=<folder> /DOutputDir=<folder> packaging\windows\autocraft.iss
;
; StageDir holds autocraft.exe, autocraft-cli.exe and the licence files to install (the
; Installers workflow stages them). The result is <OutputDir>\AutoCraft-Setup-Windows-x64.exe.
; Installs for the current user without administrator rights by default; the first page offers
; installing for all users instead.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef StageDir
  #define StageDir "..\..\dist\windows-stage"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif

[Setup]
AppId={{5C7E1C9B-3D0A-4F55-9B7E-2E61A6D0C8F4}
AppName=AutoCraft
AppVersion={#AppVersion}
AppVerName=AutoCraft {#AppVersion}
AppPublisher=AutoCraft contributors
AppPublisherURL=https://github.com/arnaved0/autocraft
AppSupportURL=https://github.com/arnaved0/autocraft/issues
DefaultDirName={autopf}\AutoCraft
DefaultGroupName=AutoCraft
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=AutoCraft-Setup-Windows-x64
SetupIconFile=..\..\assets\app-icon\autocraft.ico
UninstallDisplayIcon={app}\autocraft.exe
UninstallDisplayName=AutoCraft
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "assocdxf"; Description: "Open .dxf drawings with AutoCraft"; GroupDescription: "File types:"
Name: "assocdwg"; Description: "Open .dwg drawings with AutoCraft"; GroupDescription: "File types:"; Flags: unchecked

[Files]
Source: "{#StageDir}\autocraft.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\autocraft-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\*"; Excludes: "*.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\AutoCraft"; Filename: "{app}\autocraft.exe"
Name: "{autodesktop}\AutoCraft"; Filename: "{app}\autocraft.exe"; Tasks: desktopicon

[Registry]
; File types, under HKCU or HKLM to match the install (HKA). Removed on uninstall.
Root: HKA; Subkey: "Software\Classes\AutoCraft.Drawing"; ValueType: string; ValueName: ""; ValueData: "AutoCraft Drawing"; Flags: uninsdeletekey; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\AutoCraft.Drawing\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\autocraft.exe,0"; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\AutoCraft.Drawing\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\autocraft.exe"" ""%1"""; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\.dxf\OpenWithProgids"; ValueType: string; ValueName: "AutoCraft.Drawing"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assocdxf
Root: HKA; Subkey: "Software\Classes\.dwg\OpenWithProgids"; ValueType: string; ValueName: "AutoCraft.Drawing"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assocdwg

[Run]
Filename: "{app}\autocraft.exe"; Description: "{cm:LaunchProgram,AutoCraft}"; Flags: nowait postinstall skipifsilent
