$ErrorActionPreference = 'stop'

$7zPath = Get-Command 7z -ErrorAction SilentlyContinue
if (-not $7zPath) {
  Write-Error "7-Zip is not installed or not found in the system PATH."
  exit 1
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

$archiveName = "ffmpeg-release-full-shared"
$zipFileName = "$($archiveName).7z"
$extractPath = Join-Path -Path $scriptDir -ChildPath $archiveName

if (Test-Path $extractPath) {
  rd $extractPath -recurse -force
}

$url = "https://www.gyan.dev/ffmpeg/builds/$($zipFileName)"
$zipPath = Join-Path -Path $scriptDir -ChildPath $zipFileName

Invoke-WebRequest -Uri $url -OutFile $zipPath

7z x $zipPath -o*

$binDir = Get-ChildItem -Path "$($scriptDir)\$($archiveName)" -Recurse -Directory | Where-Object { $_.Name -eq 'bin' } | Select-Object -First 1
if ($binDir -eq $null) {
    Write-Error "No 'bin' directory found in the extracted contents."
    exit 1
}

$env:FFMPEG_DIR = $binDir.Parent.FullName

Remove-Item -Path $zipPath

Write-Output "FFMPEG_DIR is set to $($env:FFMPEG_DIR)"
