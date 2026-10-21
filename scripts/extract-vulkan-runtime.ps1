param(
    [Parameter(Mandatory)][string]$Archive,
    [Parameter(Mandatory)][string]$Destination,
    [Parameter(Mandatory)][string]$ArchiveDirectory
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$runtimeArchive = [IO.Compression.ZipFile]::OpenRead($Archive)
try {
    $runtimeFiles = @{
        'x64/vulkan-1.dll' = 'x64/vulkan-1.dll'
        'x86/vulkan-1.dll' = 'x86/vulkan-1.dll'
        'VulkanRT-License.txt' = 'VulkanRT-License.txt'
    }
    foreach ($relative in $runtimeFiles.Keys) {
        $entry = $runtimeArchive.GetEntry("$ArchiveDirectory/$relative")
        if ($null -eq $entry) { throw "Vulkan runtime archive is missing $relative" }
        $target = Join-Path $Destination $runtimeFiles[$relative]
        New-Item -ItemType Directory -Path (Split-Path $target -Parent) -Force | Out-Null
        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $true)
    }
} finally {
    $runtimeArchive.Dispose()
}
