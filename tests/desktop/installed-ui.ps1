param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [ValidateSet('check', 'quit')][string]$Action = 'check'
)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/native-window.ps1" -AppProcessId $AppProcessId -Action window | Out-Null
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$deadline = [DateTime]::UtcNow.AddSeconds(15)
do {
    $settingsWindow = [NativeControl]::AppWindow($AppProcessId)
    if ($settingsWindow -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 100
} while ([DateTime]::UtcNow -lt $deadline)
if ($settingsWindow -eq [IntPtr]::Zero) { throw 'The installed settings window did not open.' }
$root = [System.Windows.Automation.AutomationElement]::FromHandle($settingsWindow)
function Get-Control([string]$controlName, [System.Windows.Automation.ControlType]$controlType) {
    $nameCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $controlName)
    $typeCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $controlType)
    $condition = New-Object System.Windows.Automation.AndCondition($nameCondition, $typeCondition)
    $controlDeadline = [DateTime]::UtcNow.AddSeconds(10)
    do {
        $found = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
        if ($null -ne $found) { return $found }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $controlDeadline)
    throw "Installed UI control missing: $controlName"
}
function Invoke-Control([string]$controlName) {
    $control = Get-Control $controlName ([System.Windows.Automation.ControlType]::Button)
    $pattern = $control.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $pattern.Invoke()
}
if ($Action -eq 'quit') {
    Invoke-Control 'Background & privacy'
    try { Invoke-Control 'Quit application' } catch { if (Get-Process -Id $AppProcessId -ErrorAction SilentlyContinue) { throw } }
    $quitDeadline = [DateTime]::UtcNow.AddSeconds(10)
    while (Get-Process -Id $AppProcessId -ErrorAction SilentlyContinue) {
        if ([DateTime]::UtcNow -gt $quitDeadline) { throw 'Installed Quit did not exit the native runtime.' }
        Start-Sleep -Milliseconds 100
    }
    Write-Output 'Installed application Quit exited the native runtime.'
    return
}
Get-Control 'Microphone closed' ([System.Windows.Automation.ControlType]::Text) | Out-Null
Invoke-Control 'Activation'
$phrase = Get-Control 'Wake phrase' ([System.Windows.Automation.ControlType]::Edit)
$value = $phrase.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
Write-Output "Settings input accessibility provider PID: $($phrase.Current.ProcessId); application PID: $AppProcessId."
if ($value -ne '') { throw 'The installed app prefilled an unsaved wake phrase.' }
[NativeControl]::PostMessage($settingsWindow, 16, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$closeDeadline = [DateTime]::UtcNow.AddSeconds(10)
while ([NativeControl]::AppWindow($AppProcessId) -ne [IntPtr]::Zero) {
    if ([DateTime]::UtcNow -gt $closeDeadline) { throw 'The installed settings window did not close.' }
    Start-Sleep -Milliseconds 100
}
if (-not (Get-Process -Id $AppProcessId -ErrorAction SilentlyContinue)) { throw 'Closing settings exited the installed app.' }
Write-Output 'Installed bundled UI, live closed-microphone status, empty wake phrase and background window close passed.'
