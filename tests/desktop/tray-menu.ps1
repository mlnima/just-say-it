param(
    [Parameter(Mandatory = $true)][int]$AppProcessId,
    [Parameter(Mandatory = $true)][string]$Item
)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class TrayMenu {
    public delegate bool WindowCallback(IntPtr Window, IntPtr Data);
    [DllImport("user32.dll")] public static extern bool EnumWindows(WindowCallback Callback, IntPtr Data);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr Window, StringBuilder Name, int Count);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr Window, out uint Process);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr Window);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr Window, uint Message, IntPtr W, IntPtr L);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr Window, uint Message, IntPtr W, IntPtr L);
    [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr Menu);
    [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr Menu, int Position);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetMenuString(IntPtr Menu, uint Item, StringBuilder Name, int Count, uint Flags);
    public static IntPtr Find(uint Process, string Class, bool Visible) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((window, data) => {
            uint owner; GetWindowThreadProcessId(window, out owner);
            if (owner != Process || (Visible && !IsWindowVisible(window))) return true;
            var name = new StringBuilder(256); GetClassName(window, name, 256);
            if (name.ToString() != Class) return true;
            found = window;
            return false;
        }, IntPtr.Zero);
        return found;
    }
}
'@
$tray = [TrayMenu]::Find($AppProcessId, 'tray_icon_app', $false)
if ($tray -eq [IntPtr]::Zero) { throw 'The specified process has no tray window.' }
# Send the shell's right-button-release notification to the owned native tray window.
if (![TrayMenu]::PostMessage($tray, 6002, [IntPtr]::Zero, [IntPtr]0x205)) { throw 'Tray notification failed.' }
$deadline = [DateTime]::UtcNow.AddSeconds(5)
do {
    $popup = [TrayMenu]::Find($AppProcessId, '#32768', $true)
    if ($popup -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 50
} while ([DateTime]::UtcNow -lt $deadline)
if ($popup -eq [IntPtr]::Zero) { throw 'The native tray menu did not open.' }
$menu = [TrayMenu]::SendMessage($popup, 0x1E1, [IntPtr]::Zero, [IntPtr]::Zero)
$command = $null
for ($i = 0; $i -lt [TrayMenu]::GetMenuItemCount($menu); $i++) {
    $label = New-Object System.Text.StringBuilder 256
    [TrayMenu]::GetMenuString($menu, $i, $label, 256, 0x400) | Out-Null
    if ($label.ToString().Replace('&', '') -eq $Item) { $command = [TrayMenu]::GetMenuItemID($menu, $i); break }
}
if ($null -eq $command) { throw "Tray menu item not found: $Item" }
# Dispatch the selected native command through the real menu handler, after dismissing the popup.
[TrayMenu]::PostMessage($tray, 0x1F, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
if (![TrayMenu]::PostMessage($tray, 0x111, [IntPtr]$command, [IntPtr]::Zero)) { throw 'Native menu command failed.' }
Write-Output "Selected native tray menu item: $Item"
