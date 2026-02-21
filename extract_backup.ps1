Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipPath = "c:/PROJETOS/TraderLogPro/backup/TraderLogPro_20260211_0855.zip"
$destPath = "c:/PROJETOS/TraderLogPro/psychology_backup.svelte"
$zip = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
$entry = $zip.Entries | Where-Object { $_.FullName -eq "routes\(app)\psychology\+page.svelte" }
if ($entry) {
    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $destPath, $true)
    Write-Output "SUCCESS"
}
else {
    Write-Output "NOT FOUND"
}
$zip.Dispose()
