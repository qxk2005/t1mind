; AppFlowy Windows Installer Script
; Generated for AppFlowy Desktop Application

#define AppName "AppFlowy"
#define AppPublisher "AppFlowy-IO"
#define AppURL "https://appflowy.io"
#define AppExeName "AppFlowy.exe"

; Read version from pubspec.yaml
#define AppVersion GetEnv('APP_VERSION')
#if AppVersion == ""
#define AppVersion "0.9.20251022"
#endif

[Setup]
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes
LicenseFile=
OutputDir=..\..\appflowy_flutter\product\{#AppVersion}\windows
OutputBaseFilename=AppFlowy-Setup-{#AppVersion}
SetupIconFile=flowy_logo.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
MinVersion=6.1sp1

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[CustomMessages]
chinesesimplified.InstallRuntime=安装必要的运行库
chinesesimplified.RuntimeRequired=AppFlowy 需要以下运行库才能正常运行：
chinesesimplified.VCRedistRequired=Microsoft Visual C++ 2015-2022 Redistributable (x64)
chinesesimplified.DownloadRuntime=下载并安装运行库
chinesesimplified.SkipRuntime=跳过运行库安装（不推荐）
chinesesimplified.RuntimeNote=注意：如果跳过运行库安装，应用程序可能无法正常启动。
chinesesimplified.DownloadFailed=下载失败，请手动下载运行库：
chinesesimplified.VCRedistURL=https://aka.ms/vs/17/release/vc_redist.x64.exe

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1; Check: not IsAdminInstallMode
Name: "installruntime"; Description: "自动安装 Microsoft Visual C++ Redistributable（推荐）"; GroupDescription: "运行库安装"; Flags: checkedonce
Name: "skipruntime"; Description: "跳过运行库安装（不推荐）"; GroupDescription: "运行库安装"; Flags: unchecked; Check: WizardSilent

[Files]
Source: "..\..\appflowy_flutter\build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\appflowy_flutter\build\windows\x64\runner\Release\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "runtime\vc_redist.x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: quicklaunchicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Registry]
Root: HKCR; Subkey: "AppFlowy"; ValueType: "string"; ValueData: "URL:Custom Protocol"; Flags: uninsdeletekey
Root: HKCR; Subkey: "AppFlowy"; ValueType: "string"; ValueName: "URL Protocol"; ValueData: ""
Root: HKCR; Subkey: "AppFlowy\DefaultIcon"; ValueType: "string"; ValueData: "{app}\{#AppExeName},0"
Root: HKCR; Subkey: "AppFlowy\shell\open\command"; ValueType: "string"; ValueData: """{app}\{#AppExeName}"" ""%1"""

[UninstallDelete]
Type: filesandordirs; Name: "{app}\data"

[Code]
// 检测Visual C++ Redistributable是否已安装
function IsVCRedistInstalled: Boolean;
var
  Version: String;
begin
  Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Version', Version);
  if not Result then
    Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SOFTWARE\Microsoft\VisualStudio\15.0\VC\Runtimes\x64', 'Version', Version);
  if not Result then
    Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SOFTWARE\Microsoft\VisualStudio\16.0\VC\Runtimes\x64', 'Version', Version);
  if not Result then
    Result := RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SOFTWARE\Microsoft\VisualStudio\17.0\VC\Runtimes\x64', 'Version', Version);
end;

// 安装Visual C++ Redistributable
function InstallVCRedist: Boolean;
var
  VCRedistPath: String;
  ResultCode: Integer;
begin
  Result := False;
  VCRedistPath := ExpandConstant('{tmp}\vc_redist.x64.exe');
  
  if FileExists(VCRedistPath) then
  begin
    Log('Installing Visual C++ Redistributable from: ' + VCRedistPath);
    
    if Exec(VCRedistPath, '/quiet /norestart', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
    begin
      Result := (ResultCode = 0);
      if Result then
        Log('Visual C++ Redistributable installed successfully')
      else
        Log('Visual C++ Redistributable installation failed with code: ' + IntToStr(ResultCode));
    end
    else
    begin
      Log('Failed to execute Visual C++ Redistributable installer');
    end;
  end
  else
  begin
    Log('Visual C++ Redistributable file not found: ' + VCRedistPath);
  end;
end;

// 在安装前检查运行库
function NextButtonClick(CurPageID: Integer): Boolean;
var
  VCRedistInstalled: Boolean;
begin
  Result := True;
  
  if CurPageID = wpSelectTasks then
  begin
    VCRedistInstalled := IsVCRedistInstalled;
    
    if IsTaskSelected('installruntime') and not VCRedistInstalled then
    begin
      if MsgBox('检测到系统中未安装 Microsoft Visual C++ Redistributable。' + #13#10 + #13#10 +
                'AppFlowy 需要此运行库才能正常工作。' + #13#10 + #13#10 +
                '安装程序将自动为您安装运行库。' + #13#10 + #13#10 +
                '是否继续安装？', mbConfirmation, MB_YESNO) = IDNO then
      begin
        Result := False;
      end;
    end;
  end;
end;

// 在安装完成后自动安装运行库
procedure CurStepChanged(CurStep: TSetupStep);
var
  VCRedistInstalled: Boolean;
begin
  if CurStep = ssPostInstall then
  begin
    VCRedistInstalled := IsVCRedistInstalled;
    
    if IsTaskSelected('installruntime') and not VCRedistInstalled then
    begin
      Log('Starting automatic Visual C++ Redistributable installation...');
      
      if InstallVCRedist then
      begin
        Log('Visual C++ Redistributable installation completed successfully');
        MsgBox('Microsoft Visual C++ Redistributable 已成功安装！', mbInformation, MB_OK);
      end
      else
      begin
        Log('Visual C++ Redistributable installation failed');
        MsgBox('Microsoft Visual C++ Redistributable 安装失败。' + #13#10 + #13#10 +
               '请手动下载并安装运行库：' + #13#10 +
               'https://aka.ms/vs/17/release/vc_redist.x64.exe', mbError, MB_OK);
      end;
    end;
  end;
end;
