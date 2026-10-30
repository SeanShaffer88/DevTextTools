; DevTextTools installer — per-user, no admin needed.
Unicode true
Name "DevTextTools"
; The packaging step overrides OUTFILE_NAME per architecture
; (makensis -DOUTFILE_NAME=DevTextTools-Setup-x86.exe for the 32-bit build).
!ifndef OUTFILE_NAME
  !define OUTFILE_NAME "DevTextTools-Setup-x64.exe"
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
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools" "DisplayVersion" "0.7.2"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\DevTextTools.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\DevTextTools\DevTextTools.lnk"
  RMDir "$SMPROGRAMS\DevTextTools"
  Delete "$DESKTOP\DevTextTools.lnk"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\DevTextTools"
SectionEnd
