param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [ValidateSet('click', 'close', 'stats', 'window', 'focus')][string]$Action = 'stats',
    [string]$AllowedForegroundProcesses = '[]'
)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeControl {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct Mouse { public int X,Y; public uint Data, Flags, Time; public UIntPtr Extra; }
    [StructLayout(LayoutKind.Sequential)] public struct Input { public uint Type; public Mouse Mouse; }
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string Class, string Text);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr Window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr Window);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr Window, out uint Process);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr Window, out Rect Bounds);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point Point);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point Point);
    [DllImport("user32.dll")] public static extern uint SendInput(uint Count, Input[] Inputs, int Size);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr Window, uint Message, IntPtr W, IntPtr L);
    public delegate bool WindowCallback(IntPtr Window, IntPtr Data);
    [DllImport("user32.dll")] public static extern bool EnumWindows(WindowCallback Callback, IntPtr Data);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr Window, System.Text.StringBuilder Name, int Count);
    public static IntPtr AppWindow(uint Process) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((window, data) => {
            uint owner; GetWindowThreadProcessId(window, out owner);
            if (owner != Process || !IsWindowVisible(window)) return true;
            var name = new System.Text.StringBuilder(256); GetClassName(window, name, 256);
            Rect bounds; GetWindowRect(window, out bounds);
            if (name.ToString().EndsWith("-sic") || name.ToString() == "DictationInputControl" || name.ToString() == "Tao Thread Event Target" || bounds.Right <= bounds.Left || bounds.Bottom <= bounds.Top) return true;
            found = window; return false;
        }, IntPtr.Zero);
        return found;
    }
}
'@
$process = Get-Process -Id $AppProcessId
if ($Action -eq 'focus') {
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    $window = [NativeControl]::AppWindow($AppProcessId)
    if ($window -eq [IntPtr]::Zero) { throw 'The app has no settings window to focus.' }
    [NativeControl]::SetForegroundWindow($window) | Out-Null
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($window)
    $nameCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, 'Wake phrase')
    $typeCondition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)
    $condition = New-Object System.Windows.Automation.AndCondition($nameCondition, $typeCondition)
    $input = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    if ($null -eq $input) { throw 'The owned settings wake field was not found.' }
    $input.SetFocus()
    [uint32]$foregroundOwner = 0
    [NativeControl]::GetWindowThreadProcessId([NativeControl]::GetForegroundWindow(), [ref]$foregroundOwner) | Out-Null
    if ($foregroundOwner -ne $AppProcessId) { throw 'The owned settings window did not receive foreground focus.' }
    return
}
if ($Action -eq 'window') {
    $appWindow = [NativeControl]::AppWindow($AppProcessId)
    $windowClass = New-Object System.Text.StringBuilder 256
    [NativeControl]::GetClassName($appWindow, $windowClass, 256) | Out-Null
    $windowBounds = New-Object NativeControl+Rect
    [NativeControl]::GetWindowRect($appWindow, [ref]$windowBounds) | Out-Null
    $inputButton = [NativeControl]::FindWindow('DictationInputControl', 'Dictation')
    [uint32]$foregroundOwner = 0
    [NativeControl]::GetWindowThreadProcessId([NativeControl]::GetForegroundWindow(), [ref]$foregroundOwner) | Out-Null
    @{ visible_settings_window = $appWindow -ne [IntPtr]::Zero; window_class = $windowClass.ToString(); bounds = $windowBounds; input_button_visible = [NativeControl]::IsWindowVisible($inputButton); foreground_process_id = $foregroundOwner } | ConvertTo-Json -Compress
    return
}
if ($Action -eq 'stats') {
    @{ cpu_seconds = $process.CPU; working_set_bytes = $process.WorkingSet64; private_bytes = $process.PrivateMemorySize64 } | ConvertTo-Json -Compress
    exit
}
if ($Action -eq 'close') {
    $settingsWindow = [NativeControl]::AppWindow($AppProcessId)
    if ($settingsWindow -eq [IntPtr]::Zero) { throw 'The app has no settings window.' }
    [NativeControl]::PostMessage($settingsWindow, 16, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    exit
}
$allowed = $AllowedForegroundProcesses | ConvertFrom-Json
$overlay = [NativeControl]::FindWindow('DictationInputControl', 'Dictation')
$deadline = [DateTime]::UtcNow.AddSeconds(5)
while (-not [NativeControl]::IsWindowVisible($overlay)) {
    if ([DateTime]::UtcNow -gt $deadline) { throw 'The actual input button did not become visible.' }
    Start-Sleep -Milliseconds 50
}
[uint32]$owner = 0
[NativeControl]::GetWindowThreadProcessId($overlay, [ref]$owner) | Out-Null
if ($owner -ne $AppProcessId) { throw 'Input button belongs to a different process.' }
$foreground = [NativeControl]::GetForegroundWindow()
[uint32]$foregroundProcess = 0
[NativeControl]::GetWindowThreadProcessId($foreground, [ref]$foregroundProcess) | Out-Null
if ($allowed -notcontains $foregroundProcess) { throw 'The browser fixture is not focused; refusing to click.' }
$bounds = New-Object NativeControl+Rect
[NativeControl]::GetWindowRect($overlay, [ref]$bounds) | Out-Null
$point = New-Object NativeControl+Point
$point.X = [int](($bounds.Left + $bounds.Right) / 2)
$point.Y = [int](($bounds.Top + $bounds.Bottom) / 2)
if ([NativeControl]::WindowFromPoint($point) -ne $overlay) { throw 'The input button is covered; refusing to click.' }
$previous = New-Object NativeControl+Point
[NativeControl]::GetCursorPos([ref]$previous) | Out-Null
[NativeControl]::SetCursorPos($point.X, $point.Y) | Out-Null
$down = New-Object NativeControl+Input
$downMouse = New-Object NativeControl+Mouse
$downMouse.Flags = 2
$down.Mouse = $downMouse
$up = New-Object NativeControl+Input
$upMouse = New-Object NativeControl+Mouse
$upMouse.Flags = 4
$up.Mouse = $upMouse
if ($down.Mouse.Flags -ne 2 -or $up.Mouse.Flags -ne 4) { throw 'Mouse event construction failed.' }
$sent = [NativeControl]::SendInput(2, @($down, $up), [Runtime.InteropServices.Marshal]::SizeOf($down))
if ($sent -ne 2) { throw 'Windows rejected the fixture button click.' }
Start-Sleep -Milliseconds 150
if ([NativeControl]::GetForegroundWindow() -ne $foreground) { throw 'The input button took focus.' }
$current = New-Object NativeControl+Point
[NativeControl]::GetCursorPos([ref]$current) | Out-Null
if ($current.X -eq $point.X -and $current.Y -eq $point.Y) { [NativeControl]::SetCursorPos($previous.X, $previous.Y) | Out-Null }
Write-Output 'Actual input button clicked; foreground window preserved.'
