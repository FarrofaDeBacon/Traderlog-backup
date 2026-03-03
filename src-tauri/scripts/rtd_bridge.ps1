# silent_rtd_bridge.ps1
# This script bridges Excel RTD data to a local CSV file for the Tauri application.

Param(
    [string]$ExcelPath
)

# 0. Self-Cleanup: Kill any other running instances of this script to avoid resource conflicts
$currentPid = $PID
Get-Process powershell -ErrorAction SilentlyContinue | Where-Object { 
    $_.Id -ne $currentPid -and 
    (Get-CimInstance Win32_Process -Filter "ProcessId = $($_.Id)" -ErrorAction SilentlyContinue).CommandLine -like "*rtd_bridge.ps1*"
} | Stop-Process -Force -ErrorAction SilentlyContinue

# $ErrorActionPreference = "SilentlyContinue"
$tempPath = [System.IO.Path]::GetTempPath()
$csvPath = Join-Path $tempPath "traderlog_rtd_data.csv"
$debugLog = Join-Path $tempPath "traderlog_rtd_debug.log"

"--- RTD Bridge Started $(Get-Date) ---" | Out-File -FilePath $debugLog -Force
Write-Host "RTD Bridge Started... Logging to $debugLog"

$script:lastTradesMap = @{}
$script:lastQtyMap = @{}

function Get-SafeValue($val) {
    if ($null -eq $val) { return 0 }
    $vStr = [string]$val
    # suppress COM errors
    if ($vStr -match "-214682[0-9]+") { return 0 }
    if ($vStr -match "#N/A|#VALOR|#REF|#DIV/0|#NUM|#NOME|#NULO") { return 0 }
    if ($vStr -match "[0-9]{2}/[0-9]{2}/[0-9]{4}") { return 0 } # skip dates
    if ($vStr -match "[0-9]{2}:[0-9]{2}:[0-9]{2}") { return 0 } # skip times
    
    if ($val -is [double] -or $val -is [int]) { return $val }
    
    # Try parse if string
    $out = 0
    if ([double]::TryParse($vStr, [ref]$out)) { return $out }
    return 0
}

while ($true) {
    try {
        $excel = [Runtime.InteropServices.Marshal]::GetActiveObject("Excel.Application")
        if ($excel) {
            $workbook = $null
            $targetPath = "C:\PROJETOS\TraderLogPro\Profit_RTD.xlsx"
            
            foreach ($wb in $excel.Workbooks) {
                if ($wb.FullName -eq $targetPath -or $wb.Name -match "Profit_RTD") {
                    $workbook = $wb
                    break
                }
            }
            
            if (!$workbook) {
                $workbook = $excel.ActiveWorkbook
            }

            if ($workbook) {
                $aggregatedData = @()
                $aggregatedData += "TYPE;SYMBOL;LAST;OPEN;HIGH;LOW;CLOSE;VOLUME;TRADES;BID;ASK"
                
                $sheetIdx = 0
                foreach ($sheet in $workbook.Worksheets) {
                    $sheetIdx++
                    # Normalize sheet name by removing non-ASCII characters for robust matching
                    $rawName = [string]$sheet.Name
                    $normalizedName = $rawName -replace '[^a-zA-Z0-9]', '.'
                    
                    $msg = "[$(Get-Date -Format 'HH:mm:ss')] DEBUG: Checking sheet '$normalizedName' (Raw: $rawName)"
                    $msg | Out-File -FilePath $debugLog -Append
                    
                    # 1. Quotes (Match Ações -> A..es, Futuros, OR just take first 2 sheets as fallback)
                    if ($normalizedName -match "A..es|Acoes|Futuros|DADOS|Ativos|Market|LISTA|Stocks|RTD" -or $sheetIdx -le 2) {
                        $msg = "[$(Get-Date -Format 'HH:mm:ss')] INFO: => Processing Quote Sheet: $rawName"
                        $msg | Out-File -FilePath $debugLog -Append
                        
                        $range = $sheet.Range("A1:Z500")
                        $data = $range.Value2
                        if ($null -eq $data) { 
                            "[$(Get-Date -Format 'HH:mm:ss')] DEBUG: Sheet $rawName is empty" | Out-File -FilePath $debugLog -Append
                            continue 
                        }
                        
                        # Find headers dynamically
                        $colMap = @{}
                        for ($c = 1; $c -le $data.GetUpperBound(1); $c++) {
                            $h = [string]$data[1, $c]
                            if ($null -ne $h) {
                                $h = $h.Trim().ToUpper()
                                if ($h -match "ATIVO|SYMB|ASSET") { $colMap["SYM"] = $c }
                                elseif ($h -match "ULTIMO|LAST") { $colMap["LAST"] = $c }
                                elseif ($h -match "ABERT|OPEN") { $colMap["OPEN"] = $c }
                                elseif ($h -match "MAX|HIGH") { $colMap["HIGH"] = $c }
                                elseif ($h -match "MIN|LOW") { $colMap["LOW"] = $c }
                                elseif ($h -match "FECH|CLOSE") { $colMap["CLOSE"] = $c }
                                elseif ($h -match "NEGOC|TRADE") { $colMap["TRADES"] = $c }
                                elseif ($h -match "VOL") { $colMap["VOL"] = $c }
                                # Custom Indicator Detection
                                elseif ($h -match "PSICOTRADEMONITOR.*PLOT1") { $colMap["POS"] = $c }
                                elseif ($h -match "PSICOTRADEMONITOR.*PLOT2") { $colMap["AVG"] = $c }
                            }
                        }

                        $maxR = $data.GetUpperBound(0)
                        $count = 0
                        for ($r = 2; $r -le $maxR; $r++) {
                            $val = if ($colMap.SYM) { [string]$data[$r, $colMap.SYM] } else { [string]$data[$r, 1] }
                            if ($null -eq $val -or $val.Trim().Length -lt 2) { continue }
                            $sym = $val.Trim().ToUpper()
                            if ($sym -match "S[ií]mbolo|Symbol|Tipo|Type|Ativo") { continue }
                            
                            $last = if ($colMap.LAST) { Get-SafeValue $data[$r, $colMap.LAST] } else { Get-SafeValue $data[$r, 4] }
                            $open = if ($colMap.OPEN) { Get-SafeValue $data[$r, $colMap.OPEN] } else { Get-SafeValue $data[$r, 5] }
                            $high = if ($colMap.HIGH) { Get-SafeValue $data[$r, $colMap.HIGH] } else { Get-SafeValue $data[$r, 6] }
                            $low = if ($colMap.LOW) { Get-SafeValue $data[$r, $colMap.LOW] } else { Get-SafeValue $data[$r, 7] }
                            $close = if ($colMap.CLOSE) { Get-SafeValue $data[$r, $colMap.CLOSE] } else { Get-SafeValue $data[$r, 8] }
                            $trades = if ($colMap.TRADES) { Get-SafeValue $data[$r, $colMap.TRADES] } else { Get-SafeValue $data[$r, 14] }
                            $vol = if ($colMap.VOL) { Get-SafeValue $data[$r, $colMap.VOL] } else { Get-SafeValue $data[$r, 16] }

                            if ($last -gt 0) {
                                # Log trade count change if noticed
                                $prevTrades = $script:lastTradesMap[$sym]
                                if ($null -ne $prevTrades -and $trades -gt $prevTrades) {
                                    $msg = "[$(Get-Date -Format 'HH:mm:ss')] !!! TRADE DETECTED: $sym ($prevTrades -> $trades)"
                                    $msg | Out-File -FilePath $debugLog -Append
                                }
                                $script:lastTradesMap[$sym] = $trades

                                $line = "QUOTE;$sym;$last;$open;$high;$low;$close;$vol;$trades;0;0;$rawName"
                                $aggregatedData += $line
                                
                                # If we found the custom indicator columns, also emit a POS update
                                if ($colMap.POS -and $colMap.AVG) {
                                    $posQty = Get-SafeValue $data[$r, $colMap.POS]
                                    $posAvg = Get-SafeValue $data[$r, $colMap.AVG]
                                    
                                    # Normalize posQty (bridge expects abs for detection, or we can use signed later)
                                    $absQty = [Math]::Abs($posQty)
                                    
                                    $line = "POS;$sym;$absQty;$posAvg;0;0;0;0;0;0;0;$rawName"
                                    $aggregatedData += $line
                                    
                                    # Log position change for debugging
                                    $prevQty = $script:lastQtyMap[$sym]
                                    if ($null -ne $prevQty -and $absQty -ne $prevQty) {
                                        $msg = "[$(Get-Date -Format 'HH:mm:ss')] !!! NTSL POSITION CHANGE: $sym ($prevQty -> $absQty)"
                                        $msg | Out-File -FilePath $debugLog -Append
                                    }
                                    $script:lastQtyMap[$sym] = $absQty
                                }
                                
                                $count++
                            }
                        }
                        "[$(Get-Date -Format 'HH:mm:ss')] INFO: Finished sheet $rawName ($count symbols found)" | Out-File -FilePath $debugLog -Append
                    }
                
                    # 2. Book Depth (Simple pattern)
                    elseif ($normalizedName -match "BOOK") {
                        $msg = "[$(Get-Date -Format 'HH:mm:ss')] INFO: => Processing Book Sheet: $rawName"
                        $msg | Out-File -FilePath $debugLog -Append
                        $range = $sheet.Range("A1:B30")
                        $data = $range.Value2
                        if ($null -eq $data) { continue }
                        
                        $totalBid = 0
                        $totalAsk = 0
                        for ($r = 1; $r -le $data.GetUpperBound(0); $r++) {
                            $totalBid += Get-SafeValue $data[$r, 1]
                            $totalAsk += Get-SafeValue $data[$r, 2]
                        }
                        $id = "BOOK_" + $rawName
                        $aggregatedData += "BOOK;$id;0;0;0;0;0;0;0;$totalBid;$totalAsk"
                    }
                    # 3. Position Data (Your actual trades)
                    elseif ($normalizedName -match "POSICAO|POSITION|EXTRATO") {
                        $msg = "[$(Get-Date -Format 'HH:mm:ss')] INFO: => Processing Position Sheet: $rawName"
                        $msg | Out-File -FilePath $debugLog -Append
                    
                        $range = $sheet.Range("A1:C50") # Asset, Qty, Price
                        $data = $range.Value2
                        if ($null -eq $data) { continue }
                    
                        for ($r = 2; $r -le $data.GetUpperBound(0); $r++) {
                            $sym = [string]$data[$r, 1]
                            if ($null -eq $sym -or $sym.Trim().Length -lt 2) { continue }
                            $sym = $sym.Trim().ToUpper()
                        
                            $qty = Get-SafeValue $data[$r, 2]
                            $avg = Get-SafeValue $data[$r, 3]
                        
                            # Log position change for debugging
                            $prevQty = $script:lastQtyMap[$sym]
                            if ($null -ne $prevQty -and $qty -ne $prevQty) {
                                $msg = "[$(Get-Date -Format 'HH:mm:ss')] !!! POSITION CHANGE: $sym ($prevQty -> $qty)"
                                $msg | Out-File -FilePath $debugLog -Append
                            }
                            $script:lastQtyMap[$sym] = $qty

                            $aggregatedData += "POS;$sym;$qty;$avg;0;0;0;0;0;0;0;$rawName"
                        }
                    }
                }
            
                if ($aggregatedData.Count -gt 1) {
                    $aggregatedData | Set-Content -Path $csvPath -Force -Encoding utf8
                }
            }
        }
    }
    catch {
        $msg = "[$(Get-Date -Format 'HH:mm:ss')] ERROR: $($_.Exception.Message)"
        if ($_.Exception.Message -match "0x8001010A" -or $_.Exception.Message -match "0x800A01A8") {
            $msg = "[$(Get-Date -Format 'HH:mm:ss')] CRITICAL: Excel Busy or Object Disconnected. Retrying..."
        }
        $msg | Out-File -FilePath $debugLog -Append
        
        # If Excel process is gone, wait longer or signal retry
        $excelProcs = Get-Process excel -ErrorAction SilentlyContinue
        if (!$excelProcs) {
            "[$(Get-Date -Format 'HH:mm:ss')] INFO: Excel not running. Waiting for process..." | Out-File -FilePath $debugLog -Append
            Start-Sleep -Seconds 5
        }
    }
    
    # Cleanup stale CSV if bridge hasn't updated in 10s (prevents ghost data)
    if (Test-Path $csvPath) {
        $lastWrite = (Get-Item $csvPath).LastWriteTime
        if ((Get-Date) - $lastWrite -gt [TimeSpan]::FromSeconds(10)) {
            Remove-Item $csvPath -Force -ErrorAction SilentlyContinue
            "[$(Get-Date -Format 'HH:mm:ss')] DEBUG: Stale CSV removed." | Out-File -FilePath $debugLog -Append
        }
    }

    Start-Sleep -Seconds 1
}
