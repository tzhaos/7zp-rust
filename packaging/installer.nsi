Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "FileFunc.nsh"
Name "Plus7z"
OutFile "${PROJECT_ROOT}\dist\p7z-amd64-installer.exe"
InstallDir "$LOCALAPPDATA\Programs\Plus7z"
InstallDirRegKey HKCU "Software\Plus7z" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey /LANG=1033 "ProductName" "Plus7z"
VIAddVersionKey /LANG=1033 "FileDescription" "Plus7z Setup"
VIAddVersionKey /LANG=1033 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Plus7z contributors"
!define MUI_ICON "${PROJECT_ROOT}\assets\brand\app.ico"
!define MUI_UNICON "${PROJECT_ROOT}\assets\brand\app.ico"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "$(OpenDefaultApps)"
!define MUI_FINISHPAGE_RUN_FUNCTION OpenDefaultAppsSettings
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "TradChinese"
LangString OpenDefaultApps ${LANG_ENGLISH} "Choose default archive apps in Windows Settings"
LangString OpenDefaultApps ${LANG_SIMPCHINESE} "在 Windows 设置中选择压缩包默认打开应用"
LangString OpenDefaultApps ${LANG_TRADCHINESE} "在 Windows 設定中選擇壓縮檔預設開啟應用程式"
LangString ClosingApp ${LANG_ENGLISH} "Closing Plus7z..."
LangString ClosingApp ${LANG_SIMPCHINESE} "正在关闭 Plus7z…"
LangString ClosingApp ${LANG_TRADCHINESE} "正在關閉 Plus7z…"
LangString CleanupDeferred ${LANG_ENGLISH} "Loaded menu modules will be removed automatically at your next sign-in."
LangString CleanupDeferred ${LANG_SIMPCHINESE} "已加载的菜单模块将在下次登录时自动清理。"
LangString CleanupDeferred ${LANG_TRADCHINESE} "已載入的選單模組將於下次登入時自動清理。"
LangString CleanupFailed ${LANG_ENGLISH} "The old menu module could not be moved to the cleanup directory. Check the installation folder permissions."
LangString CleanupFailed ${LANG_SIMPCHINESE} "无法将旧菜单模块移入待清理目录，请检查安装目录权限。"
LangString CleanupFailed ${LANG_TRADCHINESE} "無法將舊選單模組移至待清理資料夾，請檢查安裝資料夾權限。"
Var AppLanguage
Var RetiredDir
Var RetiredId
Var ShellKeep
Var ShellFind
Var ShellFile
Var ShellPath
Var ShellName
Var CleanupOnly
Var Updating

; Retired modules have no registration. Each cleanup job owns a separate directory,
; so signing in after reinstalling cannot delete files from the new installation.
!macro RetireShell Prefix
Function ${Prefix}RetireShell
    FindFirst $ShellFind $ShellFile "$INSTDIR\shell\*"
    IfErrors retire_done
    retire_version_next:
    StrCmp $ShellFile "" retire_version_close
    StrCmp $ShellFile "." retire_version_continue
    StrCmp $ShellFile ".." retire_version_continue
    StrCmp $ShellFile $ShellKeep retire_version_continue
    StrCpy $ShellPath "$INSTDIR\shell\$ShellFile\p7z-explorer.dll"
    IfFileExists "$ShellPath" 0 retire_version_continue
    StrCpy $ShellName "p7z-explorer.dll"
    Call ${Prefix}RetireShellFile
    RMDir "$INSTDIR\shell\$ShellFile"
    retire_version_continue:
    FindNext $ShellFind $ShellFile
    Goto retire_version_next
    retire_version_close:
    FindClose $ShellFind
    RMDir "$INSTDIR\shell"
    retire_done:
FunctionEnd

Function ${Prefix}RetireShellFile
    ClearErrors
    Delete "$ShellPath"
    IfErrors 0 retire_file_done
    ClearErrors
    ; One same-volume directory per module keeps identical DLL names separate.
    CreateDirectory "$INSTDIR\.p7z-retired"
    GetTempFileName $RetiredDir "$INSTDIR\.p7z-retired"
    IfErrors retire_failed
    Delete "$RetiredDir"
    CreateDirectory "$RetiredDir"
    !if "${Prefix}" == ""
        WriteUninstaller "$RetiredDir\cleanup.exe"
    !else
        CopyFiles /SILENT "$INSTDIR\Uninstall.exe" "$RetiredDir\cleanup.exe"
    !endif
    StrCpy $RetiredId $RetiredDir
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\RunOnce" "Plus7z.Cleanup.$RetiredId" '$\"$RetiredDir\cleanup.exe$\" /S /CLEANUP'
    IfErrors retire_failed
    ClearErrors
    Rename "$ShellPath" "$RetiredDir\$ShellName"
    IfErrors retire_failed
    DetailPrint "$(CleanupDeferred)"
    Return
    retire_failed:
    FindClose $ShellFind
    MessageBox MB_ICONSTOP|MB_OK "$(CleanupFailed)" /SD IDOK
    SetErrorLevel 1
    Abort
    retire_file_done:
FunctionEnd
!macroend
!insertmacro RetireShell ""
!insertmacro RetireShell "un."

Function un.CleanupRetired
    SetOutPath "$TEMP"
    ClearErrors
    Delete "$INSTDIR\p7z-explorer.dll"
    ${If} ${Errors}
        WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\RunOnce" "Plus7z.Cleanup.$RetiredId" '$\"$INSTDIR\cleanup.exe$\" /S /CLEANUP'
        Return
    ${EndIf}
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\RunOnce" "Plus7z.Cleanup.$RetiredId"
    Delete "$INSTDIR\cleanup.exe"
    RMDir "$INSTDIR"
    ${GetParent} "$INSTDIR" $0
    RMDir "$0"
    ${GetParent} "$0" $0
    RMDir "$0"
FunctionEnd

Function OpenDefaultAppsSettings
    ExecShell "open" "ms-settings:defaultapps"
FunctionEnd

Function .onInit
    StrCpy $Updating 0
    ${GetParameters} $0
    ClearErrors
    ${GetOptions} $0 "/UPDATE" $1
    ${IfNot} ${Errors}
        StrCpy $Updating 1
    ${EndIf}
    ${IfNot} ${RunningX64}
        Abort
    ${EndIf}
    SetShellVarContext current
    System::Call 'kernel32::GetUserDefaultUILanguage() i .r0'
    StrCpy $LANGUAGE 1033
    StrCpy $AppLanguage "en-US"
    ${If} $0 == 2052
    ${OrIf} $0 == 4100
        StrCpy $LANGUAGE 2052
        StrCpy $AppLanguage "zh-CN"
    ${ElseIf} $0 == 1028
    ${OrIf} $0 == 3076
    ${OrIf} $0 == 5124
        StrCpy $LANGUAGE 1028
        StrCpy $AppLanguage "zh-TW"
    ${EndIf}
FunctionEnd

Section
    InitPluginsDir
    SetOutPath "$PLUGINSDIR"
    File /oname=p7z-maintenance.exe "${PROJECT_ROOT}\bin\p7z.exe"
    File "${PROJECT_ROOT}\bin\vcruntime140.dll"
    DetailPrint "$(ClosingApp)"
    ClearErrors
    ExecWait '"$PLUGINSDIR\p7z-maintenance.exe" --lang $AppLanguage --prepare-install "$INSTDIR"' $0
    ${If} ${Errors}
    ${OrIf} $0 != 0
        SetErrorLevel 1
        Abort
    ${EndIf}
    SetOutPath "$INSTDIR"
    ClearErrors
    File "${PROJECT_ROOT}\bin\p7z.exe"
    File "${PROJECT_ROOT}\bin\vcruntime140.dll"
    ; Explorer can keep earlier modules loaded; identical content needs no overwrite.
    SetOutPath "$INSTDIR\shell\${SHELL_HASH}"
    SetOverwrite off
    File "${PROJECT_ROOT}\bin\p7z-explorer.dll"
    SetOverwrite on
    SetOutPath "$INSTDIR"
    File /oname=Glyphs-LICENSE.txt "${PROJECT_ROOT}\assets\glyphs\LICENSE"
    File /oname=LICENSE.txt "${PROJECT_ROOT}\LICENSE"
    File /oname=Cardo-LICENSE.txt "${PROJECT_ROOT}\cardo\LICENSE"
    File /oname=SQLite-binding-LICENSE.txt "${PROJECT_ROOT}\cardo\licenses\rusqlite.txt"
    SetOutPath "$INSTDIR\runtime\7zip"
    File "${PROJECT_ROOT}\bin\runtime\7zip\*.*"
    ${If} ${Errors}
        SetErrorLevel 1
        Abort
    ${EndIf}
    SetOutPath "$INSTDIR"
    ExecWait '"$INSTDIR\p7z.exe" --lang $AppLanguage --register "$INSTDIR\shell\${SHELL_HASH}\p7z-explorer.dll"' $0
    ${If} $0 != 0
        SetErrorLevel 1
        Abort
    ${EndIf}
    StrCpy $ShellKeep "${SHELL_HASH}"
    ${If} $Updating == 0
        Call RetireShell
        CreateShortcut "$DESKTOP\Plus7z.lnk" "$INSTDIR\p7z.exe"
        CreateShortcut "$SMPROGRAMS\Plus7z.lnk" "$INSTDIR\p7z.exe"
    ${EndIf}
    ClearErrors
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    ${If} ${Errors}
        SetErrorLevel 1
        Abort
    ${EndIf}
    ClearErrors
    WriteRegStr HKCU "Software\Plus7z" "InstallDir" "$INSTDIR"
    WriteRegDWORD HKCU "Software\Plus7z" "Language" $LANGUAGE
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "DisplayName" "Plus7z"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "DisplayIcon" '"$INSTDIR\p7z.exe",0'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "InstallLocation" "$INSTDIR"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z" "NoRepair" 1
    ${If} ${Errors}
        SetErrorLevel 1
        Abort
    ${EndIf}
SectionEnd

Function un.onInit
    SetShellVarContext current
    StrCpy $CleanupOnly 0
    ${GetParameters} $0
    ClearErrors
    ${GetOptions} $0 "/CLEANUP" $1
    ${IfNot} ${Errors}
        ${GetParent} "$INSTDIR" $0
        ${GetFileName} "$0" $1
        ${If} $1 != ".p7z-retired"
            Abort
        ${EndIf}
        StrCpy $RetiredId $INSTDIR
        StrCpy $CleanupOnly 1
        Return
    ${EndIf}
    ReadRegDWORD $LANGUAGE HKCU "Software\Plus7z" "Language"
FunctionEnd

Section "Uninstall"
    ${If} $CleanupOnly == 1
        Call un.CleanupRetired
        Return
    ${EndIf}
    DetailPrint "$(ClosingApp)"
    ClearErrors
    ExecWait '"$INSTDIR\p7z.exe" --prepare-install "$INSTDIR"' $0
    ${If} ${Errors}
    ${OrIf} $0 != 0
        SetErrorLevel 1
        Abort
    ${EndIf}
    ExecWait '"$INSTDIR\p7z.exe" --unregister' $0
    ${If} $0 != 0
        Abort
    ${EndIf}
    StrCpy $ShellKeep ""
    Call un.RetireShell
    SetOutPath "$TEMP"
    ReadRegStr $1 HKCU "Software\Plus7z" "InstallDir"
    ${If} $1 == $INSTDIR
        Delete "$DESKTOP\Plus7z.lnk"
        Delete "$SMPROGRAMS\Plus7z.lnk"
    ${EndIf}
    Delete "$INSTDIR\p7z.exe"
    Delete "$INSTDIR\vcruntime140.dll"
    Delete "$INSTDIR\Glyphs-LICENSE.txt"
    Delete "$INSTDIR\LICENSE.txt"
    Delete "$INSTDIR\Cardo-LICENSE.txt"
    Delete "$INSTDIR\SQLite-binding-LICENSE.txt"
    Delete "$INSTDIR\THIRD_PARTY.md"
    Delete "$INSTDIR\runtime\7zip\7z.exe"
    Delete "$INSTDIR\runtime\7zip\7z.dll"
    Delete "$INSTDIR\runtime\7zip\7z.sfx"
    Delete "$INSTDIR\runtime\7zip\7zCon.sfx"
    Delete "$INSTDIR\runtime\7zip\License.txt"
    Delete "$INSTDIR\runtime\7zip\readme.txt"
    Delete "$INSTDIR\runtime\7zip\History.txt"
    Delete "$INSTDIR\runtime\7zip\7-zip.chm"
    RMDir "$INSTDIR\runtime\7zip"
    RMDir "$INSTDIR\runtime"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    ${If} $1 == $INSTDIR
        DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Plus7z"
        DeleteRegValue HKCU "Software\Plus7z" "InstallDir"
        DeleteRegValue HKCU "Software\Plus7z" "Language"
        DeleteRegKey /ifempty HKCU "Software\Plus7z"
    ${EndIf}
SectionEnd
