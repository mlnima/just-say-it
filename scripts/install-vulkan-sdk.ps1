$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$projectDirectory = Split-Path $PSScriptRoot -Parent
$vulkanConfig = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'vulkan.json') -Raw | ConvertFrom-Json
$sdkDirectory = Join-Path $projectDirectory 'artifacts/build-tools/vulkan-sdk'
New-Item -ItemType Directory -Path $sdkDirectory -Force | Out-Null
$sdkInstaller = Join-Path $sdkDirectory 'installer.exe'
if (!(Test-Path -LiteralPath $sdkInstaller) -or (Get-FileHash -LiteralPath $sdkInstaller -Algorithm SHA256).Hash.ToLowerInvariant() -ne $vulkanConfig.sdkSha256) {
    Invoke-WebRequest -Uri $vulkanConfig.sdkUrl -OutFile $sdkInstaller -TimeoutSec 600
}
if ((Get-FileHash -LiteralPath $sdkInstaller -Algorithm SHA256).Hash.ToLowerInvariant() -ne $vulkanConfig.sdkSha256) {
    throw 'Vulkan SDK integrity check failed.'
}
$sdkRoot = Join-Path $sdkDirectory $vulkanConfig.sdkVersion
$sdkOutput = Join-Path $sdkDirectory 'install.log'
$sdkError = Join-Path $sdkDirectory 'install-error.log'
$sdkProcess = Start-Process -FilePath $sdkInstaller -ArgumentList @('--root', "`"$sdkRoot`"", '--accept-licenses', '--default-answer', '--confirm-command', 'install', 'copy_only=1') -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput $sdkOutput -RedirectStandardError $sdkError
if ($sdkProcess.ExitCode -ne 0) {
    Get-Content -LiteralPath $sdkOutput, $sdkError -Tail 35
    throw "Vulkan SDK installation failed: $($sdkProcess.ExitCode)"
}
if (!(Test-Path -LiteralPath (Join-Path $sdkRoot 'Bin/glslc.exe'))) { throw 'The Vulkan SDK shader compiler is missing.' }
if ($env:GITHUB_ENV) { "VULKAN_SDK=$sdkRoot" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8 }
Write-Output "Vulkan SDK installed at $sdkRoot"
