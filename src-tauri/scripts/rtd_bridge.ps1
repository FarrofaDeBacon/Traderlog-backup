# rtd_bridge.ps1
# Universal Bridge for Excel RTD -> TraderLog CSV
Param([string]$ExcelPath)

$currentPid = $PID
Get-Process powershell -ErrorAction SilentlyContinue | Where-Object { 
    $_.Id -ne $currentPid -and 
    (Get-CimInstance Win32_Process -Filter "ProcessId = $($_.Id)" -ErrorAction SilentlyContinue).CommandLine -like "*rtd_bridge.ps1*"
} | Stop-Process -Force -ErrorAction SilentlyContinue

$tempPath = [System.IO.Path]::GetTempPath()
$csvPath = Join-Path $tempPath "traderlog_rtd_data.csv"
$debugLog = "C:\PROJETOS\TraderLogPro\rtd_debug.log"

function Log($msg) {
    "[$(Get-Date -Format 'HH:mm:ss')] $msg" | Out-File -FilePath $debugLog -Append
}

Log "--- RTD Universal Bridge Started ---"

function Get-SafeValue($val) {
    if ($null -eq $val) { return 0 }
    $vStr = [string]$val
    if ($vStr -match "-214682[0-9]+") { return 0 }
    if ($vStr -match "#N/A|#VALOR|#REF|#DIV/0|#NUM|#NOME|#NULO") { return 0 }
    $out = 0
    if ($vStr -match ",") {
        $vStr = $vStr.Replace(".", "").Replace(",", ".")
    }
    if ([double]::TryParse($vStr, [System.Globalization.NumberStyles]::Any, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$out)) { return $out }
    return 0
}

while ($true) {
    try {
        $excel = [Runtime.InteropServices.Marshal]::GetActiveObject("Excel.Application")
        if ($null -ne $excel) {
            $workbook = $null
            foreach ($wb in $excel.Workbooks) {
                if ($wb.FullName -eq $ExcelPath -or $wb.Name -match "Profit_RTD") { $workbook = $wb; break }
            }
            if ($null -eq $workbook) { $workbook = $excel.ActiveWorkbook }

            if ($null -ne $workbook) {
                # Log "Scanning workbook: $($workbook.Name)"
                $aggregatedData = @()
                $aggregatedData += "TYPE;SYMBOL;LAST;OPEN;HIGH;LOW;CLOSE;VOLUME;TRADES;BID;ASK"
                
                foreach ($sheet in $workbook.Worksheets) {
                    $rawName = [string]$sheet.Name
                    
                    # USE Value2 from Range A1:AZ100 for better performance
                    $range = $sheet.Range("A1:AZ100")
                    $data = $range.Value2 
                    if ($null -eq $data) { continue }
                    
                    $colMap = @{}
                    $colsCount = $data.GetUpperBound(1)
                    for ($c = 1; $c -le $colsCount; $c++) {
                        $h = [string]$data[1, $c]
                        if ($h) {
                            $h = $h.Trim().ToUpper()
                            if ($null -eq $colMap.SYM -and ($h -match "^ASSET$|^SYMBOL$|^ATIVO$")) { $colMap["SYM"] = $c }
                            if ($h -match "ULTIMO|LAST") { $colMap["LAST"] = $c }
                            if ($h -match "NEGOC") { $colMap["TRADES"] = $c }
                            if ($h -match "PSICOTRADE") { $colMap["POS"] = $c }
                            if ($null -eq $colMap.POS -and ($h -match "QUANTIDADE")) { $colMap["POS"] = $c }
                            if ($h -match "MEDIO|AVG") { $colMap["AVG"] = $c }
                        }
                    }

                    if ($null -eq $colMap.SYM) { continue }

                    for ($r = 2; $r -le $data.GetUpperBound(0); $r++) {
                        $sym = [string]$data[$r, $colMap.SYM]
                        if ($null -eq $sym -or $sym.Length -lt 2) { continue }
                        $sym = $sym.Trim().ToUpper()
                        if ($sym -match "SYMBOL|ATIVO|SIMBOLO") { continue }
                        
                        $last = if ($colMap.LAST) { Get-SafeValue $data[$r, $colMap.LAST] } else { 0 }
                        $trades = if ($colMap.TRADES) { Get-SafeValue $data[$r, $colMap.TRADES] } else { 0 }
                        $pos = if ($colMap.POS) { Get-SafeValue $data[$r, $colMap.POS] } else { 0 }
                        $avg = if ($colMap.AVG) { Get-SafeValue $data[$r, $colMap.AVG] } else { 0 }

                        # STRICT FILTER: Price must be > 0.01 to be considered a valid trade/quote
                        if ($last -gt 0.01) {
                             $rLast = [Math]::Round($last, 4)
                             $rTrades = [Math]::Round($trades, 0)
                             $aggregatedData += "QUOTE;$sym;$rLast;0;0;0;0;0;$rTrades;0;0;$rawName"
                             
                             if ($colMap.POS -and [Math]::Abs($pos) -gt 0.001) {
                                 $rPos = [Math]::Round($pos, 4)
                                 $rAvg = [Math]::Round($avg, 4)
                                 $aggregatedData += "POS;$sym;$([Math]::Abs($rPos));$rAvg;0;0;0;0;0;0;0;$rawName"
                             }
                        }
                    }
                }

                if ($aggregatedData.Count -gt 1) {
                    $tmpCsv = $csvPath + ".tmp"
                    $aggregatedData | Set-Content -Path $tmpCsv -Force -Encoding utf8
                    Move-Item -Path $tmpCsv -Destination $csvPath -Force -ErrorAction SilentlyContinue
                    # Log "CSV Updated: $($aggregatedData.Count) lines"
                }
            }
        }
    }
    catch {
        $e = $_.Exception.Message
        if ($e -notmatch "0x8001010A|0x800A01A8|MK_E_UNAVAILABLE") {
            Log "ERROR: $e"
        }
    }
    
    Start-Sleep -Milliseconds 500
}
