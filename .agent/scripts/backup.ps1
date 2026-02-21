$date = Get-Date -Format "yyyyMMdd_HHmm"
$backupDir = "c:\PROJETOS\TraderLogPro\backup"
$dest = Join-Path $backupDir "TraderLogPro_$date.zip"

if (!(Test-Path $backupDir)) {
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
}

Write-Host "Creating backup: $dest"

# Delete old backups
# # # Get-ChildItem -Path $backupDir -Filter "*.zip" | Remove-Item -Force

$files = Get-ChildItem -Path "c:\PROJETOS\TraderLogPro\*" -Exclude "node_modules", ".svelte-kit", "backup", ".git", "build", "target", "src-tauri/target"

Compress-Archive -Path $files.FullName -DestinationPath $dest

Write-Host "Backup created successfully: $dest"
