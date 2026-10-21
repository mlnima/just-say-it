param(
    [Parameter(Mandatory = $true)][int]$FixtureProcessId,
    [Parameter(Mandatory = $true)][ValidateSet('Space', 'F9')][string]$Key
)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ShortcutInput {
    [StructLayout(LayoutKind.Sequential)] public struct Keyboard {
        public ushort Key, Scan;
        public uint Flags, Time;
        public UIntPtr Extra;
    }
    [StructLayout(LayoutKind.Sequential)] public struct Mouse {
        public int X, Y;
        public uint Data, Flags, Time;
        public UIntPtr Extra;
    }
    [StructLayout(LayoutKind.Explicit)] public struct Data {
        [FieldOffset(0)] public Keyboard Keyboard;
        [FieldOffset(0)] public Mouse Mouse;
    }
    [StructLayout(LayoutKind.Sequential)] public struct Input {
        public uint Type;
        public Data Data;
    }
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int key);
    [DllImport("user32.dll")] public static extern uint SendInput(uint count, Input[] inputs, int size);
    public static void Press(uint fixture, ushort key) {
        uint owner;
        GetWindowThreadProcessId(GetForegroundWindow(), out owner);
        if (owner != fixture) throw new InvalidOperationException("The owned fixture is not focused.");
        foreach (int modifier in new[] { 0x10, 0x11, 0x12, 0x5B, 0x5C }) {
            if (GetAsyncKeyState(modifier) < 0) throw new InvalidOperationException("Release held modifiers first.");
        }
        ushort[] keys = { 0x11, 0x10, key, key, 0x10, 0x11 };
        var inputs = new Input[keys.Length];
        for (int i = 0; i < keys.Length; i++) {
            inputs[i].Type = 1;
            inputs[i].Data.Keyboard.Key = keys[i];
            inputs[i].Data.Keyboard.Flags = i < 3 ? 0u : 2u;
        }
        if (SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(Input))) != inputs.Length) {
            throw new InvalidOperationException("Windows rejected the shortcut events.");
        }
    }
}
'@
Get-Process -Id $FixtureProcessId -ErrorAction Stop | Out-Null
$keyCode = if ($Key -eq 'Space') { 0x20 } else { 0x78 }
[ShortcutInput]::Press($FixtureProcessId, $keyCode)
Write-Output "Dispatched Control+Shift+$Key to the owned fixture."
