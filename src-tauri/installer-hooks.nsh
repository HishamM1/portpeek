!ifndef PORTPEEK_INSTALLER_HOOKS_NSH
!define PORTPEEK_INSTALLER_HOOKS_NSH

!include "WinMessages.nsh"

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Configuring PortPeek CLI companion..."
  CreateDirectory "$INSTDIR\bin"
  Delete "$INSTDIR\bin\portpeek.exe"
  ClearErrors
  Rename "$INSTDIR\portpeek-cli.exe" "$INSTDIR\bin\portpeek.exe"
  IfErrors cli_copy_failed

  ; Ask the user if they want to add PortPeek to their PATH
  ${Unless} ${Silent}
    MessageBox MB_YESNO|MB_ICONQUESTION "Would you like to add PortPeek to your PATH?$\r$\n$\r$\nThis enables you to run the 'portpeek' command from any PowerShell or Command Prompt window." IDNO skip_path
  ${EndUnless}

  DetailPrint "Adding PortPeek to current-user PATH..."
  ExecWait '"$INSTDIR\bin\portpeek.exe" --install-path "$INSTDIR\bin"' $0
  IntCmp $0 0 path_added path_failed path_failed

path_added:
  WriteRegStr HKCU "Software\PortPeek" "AddedCliPath" "1"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  Goto skip_path

path_failed:
  DetailPrint "Could not add PortPeek to PATH. The CLI remains available at $INSTDIR\bin\portpeek.exe."
  Goto skip_path

cli_copy_failed:
  DetailPrint "Could not install the PortPeek CLI companion."

skip_path:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing PortPeek from current-user PATH..."

  ReadRegStr $1 HKCU "Software\PortPeek" "AddedCliPath"
  StrCmp $1 "1" 0 done
  ExecWait '"$INSTDIR\bin\portpeek.exe" --remove-path "$INSTDIR\bin"' $0
  IntCmp $0 0 path_removed done done

path_removed:
  DeleteRegValue HKCU "Software\PortPeek" "AddedCliPath"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000

done:
  DetailPrint "Cleaning up PortPeek files..."
  Delete "$INSTDIR\bin\portpeek.exe"
  RMDir "$INSTDIR\bin"
!macroend

!endif
