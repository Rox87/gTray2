Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)
parentDir = fso.GetParentFolderName(scriptDir)

exePath = ""
workDir = ""

If fso.FileExists(parentDir & "\gtray.exe") Then
    exePath = parentDir & "\gtray.exe"
    workDir = parentDir
ElseIf fso.FileExists(scriptDir & "\gtray.exe") Then
    exePath = scriptDir & "\gtray.exe"
    workDir = scriptDir
ElseIf fso.FileExists(parentDir & "\bin\gtray.exe") Then
    exePath = parentDir & "\bin\gtray.exe"
    workDir = parentDir & "\bin"
End If

If exePath <> "" Then
    Set WshShell = CreateObject("WScript.Shell")
    WshShell.Run "powershell -WindowStyle Hidden -NoProfile -ExecutionPolicy Bypass -Command ""Start-Process -FilePath '" & exePath & "' -ArgumentList '2' -WorkingDirectory '" & workDir & "' -WindowStyle Hidden""", 0, False
End If
