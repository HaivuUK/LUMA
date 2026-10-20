# WiX's license dialog needs RTF. Converts a plain-text license to RTF.
param([string]$In, [string]$Out)
$lines = Get-Content -LiteralPath $In -Encoding UTF8 | ForEach-Object {
  $e = $_.Replace('\', '\\').Replace('{', '\{').Replace('}', '\}')
  $e = [regex]::Replace($e, '[^\x00-\x7F]', [System.Text.RegularExpressions.MatchEvaluator]{
    param($m)
    $c = [int][char]$m.Value
    if ($c -gt 32767) { $c -= 65536 }
    "\u$c?"
  })
  "$e\par"
}
$rtf = "{\rtf1\ansi\deff0{\fonttbl{\f0\fmodern Consolas;}}\f0\fs18`r`n" + ($lines -join "`r`n") + "`r`n}"
Set-Content -LiteralPath $Out -Value $rtf -Encoding ASCII
