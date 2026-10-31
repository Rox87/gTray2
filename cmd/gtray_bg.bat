@echo off
cd ..
if exist "bin\gtray.exe" (
    start "" "bin\gtray.exe" 2
) else if exist "target\release\gtray.exe" (
    start "" "target\release\gtray.exe" 2
) else (
    start "" "target\debug\gtray.exe" 2
)
