; DevTextTools installer — per-user, no admin needed.
Unicode true
Name "DevTextTools"
; The release workflow passes the version with -DAppVersion=x.y.z;
; the default below is only for local test builds.
!ifndef AppVersion
  !define AppVersion "0.8.0"
!endif
; The packaging step sets the output file with -DOUTFILE_NAME=...
; (the workflow renames the result to DevTextTools-v{VERSION}-x64-Setup.exe).
!ifndef OUTFILE_NAME
  !define OUTFILE_NAME "DevTextTools-Setup.exe"
!endif
OutFile "${OUTFILE_NAME}"
InstallDir "$LOCALAPPDATA\Programs\DevTextTools"
RequestExecutionLevel user
Icon "..\assets\icon.ico"
UninstallIcon "..\assets\icon.ico"
SetCompressor /SOLID lzma

Page directory
Page instfiles

Section "Install"
  SetOutPath "$INSTDIR"
  File "..\dist\DevTextTools.exe"
  File "..\LICENSE"
  File "..\README.md"
  CreateDirectory "$SMPROGRAMS\DevTextTools"
  CreateShortcut "$SMPROGRAMS\DevTextTools\DevTextTools.lnk" "$INSTDIR\DevTextTools.exe"
  CreateShortcut "$DESKTOP\DevTextTools.lnk" "$INSTDIR\DevTextTools.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools" "DisplayName" "DevTextTools"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools" "DisplayVersion" "${AppVersion}"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\DevTextTools.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\Uninstall.exe"
  ; The program's own data folder (settings + saved session) lives
  ; beside the exe, i.e. inside $INSTDIR for an installed build.
  RMDir /r "$INSTDIR\data"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\DevTextTools\DevTextTools.lnk"
  RMDir "$SMPROGRAMS\DevTextTools"
  Delete "$DESKTOP\DevTextTools.lnk"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools"
SectionEnd
