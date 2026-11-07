; Bhabex Installer for Windows
; Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

!include "MUI2.nsh"

Name "Bhabex"
OutFile "bhabex-installer.exe"
InstallDir "$LOCALAPPDATA\Bhabex"
RequestExecutionLevel user

!define MUI_ICON "assets\bhabex.ico"
!define MUI_UNICON "assets\bhabex.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

Function .onInit
    ; Check if already installed
    ReadRegStr $0 HKCU "Software\Bhabex" "InstallPath"
    StrCmp $0 "" 0 already_installed
    Goto done
    
already_installed:
    MessageBox MB_YESNO "Bhabex is already installed at $0.\nReinstall?" IDYES done IDNO cancel
    
cancel:
    Abort
    
done:
FunctionEnd

Section "Main"
    SetOutPath $INSTDIR
    File /r "bin\*"
    File /r "stdlib\*"
    File "bin\bx.exe"
    File "bin\bxpkg.exe"
    File "LICENSE.txt"
    
    ; Add to PATH
    ReadEnvStr $0 PATH
    StrStr $1 $0 "$INSTDIR\bin"
    StrCmp $1 "" add_to_path skip_path
    
add_to_path:
    System::Call 'kernel32::SetEnvironmentVariable(t "PATH", t "$0;$INSTDIR\bin")'
    WriteRegStr HKCU "Environment" "PATH" "$0;$INSTDIR\bin"
    
skip_path:
    ; Register .bx file association
    WriteRegStr HKCR ".bx" "" "BhabexFile"
    WriteRegStr HKCR "BhabexFile" "" "Bhabex Source File"
    WriteRegStr HKCR "BhabexFile\DefaultIcon" "" "$INSTDIR\bin\bhabex.ico"
    WriteRegStr HKCR "BhabexFile\shell\open\command" "" '"$INSTDIR\bin\bx.exe" "%1"'
    
    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKCU "Software\Bhabex" "InstallPath" "$INSTDIR"
    
    ; Install VS Code extension
    ExecWait '"$INSTDIR\bin\bx.exe" vscode install'
SectionEnd

Section "Uninstall"
    ; Remove from PATH
    ReadEnvStr $0 PATH
    StrReplace $0 $0 "$INSTDIR\bin;" ""
    System::Call 'kernel32::SetEnvironmentVariable(t "PATH", t "$0")'
    WriteRegStr HKCU "Environment" "PATH" "$0"
    
    ; Remove file association
    DeleteRegKey HKCR ".bx"
    DeleteRegKey HKCR "BhabexFile"
    
    ; Remove registry
    DeleteRegKey HKCU "Software\Bhabex"
    
    ; Uninstall VS Code extension
    ExecWait '"$INSTDIR\bin\bx.exe" vscode uninstall'
SectionEnd