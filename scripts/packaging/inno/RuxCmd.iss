; RuxCmd standalone installer (Inno Setup 7.x; pinned 7.1.1).
;
; The packaging script copies this file into a temporary build directory and
; generates build-info.iss and generated-files.iss next to it. The [Files]
; list is derived from the release ZIP manifest, never maintained by hand.
;
; AppId must stay stable across every release: it is the upgrade identity.
; The matching GUID is also used by scripts/install.ps1 for mutual exclusion.

#include "build-info.iss"

[Setup]
AppId={{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}
AppName=RuxCmd
AppVersion={#AppVersion}
AppVerName=RuxCmd {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL=https://github.com/wangdxnum1/RuxCmd
AppSupportURL=https://github.com/wangdxnum1/RuxCmd
AppUpdatesURL=https://github.com/wangdxnum1/RuxCmd/releases
AppCopyright={#AppCopyright}
DefaultDirName={code:GetDefaultDir}
DisableDirPage=auto
DisableProgramGroupPage=yes
UsePreviousAppDir=yes
UsePreviousPrivileges=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
AllowNoIcons=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
Compression=lzma2/max
SolidCompression=yes
ChangesEnvironment=yes
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
UninstallDisplayName=RuxCmd
UninstallDisplayIcon={app}\find.exe
VersionInfoVersion={#VersionInfoVersion}
VersionInfoProductVersion={#AppVersion}
VersionInfoProductName={#AppProductName}
VersionInfoCompany={#AppPublisher}
VersionInfoDescription=RuxCmd command suite installer
VersionInfoCopyright={#AppCopyright}
VersionInfoOriginalFileName={#OutputBaseFilename}.exe
WizardStyle=modern

[Types]
Name: "full"; Description: "Full installation"
Name: "compact"; Description: "Command line only"; Flags: iscustom

[Components]
Name: "desktop"; Description: "RuxCmd Desktop (UI)"; Types: full

[InstallDelete]
; Remove shims from older versions before laying down the new set. The share
; directory is installer-owned documentation; reinstall it cleanly.
Type: files; Name: "{app}\rux-*.cmd"
Type: filesandordirs; Name: "{app}\share"
; The desktop UI lives one level above {app} (the PATH directory), so clean
; up the copy older versions left inside bin\ as well.
Type: files; Name: "{app}\ruxcmd-desktop.exe"
Type: files; Name: "{app}\..\ruxcmd-desktop.exe"

[UninstallDelete]
; Stale shims from versions installed before the current one are not in the
; current install log, so remove them explicitly. The uninstaller also
; removes every file it recorded.
Type: files; Name: "{app}\rux-*.cmd"
; The desktop UI lives one level above {app} (the PATH directory), so clean
; up the copy older versions left inside bin\ as well.
Type: files; Name: "{app}\ruxcmd-desktop.exe"
Type: files; Name: "{app}\..\ruxcmd-desktop.exe"
Type: filesandordirs; Name: "{app}\share"
Type: dirifempty; Name: "{app}"

#include "generated-files.iss"

[Code]
const
  AppIdValue = '{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}';
  UserEnvironmentKey = 'Environment';
  MachineEnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

function UserUninstallKey: string;
begin
  Result := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\' + AppIdValue + '_is1';
end;

function TrimTrailingBackslash(const Value: string): string;
begin
  Result := Value;
  while (Length(Result) > 0) and (Result[Length(Result)] = '\') do
    Delete(Result, Length(Result), 1);
end;

function NormalizePathEntry(const Value: string): string;
var
  Entry: string;
begin
  Entry := Trim(Value);
  if (Length(Entry) >= 2) and (Entry[1] = '"') and (Entry[Length(Entry)] = '"') then
    Entry := Copy(Entry, 2, Length(Entry) - 2);
  Result := TrimTrailingBackslash(Entry);
end;

function SamePathEntry(const Left, Right: string): Boolean;
begin
  Result := CompareText(NormalizePathEntry(Left), NormalizePathEntry(Right)) = 0;
end;

function PathContains(const PathValue, Entry: string): Boolean;
var
  Rest, Part: string;
  Separator: Integer;
begin
  Result := False;
  Rest := PathValue;
  while Rest <> '' do
  begin
    Separator := Pos(';', Rest);
    if Separator = 0 then
    begin
      Part := Rest;
      Rest := '';
    end
    else
    begin
      Part := Copy(Rest, 1, Separator - 1);
      Rest := Copy(Rest, Separator + 1, Length(Rest) - Separator);
    end;
    if SamePathEntry(Part, Entry) then
    begin
      Result := True;
      Exit;
    end;
  end;
end;

function RemovePathEntry(const PathValue, Entry: string): string;
var
  Rest, Part: string;
  Separator: Integer;
begin
  Result := '';
  Rest := PathValue;
  while Rest <> '' do
  begin
    Separator := Pos(';', Rest);
    if Separator = 0 then
    begin
      Part := Rest;
      Rest := '';
    end
    else
    begin
      Part := Copy(Rest, 1, Separator - 1);
      Rest := Copy(Rest, Separator + 1, Length(Rest) - Separator);
    end;
    if (Part <> '') and not SamePathEntry(Part, Entry) then
    begin
      if Result <> '' then
        Result := Result + ';';
      Result := Result + Part;
    end;
  end;
end;

procedure AddPathEntry;
var
  RootKey: Integer;
  SubKey, Current, Updated: string;
begin
  if IsAdminInstallMode then
  begin
    RootKey := HKLM;
    SubKey := MachineEnvironmentKey;
  end
  else
  begin
    RootKey := HKCU;
    SubKey := UserEnvironmentKey;
  end;

  if not RegQueryStringValue(RootKey, SubKey, 'Path', Current) then
    Current := '';
  if PathContains(Current, ExpandConstant('{app}')) then
    Exit;

  if Current = '' then
    Updated := ExpandConstant('{app}')
  else
    Updated := Current + ';' + ExpandConstant('{app}');
  RegWriteExpandStringValue(RootKey, SubKey, 'Path', Updated);
end;

procedure RemovePathEntryFromEnvironment;
var
  RootKey: Integer;
  SubKey, Current, Updated: string;
begin
  if IsAdminInstallMode then
  begin
    RootKey := HKLM;
    SubKey := MachineEnvironmentKey;
  end
  else
  begin
    RootKey := HKCU;
    SubKey := UserEnvironmentKey;
  end;

  if not RegQueryStringValue(RootKey, SubKey, 'Path', Current) then
    Exit;
  Updated := RemovePathEntry(Current, ExpandConstant('{app}'));
  if Updated = Current then
    Exit;
  if Updated = '' then
    RegDeleteValue(RootKey, SubKey, 'Path')
  else
    RegWriteExpandStringValue(RootKey, SubKey, 'Path', Updated);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    AddPathEntry;
end;

function GetVersionPart(const Value: string; Index: Integer): Integer;
var
  Position, Start, PartIndex: Integer;
begin
  Result := 0;
  Start := 1;
  PartIndex := 1;
  for Position := 1 to Length(Value) do
  begin
    if Value[Position] = '.' then
    begin
      if PartIndex = Index then
      begin
        Result := StrToIntDef(Copy(Value, Start, Position - Start), 0);
        Exit;
      end;
      Inc(PartIndex);
      Start := Position + 1;
    end;
  end;
  if PartIndex = Index then
    Result := StrToIntDef(Copy(Value, Start, Length(Value) - Start + 1), 0);
end;

function IsNewerVersion(const Candidate, Current: string): Boolean;
var
  Index, Left, Right: Integer;
begin
  Result := False;
  for Index := 1 to 3 do
  begin
    Left := GetVersionPart(Candidate, Index);
    Right := GetVersionPart(Current, Index);
    if Left <> Right then
    begin
      Result := Left > Right;
      Exit;
    end;
  end;
end;

function FindInstalledVersion: string;
var
  UserVersion, MachineVersion: string;
begin
  Result := '';
  if RegQueryStringValue(HKCU, UserUninstallKey, 'DisplayVersion', UserVersion) then
    Result := UserVersion;
  if RegQueryStringValue(HKLM, UserUninstallKey, 'DisplayVersion', MachineVersion) then
    if (Result = '') or IsNewerVersion(MachineVersion, Result) then
      Result := MachineVersion;
end;

function ReadPreviousAppDir(RootKey: Integer): string;
begin
  Result := '';
  { Inno Setup 7 renamed the value to "Inno Setup: App Path"; accept both. }
  if RegQueryStringValue(RootKey, UserUninstallKey, 'Inno Setup: App Path', Result) and (Result <> '') then
    Exit;
  RegQueryStringValue(RootKey, UserUninstallKey, 'Inno Setup: AppPath', Result);
end;

function GetPreviousAppDir: string;
begin
  Result := ReadPreviousAppDir(HKCU);
  if Result = '' then
    Result := ReadPreviousAppDir(HKLM);
end;

function StartsWithPath(const Value, Prefix: string): Boolean;
begin
  Result := CompareText(Copy(Value, 1, Length(Prefix)), Prefix) = 0;
  if Result and (Length(Value) > Length(Prefix)) and (Value[Length(Prefix) + 1] <> '\') then
    Result := False;
end;

function IsProtectedDirectory(const Directory: string): Boolean;
begin
  Result :=
    StartsWithPath(Directory, ExpandConstant('{commonpf}')) or
    StartsWithPath(Directory, ExpandConstant('{commonpf32}')) or
    StartsWithPath(Directory, ExpandConstant('{win}')) or
    StartsWithPath(Directory, ExpandConstant('{sys}'));
end;

function IsDriveRoot(const Directory: string): Boolean;
var
  Trimmed: string;
begin
  Trimmed := TrimTrailingBackslash(Directory);
  Result := (Length(Trimmed) = 2) and (Trimmed[2] = ':');
end;

function IsValidInstallDirectory(const Directory: string): Boolean;
begin
  Result :=
    (Directory <> '') and
    (Pos(';', Directory) = 0) and
    (Pos('"', Directory) = 0) and
    (Pos('%', Directory) = 0) and
    (Pos(#13, Directory) = 0) and
    (Pos(#10, Directory) = 0) and
    not IsDriveRoot(Directory);
end;

function FindScriptInstallInUserPath: string;
var
  PathValue, Rest, Part: string;
  Separator: Integer;
begin
  Result := '';
  if not RegQueryStringValue(HKCU, UserEnvironmentKey, 'Path', PathValue) then
    Exit;
  Rest := PathValue;
  while Rest <> '' do
  begin
    Separator := Pos(';', Rest);
    if Separator = 0 then
    begin
      Part := Rest;
      Rest := '';
    end
    else
    begin
      Part := Copy(Rest, 1, Separator - 1);
      Rest := Copy(Rest, Separator + 1, Length(Rest) - Separator);
    end;
    Part := Trim(Part);
    if (Part <> '') and FileExists(AddBackslash(Part) + '.ruxcmd-install.json') then
    begin
      Result := Part;
      Exit;
    end;
  end;
end;

function GetDefaultDir(Param: string): string;
begin
  if IsAdminInstallMode then
    Result := ExpandConstant('{commonpf}\RuxCmd\bin')
  else
    Result := ExpandConstant('{localappdata}\RuxCmd\bin');
end;

function InitializeSetup: Boolean;
var
  InstalledVersion: string;
begin
  Result := True;
  InstalledVersion := FindInstalledVersion;
  if InstalledVersion <> '' then
  begin
    if IsNewerVersion(InstalledVersion, '{#AppVersion}') then
    begin
      MsgBox('RuxCmd ' + InstalledVersion + ' is already installed. This installer only upgrades to ' +
             '{#AppVersion}' + ' or reinstalls the same version. Uninstall the newer version first.',
             mbError, MB_OK);
      Result := False;
    end;
  end;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Directory: string;
begin
  Result := True;
  if CurPageID = wpSelectDir then
  begin
    Directory := WizardForm.DirEdit.Text;
    if not IsValidInstallDirectory(Directory) then
    begin
      MsgBox('The installation directory must not contain semicolons, quotes, percent signs, or line breaks, and must not be a drive root.',
             mbError, MB_OK);
      Result := False;
    end
    else if (not IsAdminInstallMode) and IsProtectedDirectory(Directory) then
    begin
      MsgBox('Installing into Program Files or Windows requires the "Install for all users" mode. Choose that mode, or select a directory under your user profile.',
             mbError, MB_OK);
      Result := False;
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Directory, PreviousDirectory, ScriptDirectory: string;
begin
  Result := '';
  NeedsRestart := False;
  Directory := ExpandConstant('{app}');

  if not IsValidInstallDirectory(Directory) then
  begin
    Result := 'The installation directory must not contain semicolons, quotes, percent signs, or line breaks, and must not be a drive root.';
    Exit;
  end;

  ScriptDirectory := FindScriptInstallInUserPath;
  if ScriptDirectory <> '' then
  begin
    Result := 'A scripts\install.ps1 installation was found at "' + ScriptDirectory +
              '". Uninstall it first (pwsh -File scripts\install.ps1 -Uninstall -RemoveInstallDir -InstallDir "' +
              ScriptDirectory + '"), then run this installer again.';
    Exit;
  end;

  if FindInstalledVersion <> '' then
  begin
    PreviousDirectory := GetPreviousAppDir;
    if PreviousDirectory = '' then
    begin
      Result := 'An existing RuxCmd installation was found, but its directory could not be read. Uninstall it before running this installer.';
      Exit;
    end;
    if not SamePathEntry(PreviousDirectory, Directory) then
    begin
      Result := 'RuxCmd is already installed in "' + PreviousDirectory +
                '". This installer upgrades in place and does not support changing the directory. Uninstall the existing version first, then install into the new directory.';
      Exit;
    end;
  end;

  if (not IsAdminInstallMode) and IsProtectedDirectory(Directory) then
  begin
    Result := 'Installing into Program Files or Windows requires the "Install for all users" mode. Choose that mode, or select a directory under your user profile.';
    Exit;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemovePathEntryFromEnvironment;
end;
