$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
try {
    $wb = $excel.Workbooks.Open("C:\PROJETOS\TraderLogPro\Profit_RTD.xlsx")
    foreach ($ws in $wb.Worksheets) {
        $name = $ws.Name
        $headers = @()
        for ($c = 1; $c -le 20; $c++) {
            $val = $ws.Cells.Item(1, $c).Value2
            if ($null -ne $val) { $headers += [string]$val }
        }
        Write-Output "SHEET: $name | HEADERS: $($headers -join '; ')"
    }
    $wb.Close($false)
}
finally {
    $excel.Quit()
}
