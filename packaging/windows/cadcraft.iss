; CADCraft for Windows: a single-file setup program built with Inno Setup 6.
;
;   iscc /DAppVersion=0.17.0 /DStageDir=<folder> /DOutputDir=<folder> packaging\windows\cadcraft.iss
;
; StageDir holds cadcraft.exe, cadcraft-cli.exe and the licence files to install (the
; Installers workflow stages them). The result is <OutputDir>\CADCraft-Setup-Windows-x64.exe.
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
AppName=CADCraft
AppVersion={#AppVersion}
AppVerName=CADCraft {#AppVersion}
AppPublisher=CADCraft contributors
AppPublisherURL=https://github.com/arnaved0/cadcraft
AppSupportURL=https://github.com/arnaved0/cadcraft/issues
DefaultDirName={autopf}\CADCraft
DefaultGroupName=CADCraft
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=CADCraft-Setup-Windows-x64
SetupIconFile=..\..\assets\app-icon\cadcraft.ico
UninstallDisplayIcon={app}\cadcraft.exe
UninstallDisplayName=CADCraft
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "assocdxf"; Description: "Open .dxf drawings with CADCraft"; GroupDescription: "File types:"
Name: "assocdwg"; Description: "Open .dwg drawings with CADCraft"; GroupDescription: "File types:"; Flags: unchecked

[Files]
Source: "{#StageDir}\cadcraft.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\cadcraft-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\*"; Excludes: "*.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\CADCraft"; Filename: "{app}\cadcraft.exe"
Name: "{autodesktop}\CADCraft"; Filename: "{app}\cadcraft.exe"; Tasks: desktopicon

[Registry]
; File types, under HKCU or HKLM to match the install (HKA). Removed on uninstall.
Root: HKA; Subkey: "Software\Classes\CADCraft.Drawing"; ValueType: string; ValueName: ""; ValueData: "CADCraft Drawing"; Flags: uninsdeletekey; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\CADCraft.Drawing\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\cadcraft.exe,0"; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\CADCraft.Drawing\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\cadcraft.exe"" ""%1"""; Tasks: assocdxf assocdwg
Root: HKA; Subkey: "Software\Classes\.dxf\OpenWithProgids"; ValueType: string; ValueName: "CADCraft.Drawing"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assocdxf
Root: HKA; Subkey: "Software\Classes\.dwg\OpenWithProgids"; ValueType: string; ValueName: "CADCraft.Drawing"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assocdwg

[Run]
Filename: "{app}\cadcraft.exe"; Description: "{cm:LaunchProgram,CADCraft}"; Flags: nowait postinstall skipifsilent
