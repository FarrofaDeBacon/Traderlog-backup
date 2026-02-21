$tempPath = [System.IO.Path]::GetTempPath()
$csvPath = Join-Path $tempPath "traderlog_rtd_data.csv"

function Write-Mock-CSV($trades) {
    $content = "TYPE;SYMBOL;LAST;OPEN;HIGH;LOW;CLOSE;VOLUME;TRADES;BID;ASK;SHEET`n"
    $content += "QUOTE;WIN;128500;128000;129000;127500;128000;500000;$trades;0;0;RTD_REAL"
    $content | Set-Content -Path $csvPath -Force
    Write-Host "Wrote Mock CSV with $trades trades to $csvPath"
}

Write-Host "--- RTD Mock Tester ---"
Write-Mock-CSV 10
Start-Sleep -Seconds 3
Write-Mock-CSV 11 # Trigger detection
Start-Sleep -Seconds 3
Write-Mock-CSV 12 # Trigger second detection
