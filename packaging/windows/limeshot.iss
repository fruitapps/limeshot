; LimeShot Windows installer (PF-006). Built by scripts/package-windows.ps1, which stages the files and
; passes the defines below:
;   ISCC /DAppVersion=0.1.0-alpha.1 /DNumericVersion=0.1.0 /DStage=dist\windows\LimeShot
;        /DOutputDir=dist /DOutputBase=LimeShot-0.1.0-alpha.1-windows-x64-setup packaging\windows\limeshot.iss
;
; Components:
;   * LimeShot (required): limeshot.exe, limeshot-cli.exe, bundled samples, icon and licences.
;   * Lime Native VST3 suite (default on): the twelve Lime VST3 bundles, LimeShot's own instruments and effects,
;     installed into the standard VST3 folder so other hosts can load them too. LimeShot finds them there.
;     Unticked, the bundles go into the app's private "plugins" folder instead: LimeShot needs them either way.
; Installs per machine by default (Program Files + Common Files\VST3); "Install for me only" uses
; %LOCALAPPDATA%\Programs and %LOCALAPPDATA%\Programs\Common\VST3, both of which LimeShot also searches.

#ifndef AppVersion
  #error Pass /DAppVersion=... (scripts/package-windows.ps1 does)
#endif
#ifndef NumericVersion
  #define NumericVersion "0.0.0"
#endif
#ifndef Stage
  #error Pass /DStage=<staged LimeShot folder>
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif
#ifndef OutputBase
  #define OutputBase "LimeShot-" + AppVersion + "-windows-x64-setup"
#endif

[Setup]
AppId={{6E1B7C3A-4F2D-4C59-9C1E-5A3D2B8F0C1D}
AppName=LimeShot
AppVersion={#AppVersion}
AppVerName=LimeShot {#AppVersion}
AppPublisher=LimeShot contributors
AppPublisherURL=https://github.com/fruitapps/limeshot
AppSupportURL=https://github.com/fruitapps/limeshot/issues
VersionInfoVersion={#NumericVersion}
VersionInfoProductVersion={#NumericVersion}
VersionInfoProductTextVersion={#AppVersion}
DefaultDirName={autopf}\LimeShot
DefaultGroupName=LimeShot
DisableProgramGroupPage=yes
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog commandline
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
LicenseFile={#Stage}\LICENSE
SetupIconFile={#Stage}\LimeShot.ico
UninstallDisplayIcon={app}\LimeShot.ico
UninstallDisplayName=LimeShot {#AppVersion}
ChangesAssociations=yes
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBase}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Types]
Name: "full"; Description: "LimeShot and the Lime VST3 suite for all hosts"
Name: "app"; Description: "LimeShot only (Lime VST3s private to LimeShot)"
Name: "custom"; Description: "Custom"; Flags: iscustom

[Components]
Name: "main"; Description: "LimeShot"; Types: full app custom; Flags: fixed
Name: "vst3"; Description: "Lime Native VST3 suite in the shared VST3 folder (usable by other DAWs)"; Types: full

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "assoc"; Description: "Open .lime projects with LimeShot"; GroupDescription: "File associations:"

[Files]
Source: "{#Stage}\limeshot.exe"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\limeshot-cli.exe"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\LimeShot.ico"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\LICENSE"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\LICENSE-MIT"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\LICENSE-APACHE"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\ATTRIBUTION.md"; DestDir: "{app}"; Components: main; Flags: ignoreversion
Source: "{#Stage}\README.txt"; DestDir: "{app}"; Components: main; Flags: ignoreversion isreadme
Source: "{#Stage}\samples\*"; DestDir: "{app}\samples"; Components: main; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#Stage}\licenses\*"; DestDir: "{app}\licenses"; Components: main; Flags: ignoreversion recursesubdirs createallsubdirs
; The Lime VST3 bundles: shared VST3 folder, or the app's private folder when the component is unticked.
Source: "{#Stage}\plugins\*"; DestDir: "{autocf64}\VST3"; Components: vst3; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#Stage}\plugins\*"; DestDir: "{app}\plugins"; Components: not vst3; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\LimeShot"; Filename: "{app}\limeshot.exe"; IconFilename: "{app}\LimeShot.ico"
Name: "{autodesktop}\LimeShot"; Filename: "{app}\limeshot.exe"; IconFilename: "{app}\LimeShot.ico"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.lime\OpenWithProgids"; ValueType: string; ValueName: "LimeShot.Project"; ValueData: ""; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LimeShot.Project"; ValueType: string; ValueName: ""; ValueData: "LimeShot project"; Flags: uninsdeletekey; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LimeShot.Project\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\LimeShot.ico,0"; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\LimeShot.Project\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\limeshot.exe"" ""%1"""; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\Applications\limeshot.exe\SupportedTypes"; ValueType: string; ValueName: ".lime"; ValueData: ""; Flags: uninsdeletekey; Tasks: assoc

[Run]
Filename: "{app}\limeshot.exe"; Description: "{cm:LaunchProgram,LimeShot}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Folders the bundles were copied into (Inno removes the files; this removes the bundle folders themselves).
#define Bundle(Name) \
  "Type: filesandordirs; Name: ""{autocf64}\VST3\" + Name + ".vst3""; Components: vst3"
{#Bundle("LimeSynth")}
{#Bundle("Lime Sampler")}
{#Bundle("Lime Utility")}
{#Bundle("Lime EQ")}
{#Bundle("Lime Filter")}
{#Bundle("Lime Delay")}
{#Bundle("Lime Room")}
{#Bundle("Lime Comp")}
{#Bundle("Lime Limiter")}
{#Bundle("Lime Gate")}
{#Bundle("Lime Drive")}
{#Bundle("Lime Chorus")}
Type: filesandordirs; Name: "{app}\plugins"
