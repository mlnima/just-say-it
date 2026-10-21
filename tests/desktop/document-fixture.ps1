param([Parameter(Mandatory = $true)][string]$Directory)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName PresentationFramework
$window = New-Object System.Windows.Window
$window.Title = 'Dictation document test fixture'
$window.Width = 560
$window.Height = 370
$window.WindowStartupLocation = 'CenterScreen'
$window.Topmost = $true
$panel = New-Object System.Windows.Controls.StackPanel
$window.Content = $panel
$fields = @(0..2 | ForEach-Object {
    $field = New-Object System.Windows.Controls.RichTextBox
    $field.Height = 90
    $field.Margin = '12'
    [System.Windows.Automation.AutomationProperties]::SetName($field, "Test document $_")
    $panel.Children.Add($field) | Out-Null
    $field
})
$fields[2].IsReadOnly = $true
$timer = New-Object System.Windows.Threading.DispatcherTimer
$timer.Interval = [TimeSpan]::FromMilliseconds(50)
$commandFile = Join-Path $Directory 'command.json'
$timer.Add_Tick({
    if (-not (Test-Path -LiteralPath $commandFile)) { return }
    try { $command = Get-Content -LiteralPath $commandFile -Raw | ConvertFrom-Json } catch { return }
    if ($command.action -eq 'focus') {
        $window.Activate() | Out-Null
        $fields[$command.index].Focus() | Out-Null
    }
    if ($command.action -eq 'read') {
        $first = New-Object System.Windows.Documents.TextRange($fields[0].Document.ContentStart, $fields[0].Document.ContentEnd)
        $second = New-Object System.Windows.Documents.TextRange($fields[1].Document.ContentStart, $fields[1].Document.ContentEnd)
        @{ first = $first.Text.TrimEnd("`r`n"); second = $second.Text.TrimEnd("`r`n") } |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Directory 'result.json') -Encoding utf8
    }
    Remove-Item -LiteralPath $commandFile
    if ($command.action -eq 'close') { $window.Close() }
})
$window.Add_ContentRendered({
    $window.Activate() | Out-Null
    $fields[0].Focus() | Out-Null
    Set-Content -LiteralPath (Join-Path $Directory 'ready') -Value 'ready'
    $timer.Start()
})
$window.ShowDialog() | Out-Null
$timer.Stop()
