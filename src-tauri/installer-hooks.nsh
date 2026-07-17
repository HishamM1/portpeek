!ifndef PORTPEEK_INSTALLER_HOOKS_NSH
!define PORTPEEK_INSTALLER_HOOKS_NSH

!include "WinMessages.nsh"

; StrReplace: Standard NSIS string replace function (Installer version)
Function StrReplace
  Exch $0 ; replace with
  Exch
  Exch $1 ; search for
  Exch 2
  Exch $2 ; string
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  StrLen $3 $1 ; len of search for
  StrLen $4 $0 ; len of replace with
  StrCpy $5 0 ; index
  loop:
    StrCpy $6 $2 $3 $5
    StrCmp $6 $1 found
    StrCpy $6 $2 1 $5
    StrCmp $6 "" done
    IntOp $5 $5 + 1
    Goto loop
  found:
    StrCpy $6 $2 $5 ; left side
    IntOp $7 $5 + $3
    StrCpy $7 $2 "" $7 ; right side
    StrCpy $2 "$6$0$7"
    IntOp $5 $5 + $4
    Goto loop
  done:
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $1
  Pop $0
  Exch $2
FunctionEnd

; un.StrReplace: Standard NSIS string replace function (Uninstaller version)
Function un.StrReplace
  Exch $0 ; replace with
  Exch
  Exch $1 ; search for
  Exch 2
  Exch $2 ; string
  Push $3
  Push $4
  Push $5
  Push $6
  Push $7
  StrLen $3 $1 ; len of search for
  StrLen $4 $0 ; len of replace with
  StrCpy $5 0 ; index
  loop:
    StrCpy $6 $2 $3 $5
    StrCmp $6 $1 found
    StrCpy $6 $2 1 $5
    StrCmp $6 "" done
    IntOp $5 $5 + 1
    Goto loop
  found:
    StrCpy $6 $2 $5 ; left side
    IntOp $7 $5 + $3
    StrCpy $7 $2 "" $7 ; right side
    StrCpy $2 "$6$0$7"
    IntOp $5 $5 + $4
    Goto loop
  done:
  Pop $7
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $1
  Pop $0
  Exch $2
FunctionEnd

!macro StrReplace ResultVar String Search Replace
  Push `${String}`
  Push `${Search}`
  Push `${Replace}`
  Call StrReplace
  Pop `${ResultVar}`
!macroend

!macro un_StrReplace ResultVar String Search Replace
  Push `${String}`
  Push `${Search}`
  Push `${Replace}`
  Call un.StrReplace
  Pop `${ResultVar}`
!macroend

; StrStr: Standard NSIS string search function
Function StrStr
  Exch $R0 ; SubString
  Exch
  Exch $R1 ; String
  Push $R2
  Push $R3
  Push $R4
  Push $R5
  StrLen $R2 $R0
  StrLen $R3 $R1
  StrCpy $R4 0
  loop:
    StrCpy $R5 $R1 $R2 $R4
    StrCmp $R5 $R0 found
    StrCmp $R4 $R3 notfound
    IntOp $R4 $R4 + 1
    Goto loop
  found:
    StrCpy $R0 $R1 "" $R4
    Goto done
  notfound:
    StrCpy $R0 ""
  done:
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Pop $R1
  Exch $R0
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Configuring PortPeek CLI companion..."
  
  ; Ensure the bin directory exists
  CreateDirectory "$INSTDIR\bin"
  
  ; Copy portpeek-cli.exe from resources to bin/portpeek.exe
  CopyFiles /SILENT "$INSTDIR\resources\portpeek-cli.exe" "$INSTDIR\bin\portpeek.exe"

  ; Ask the user if they want to add PortPeek to their PATH
  ${Unless} ${Silent}
    MessageBox MB_YESNO|MB_ICONQUESTION "Would you like to add PortPeek to your PATH?$\r$\n$\r$\nThis enables you to run the 'portpeek' command from any PowerShell or Command Prompt window." IDNO skip_path
  ${EndUnless}

  DetailPrint "Adding PortPeek to current-user PATH..."
  
  ; Read current PATH
  ReadRegStr $0 HKCU "Environment" "Path"
  
  ; Check if already in PATH
  Push $0
  Push "$INSTDIR\bin"
  Call StrStr
  Pop $1
  StrCmp $1 "" not_found
  DetailPrint "PortPeek bin directory is already in PATH."
  Goto skip_path

not_found:
  ; Append $INSTDIR\bin to PATH
  StrCmp $0 "" empty_path
    WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR\bin"
    Goto notify
  empty_path:
    WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR\bin"

notify:
  ; Broadcast environment change
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

skip_path:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing PortPeek from current-user PATH..."

  ; Read current PATH
  ReadRegStr $0 HKCU "Environment" "Path"
  
  ; Clean up PATH using un_StrReplace macro for the uninstaller
  ; Remove ;$INSTDIR\bin
  !insertmacro un_StrReplace $0 $0 ";$INSTDIR\bin" ""
  ; Remove $INSTDIR\bin;
  !insertmacro un_StrReplace $0 $0 "$INSTDIR\bin;" ""
  ; Remove $INSTDIR\bin
  !insertmacro un_StrReplace $0 $0 "$INSTDIR\bin" ""
  
  ; Write updated PATH back to registry
  WriteRegExpandStr HKCU "Environment" "Path" $0
  
  ; Broadcast environment change
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

  DetailPrint "Cleaning up PortPeek files..."
  ; Delete our copied CLI binary and its folder
  Delete "$INSTDIR\bin\portpeek.exe"
  RMDir "$INSTDIR\bin"
!macroend

!endif
