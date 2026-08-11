!ifndef PORTPEEK_INSTALLER_HOOKS_NSH
!define PORTPEEK_INSTALLER_HOOKS_NSH

!include "WinMessages.nsh"

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

  ; ponytail: manual HKCU PATH editing. Boundary matching is exact-entry
  ; (semicolon-delimited), not substring. Ceiling — ReadRegStr is capped at
  ; NSIS_MAX_STRLEN, and it cannot tell a missing key from a value that is too
  ; long: both fail with the error flag set and return "". We therefore bail out
  ; on any read error rather than write the truncated value back (which would
  ; wipe a real long PATH). Cost: a user whose HKCU Path key does not yet exist
  ; won't get an auto-add (the CLI still works via its full path). Upgrade path
  ; for full long-PATH support: switch add/remove to the NSIS EnVar plugin.

  ; Read current PATH. Abort the modification if the read fails — writing back
  ; an empty/truncated read here would clobber the user's existing PATH.
  ClearErrors
  ReadRegStr $0 HKCU "Environment" "Path"
  IfErrors path_unreadable

  ; Check for an exact, semicolon-delimited PATH entry.
  StrCpy $2 ";$0;"
  Push $2
  Push ";$INSTDIR\bin;"
  Call StrStr
  Pop $1
  StrCmp $1 "" not_found
  DetailPrint "PortPeek bin directory is already in PATH."
  Goto skip_path

not_found:
  ; Append $INSTDIR\bin to PATH (the empty branch only fires for a genuinely
  ; empty value — a failed read was already caught above).
  StrCmp $0 "" empty_path
    WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR\bin"
    Goto notify
  empty_path:
    WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR\bin"

notify:
  WriteRegStr HKCU "Software\PortPeek" "AddedCliPath" "1"
  ; Broadcast environment change
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  Goto skip_path

path_unreadable:
  DetailPrint "Could not read the current PATH safely; skipped adding PortPeek to PATH. Add $INSTDIR\bin manually if you want the 'portpeek' command."

skip_path:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing PortPeek from current-user PATH..."

  ; Remove a PATH entry only when this installer added it.
  ReadRegStr $1 HKCU "Software\PortPeek" "AddedCliPath"
  StrCmp $1 "1" 0 done

  ; ponytail: removes every exact ";$INSTDIR\bin;" entry we own (duplicates are
  ; semantically redundant, so dropping all of them is fine). Rebuilds PATH from
  ; the semicolon-delimited entries, never a naive substring replace.
  ; Abort if the read fails — writing back a truncated/empty read would wipe the
  ; user's PATH (see the POSTINSTALL note); leave the marker so state is honest.
  ClearErrors
  ReadRegStr $0 HKCU "Environment" "Path"
  IfErrors path_unreadable
  StrCpy $1 ";$0;"
  !insertmacro un_StrReplace $1 $1 ";$INSTDIR\bin;" ";"
  StrCpy $0 $1 "" 1
  StrLen $2 $0
  IntCmp $2 0 path_empty
  IntOp $2 $2 - 1
  StrCpy $0 $0 $2

path_empty:
  WriteRegExpandStr HKCU "Environment" "Path" $0
  DeleteRegValue HKCU "Software\PortPeek" "AddedCliPath"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  Goto done

path_unreadable:
  DetailPrint "Could not read the current PATH safely; left PATH unchanged."

done:
  DetailPrint "Cleaning up PortPeek files..."
  ; Delete our copied CLI binary and its folder
  Delete "$INSTDIR\bin\portpeek.exe"
  RMDir "$INSTDIR\bin"
!macroend

!endif
