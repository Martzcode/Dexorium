#requires -Version 5.1
<#
.SYNOPSIS
    Packages the compiled Dexorium application as a Microsoft Store MSIX.

.DESCRIPTION
    The MSIX is built from the release binary and its bundled reference
    database. The package identity comes from the Microsoft Partner Center
    reservation: each value can be passed as a parameter or through an
    environment variable, falling back to the provisional defaults.

    The package stays unsigned by default. Microsoft Store signs submitted
    MSIX packages, so a Partner Center submission does not need a certificate.
    Pass a PFX (or MSIX_SIGN_PFX_PATH) to sign the package for sideloading.

.PARAMETER Semver
    Application version used by the MSIX manifest, e.g. 2026.11.1.

.PARAMETER ReleaseVersion
    Human-readable release version used in the file name, e.g. 2026.11.01.

.PARAMETER PackageName
    Microsoft Store package identity name (env: MSIX_IDENTITY_NAME).

.PARAMETER Publisher
    Microsoft Store publisher identity, CN=... (env: MSIX_PUBLISHER).

.PARAMETER PublisherDisplayName
    Publisher display name shown by Windows (env: MSIX_PUBLISHER_DISPLAY).

.PARAMETER SignPfx
    Optional PKCS#12 certificate used to sign the package
    (env: MSIX_SIGN_PFX_PATH).

.PARAMETER SignPassword
    Optional password of that certificate (env: MSIX_SIGN_PASSWORD).
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Semver,

    [Parameter(Mandatory = $true)]
    [string]$ReleaseVersion,

    [string]$PackageName,
    [string]$Publisher,
    [string]$PublisherDisplayName,
    [string]$Target = 'release',
    [string]$OutputDirectory,
    [string]$SignPfx,
    [string]$SignPassword
)

function Resolve-Setting {
    param(
        [string]$Value,
        [string]$EnvironmentName,
        [string]$Default
    )

    if (-not [string]::IsNullOrWhiteSpace($Value)) { return $Value.Trim() }

    $fromEnvironment = [Environment]::GetEnvironmentVariable($EnvironmentName)
    if (-not [string]::IsNullOrWhiteSpace($fromEnvironment)) { return $fromEnvironment.Trim() }

    return $Default
}

$PackageName = Resolve-Setting $PackageName 'MSIX_IDENTITY_NAME' 'Martzcode.Dexorium'
$Publisher = Resolve-Setting $Publisher 'MSIX_PUBLISHER' 'CN=Martzcode'
$PublisherDisplayName = Resolve-Setting $PublisherDisplayName 'MSIX_PUBLISHER_DISPLAY' 'Martzcode'
$SignPfx = Resolve-Setting $SignPfx 'MSIX_SIGN_PFX_PATH' ''
$SignPassword = Resolve-Setting $SignPassword 'MSIX_SIGN_PASSWORD' ''

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

function ConvertTo-XmlText([string]$Value) {
    return [System.Security.SecurityElement]::Escape($Value)
}

$manifest = Get-Content -LiteralPath $manifestTemplate -Raw
$manifest = $manifest.Replace('{{PACKAGE_NAME}}', (ConvertTo-XmlText $PackageName))
$manifest = $manifest.Replace('{{PUBLISHER}}', (ConvertTo-XmlText $Publisher))
$manifest = $manifest.Replace('{{PUBLISHER_DISPLAY_NAME}}', (ConvertTo-XmlText $PublisherDisplayName))
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

if ($SignPfx) {
    if (-not (Test-Path -LiteralPath $SignPfx)) {
        throw "Signing certificate not found: $SignPfx"
    }

    $signTool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($signTool) {
        $signToolPath = $signTool.Source
    } else {
        $kitsRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
        $signToolPath = Get-ChildItem -LiteralPath $kitsRoot -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending |
            ForEach-Object { Join-Path $_.FullName 'x64/signtool.exe' } |
            Where-Object { Test-Path -LiteralPath $_ } |
            Select-Object -First 1
    }

    if (-not $signToolPath) {
        throw 'SignTool.exe was not found. Install the Windows SDK or run this script on a Windows runner.'
    }

    if ($SignPassword) {
        & $signToolPath sign /fd SHA256 /a /f $SignPfx /p $SignPassword /tr http://timestamp.digicert.com /td SHA256 $package
    } else {
        & $signToolPath sign /fd SHA256 /a /f $SignPfx /tr http://timestamp.digicert.com /td SHA256 $package
    }

    if ($LASTEXITCODE -ne 0) {
        throw "SignTool failed with exit code $LASTEXITCODE."
    }

    Write-Host "MSIX package signed: $package"
} else {
    Write-Host '::notice::MSIX left unsigned - ready for Partner Center (the Store re-signs it).'
}

Write-Host "MSIX package: $package"
