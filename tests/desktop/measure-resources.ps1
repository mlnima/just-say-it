param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [int]$Seconds = 300,
    [string]$OutputPath = 'artifacts/release-resources.json'
)
$ErrorActionPreference = 'Stop'
$application = Get-Process -Id $AppProcessId
$application.Refresh()
$initialCpu = $application.CPU
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$samples = @()
while ($timer.Elapsed.TotalSeconds -lt $Seconds) {
    $remaining = $Seconds - $timer.Elapsed.TotalSeconds
    Start-Sleep -Milliseconds ([int]([Math]::Min(10000, $remaining * 1000)))
    $application.Refresh()
    $samples += [PSCustomObject]@{
        elapsed_seconds = $timer.Elapsed.TotalSeconds
        cpu_seconds = $application.CPU
        working_set_bytes = $application.WorkingSet64
        private_bytes = $application.PrivateMemorySize64
    }
    if ($samples.Count % 6 -eq 0) { Write-Output "Release observation: $([int]$timer.Elapsed.TotalSeconds)/$Seconds seconds." }
}
$elapsed = $timer.Elapsed.TotalSeconds
$timer.Stop()
$logicalProcessors = (Get-CimInstance Win32_Processor | Measure-Object NumberOfLogicalProcessors -Sum).Sum
$processes = Get-CimInstance Win32_Process
$owned = [System.Collections.Generic.HashSet[uint32]]::new()
[void]$owned.Add($AppProcessId)
do {
    $added = $false
    foreach ($candidate in $processes) {
        if ($owned.Contains($candidate.ParentProcessId) -and $owned.Add($candidate.ProcessId)) { $added = $true }
    }
} while ($added)
$family = @(Get-Process -Id @($owned) -ErrorAction SilentlyContinue)
$application.Refresh()
$oneCore = ($application.CPU - $initialCpu) / $elapsed * 100
$windowState = & "$PSScriptRoot/native-window.ps1" -AppProcessId $AppProcessId -Action window | ConvertFrom-Json
$result = [PSCustomObject]@{
    interval_seconds = $elapsed
    logical_processors = $logicalProcessors
    cpu_percent_of_one_core = $oneCore
    cpu_percent_of_machine = $oneCore / $logicalProcessors
    native_working_set_bytes = $application.WorkingSet64
    native_private_bytes = $application.PrivateMemorySize64
    descendant_process_count = $family.Count - 1
    family_working_set_bytes = ($family | Measure-Object WorkingSet64 -Sum).Sum
    family_private_bytes = ($family | Measure-Object PrivateMemorySize64 -Sum).Sum
    visible_settings_window = $windowState.visible_settings_window
    samples = $samples
}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutputPath -Encoding utf8
$result | Select-Object -Property * -ExcludeProperty samples | ConvertTo-Json
