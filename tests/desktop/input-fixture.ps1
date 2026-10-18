param([Parameter(Mandatory = $true)][string]$Directory)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
$window = New-Object System.Windows.Forms.Form
$window.Text = 'Dictation adapter test fixture'
$window.Size = New-Object System.Drawing.Size(520, 310)
$window.StartPosition = 'CenterScreen'
$window.TopMost = $true
$fields = @()
for ($position = 0; $position -lt 4; $position++) {
    $field = New-Object System.Windows.Forms.TextBox
    $field.Location = New-Object System.Drawing.Point(30, (30 + 48 * $position))
    $field.Size = New-Object System.Drawing.Size(440, 30)
    $field.AccessibleName = "Test input $position"
    $window.Controls.Add($field)
    $fields += $field
}
$fields[2].UseSystemPasswordChar = $true
$fields[3].ReadOnly = $true
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 50
$commandFile = Join-Path $Directory 'command.json'
$resultFile = Join-Path $Directory 'result.json'
$readyFile = Join-Path $Directory 'ready'
$timer.Add_Tick({
    if (Test-Path -LiteralPath $commandFile) {
        try { $command = Get-Content -LiteralPath $commandFile -Raw | ConvertFrom-Json } catch { return }
        Remove-Item -LiteralPath $commandFile
        if ($command.action -eq 'focus') {
            $window.Activate()
            $fields[$command.index].Focus() | Out-Null
        }
        if ($command.action -eq 'read') {
            @{ first = $fields[0].Text; second = $fields[1].Text } | ConvertTo-Json | Set-Content -LiteralPath $resultFile -Encoding utf8
        }
        if ($command.action -eq 'close') { $window.Close() }
    }
})
$window.Add_Shown({ $window.Activate(); $fields[0].Focus() | Out-Null; Set-Content -LiteralPath $readyFile -Value 'ready'; $timer.Start() })
[System.Windows.Forms.Application]::Run($window)
$timer.Dispose()
$window.Dispose()
