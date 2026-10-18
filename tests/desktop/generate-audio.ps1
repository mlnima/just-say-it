param([string]$OutputDirectory = 'artifacts/test-audio')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$format = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo(16000, [System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen, [System.Speech.AudioFormat.AudioChannel]::Mono)
$samples = @{
    'wake-one.wav' = 'Hello little notebook'
    'wake-two.wav' = 'Good morning computer'
    'dictation.wav' = 'Please write this sentence in my notebook.'
    'ordinary-stop.wav' = 'I would like you to finish these words before we leave.'
    'stop.wav' = 'Finish these words'
}
foreach ($entry in $samples.GetEnumerator()) {
    $synthesizer = New-Object System.Speech.Synthesis.SpeechSynthesizer
    $synthesizer.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::NotSet, [System.Speech.Synthesis.VoiceAge]::NotSet, 0, [System.Globalization.CultureInfo]::GetCultureInfo('en-US'))
    $path = [System.IO.Path]::GetFullPath((Join-Path $OutputDirectory $entry.Key))
    $synthesizer.SetOutputToWaveFile($path, $format)
    $synthesizer.Speak($entry.Value)
    $synthesizer.Dispose()
}
Write-Output 'Synthetic English fixtures generated locally; no microphone recording was retained.'
