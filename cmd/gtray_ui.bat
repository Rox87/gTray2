@echo off
cd ..
if exist "bin\gtray.exe" (
    start "" "bin\gtray.exe"
) else if exist "target\release\gtray.exe" (
    start "" "target\release\gtray.exe"
) else (
    start "" "target\debug\gtray.exe"
)
