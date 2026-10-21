param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [string]$RestorePointer
)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/native-window.ps1" -AppProcessId $AppProcessId -Action stats | Out-Null
if ($RestorePointer) {
    $saved = $RestorePointer | ConvertFrom-Json
    $current = New-Object NativeControl+Point
    [NativeControl]::GetCursorPos([ref]$current) | Out-Null
    if ($current.X -eq $saved.placed.X -and $current.Y -eq $saved.placed.Y) {
        [NativeControl]::SetCursorPos($saved.previous.X, $saved.previous.Y) | Out-Null
    }
    return
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class FixtureWindows {
    public delegate bool Callback(IntPtr window, IntPtr data);
    [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr window, Callback callback, IntPtr data);
}
'@
$rootWindow = [NativeControl]::AppWindow($AppProcessId)
$editors = [System.Collections.Generic.List[NativeControl+Rect]]::new()
[FixtureWindows]::EnumChildWindows($rootWindow, {
    param($window, $data)
    [uint32]$owner = 0
    [NativeControl]::GetWindowThreadProcessId($window, [ref]$owner) | Out-Null
    $class = New-Object System.Text.StringBuilder 256
    [NativeControl]::GetClassName($window, $class, 256) | Out-Null
    if ($owner -eq $AppProcessId -and $class.ToString().Contains('EDIT')) {
        $bounds = New-Object NativeControl+Rect
        [NativeControl]::GetWindowRect($window, [ref]$bounds) | Out-Null
        $editors.Add($bounds)
    }
    return $true
}, [IntPtr]::Zero) | Out-Null
if ($editors.Count) {
    $bounds = $editors | Sort-Object Top | Select-Object -First 1
} else {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($rootWindow)
    $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document)
    $document = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    if ($null -eq $document) { throw 'Fixture has no document control.' }
    $rectangle = $document.Current.BoundingRectangle
    $bounds = @{ Left = $rectangle.Left; Top = $rectangle.Top; Right = $rectangle.Right; Bottom = $rectangle.Bottom }
}
$point = New-Object NativeControl+Point
$point.X = [int](($bounds.Left + $bounds.Right) / 2)
$point.Y = [int](($bounds.Top + $bounds.Bottom) / 2)
[uint32]$hitOwner = 0
[NativeControl]::GetWindowThreadProcessId([NativeControl]::WindowFromPoint($point), [ref]$hitOwner) | Out-Null
if ($hitOwner -ne $AppProcessId) { throw 'Fixture is covered; refusing to click another application.' }
$previous = New-Object NativeControl+Point
[NativeControl]::GetCursorPos([ref]$previous) | Out-Null
[NativeControl]::SetCursorPos($point.X, $point.Y) | Out-Null
$events = @(2, 4 | ForEach-Object {
    $input = New-Object NativeControl+Input
    $mouse = New-Object NativeControl+Mouse
    $mouse.Flags = $_
    $input.Mouse = $mouse
    $input
})
if ([NativeControl]::SendInput(2, $events, [Runtime.InteropServices.Marshal]::SizeOf($events[0])) -ne 2) {
    throw 'Windows rejected the fixture focus click.'
}
@{ previous = $previous; placed = $point } | ConvertTo-Json -Compress
