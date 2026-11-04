@echo off
rem Beispielaktion fuer das Software Portal: OneDrive zuruecksetzen.
rem Laeuft als angemeldeter Benutzer. Exit-Code 0 = Erfolg; die letzte Ausgabezeile
rem erscheint im Software Portal als Meldung, bei Fehlern die erste Fehlerzeile.
setlocal

set "ONEDRIVE=%LOCALAPPDATA%\Microsoft\OneDrive\OneDrive.exe"
if not exist "%ONEDRIVE%" set "ONEDRIVE=%ProgramFiles%\Microsoft OneDrive\OneDrive.exe"
if not exist "%ONEDRIVE%" (
    echo OneDrive ist auf diesem Computer nicht installiert. 1>&2
    exit /b 1
)

"%ONEDRIVE%" /reset
rem "timeout" needs a console; ping waits about 10 seconds without one
ping -n 11 127.0.0.1 >nul
start "" "%ONEDRIVE%"
echo OneDrive wurde zurückgesetzt und startet neu.
exit /b 0
