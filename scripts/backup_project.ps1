$timestamp = Get-Date -Format "yyyyMMdd_HHmm"
$zipPath = "c:\PROJETOS\TraderLogPro\backup\traderlog_pro_backup_$timestamp.zip"
$sourcePath = "c:\PROJETOS\TraderLogPro"

Write-Host "Starting backup to $zipPath..."

$files = Get-ChildItem -Path $sourcePath -Recurse | Where-Object { 
    $_.FullName -notmatch "node_modules|\\.svelte-kit|src-tauri[\\/]target|\\.git|backup" 
}

if ($files) {
    Compress-Archive -Path $files.FullName -DestinationPath $zipPath -Force
    Write-Host "Backup created successfully at $zipPath"
} else {
    Write-Error "No files found to backup!"
}
