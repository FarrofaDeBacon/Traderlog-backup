$tempPath = [System.IO.Path]::GetTempPath()
$csvPath = Join-Path $tempPath "traderlog_rtd_data.csv"

# Header + Quote + Position
$csvContent = @"
TYPE;SYMBOL;LAST;OPEN;HIGH;LOW;CLOSE;VOLUME;TRADES;BID;ASK;SHEET
QUOTE;WINJ24;120500;120000;121000;119500;120200;500000;1500;120495;120505;Planilha1
POS;WINJ24;5;120450;;;;;;;;Planilha1
"@

Write-Host "Simulating RTD Position Entry: WINJ24 +5 contracts @ 120450"
$csvContent | Out-File -FilePath $csvPath -Encoding UTF8 -Force
Write-Host "Mock CSV created at $csvPath. App monitor should detect it in < 1s."
