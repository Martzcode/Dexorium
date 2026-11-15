#requires -Version 5.1
<#
.SYNOPSIS
    Packages the compiled Dexorium application as an unsigned MSIX.

.DESCRIPTION
    The MSIX is built from the release binary and its bundled reference
    database. The package identity is intentionally provisional: replace the
    PackageName, Publisher and PublisherDisplayName values with the identity
    reserved in Microsoft Partner Center before submitting to the Store.

    The package is not signed. Microsoft Store signs submitted MSIX packages;
    for local sideloading, sign the .msix with a trusted certificate first.

.PARAMETER Semver
    Application version used by the MSIX manifest, e.g. 2026.11.1.

.PARAMETER ReleaseVersion
    Human-readable release version used in the file name, e.g. 2026.11.01.

.PARAMETER PackageName
    Microsoft Store package identity name.

.PARAMETER Publisher
    Microsoft Store publisher identity (CN=...).

.PARAMETER PublisherDisplayName
    Publisher display name shown by Windows.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Semver,

    [Parameter(Mandatory = $true)]
    [string]$ReleaseVersion,

    [string]$PackageName = 'Martzcode.Dexorium',
    [string]$Publisher = 'CN=Martzcode',
    [string]$PublisherDisplayName = 'Martzcode',
    [string]$Target = 'release',
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$targetDirectory = Join-Path $root "src-tauri/target/$Target"
$executable = Join-Path $targetDirectory 'dexorium.exe'
$referenceDatabase = Join-Path $targetDirectory 'reference.db'
$iconsDirectory = Join-Path $root 'src-tauri/icons'
$manifestTemplate = Join-Path $PSScriptRoot 'msix/AppxManifest.xml'

if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $targetDirectory 'bundle/msix'
}

foreach ($required in @($executable, $referenceDatabase, $manifestTemplate)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Required file not found: $required"
    }
}

if ($Semver -notmatch '^\d+\.\d+\.\d+$') {
    throw "Semver must have three numeric parts, got '$Semver'."
}

$msixVersion = "$Semver.0"
$layout = Join-Path $OutputDirectory 'layout'

if (Test-Path -LiteralPath $layout) {
    Remove-Item -LiteralPath $layout -Recurse -Force
}

New-Item -ItemType Directory -Path (Join-Path $layout 'Assets') -Force | Out-Null
Copy-Item -LiteralPath $executable -Destination (Join-Path $layout 'Dexorium.exe')
Copy-Item -LiteralPath $referenceDatabase -Destination (Join-Path $layout 'reference.db')

foreach ($asset in @('StoreLogo.png', 'Square150x150Logo.png', 'Square44x44Logo.png')) {
    Copy-Item -LiteralPath (Join-Path $iconsDirectory $asset) -Destination (Join-Path $layout "Assets/$asset")
}

$manifest = Get-Content -LiteralPath $manifestTemplate -Raw
$manifest = $manifest.Replace('{{PACKAGE_NAME}}', $PackageName)
$manifest = $manifest.Replace('{{PUBLISHER}}', $Publisher)
$manifest = $manifest.Replace('{{PUBLISHER_DISPLAY_NAME}}', $PublisherDisplayName)
$manifest = $manifest.Replace('{{MSIX_VERSION}}', $msixVersion)

[System.IO.File]::WriteAllText(
    (Join-Path $layout 'AppxManifest.xml'),
    $manifest,
    [System.Text.UTF8Encoding]::new($false)
)

$makeAppx = Get-Command makeappx.exe -ErrorAction SilentlyContinue
if ($makeAppx) {
    $makeAppxPath = $makeAppx.Source
} else {
    $kitsRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
    $makeAppxPath = Get-ChildItem -LiteralPath $kitsRoot -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending |
        ForEach-Object { Join-Path $_.FullName 'x64/makeappx.exe' } |
        Where-Object { Test-Path -LiteralPath $_ } |
        Select-Object -First 1
}

if (-not $makeAppxPath) {
    throw 'MakeAppx.exe was not found. Install the Windows SDK or run this script on a Windows runner.'
}

$package = Join-Path $OutputDirectory "Dexorium_${ReleaseVersion}_x64.msix"
& $makeAppxPath pack /o /d $layout /p $package
if ($LASTEXITCODE -ne 0) {
    throw "MakeAppx failed with exit code $LASTEXITCODE."
}

Write-Host "MSIX package: $package"
