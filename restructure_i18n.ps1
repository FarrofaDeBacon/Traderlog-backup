
function Restructure-Json {
    param($filePath)
    if (-not (Test-Path $filePath)) { Write-Host "File not found: $filePath"; return }
    
    $jsonContent = Get-Content $filePath -Raw -Encoding utf8 | ConvertFrom-Json
    
    # 1. Prepare segments
    $settings = $jsonContent.settings
    $api = $settings.api
    $integrations = $api.integrations
    
    # Extract from integrations
    $assets = $integrations.assets
    $assetTypes = $integrations.assetTypes
    $modalities = $integrations.modalities
    $psychology_data = $integrations.psychology
    
    # Remove from integrations (by creating a new object without them)
    $newIntegrations = New-Object PSObject
    foreach ($property in $integrations.psobject.Properties) {
        if ($property.Name -notin @('assets', 'assetTypes', 'modalities', 'psychology')) {
            $newIntegrations | Add-Member -MemberType NoteProperty -Name $property.Name -Value $property.Value
        }
    }

    # 2. Extract from root or forms
    $accounts = $jsonContent.accounts
    $currencies = $jsonContent.currencies
    $fees = $jsonContent.fees
    $risk = $jsonContent.risk
    
    $formsRoot = $jsonContent.forms
    $formsSettings = $settings.forms
    
    # Consolidate forms
    $forms = New-Object PSObject
    # Add from settings forms first
    if ($null -ne $formsSettings) {
        foreach ($p in $formsSettings.psobject.Properties) {
            $forms | Add-Member -MemberType NoteProperty -Name $p.Name -Value $p.Value -ErrorAction SilentlyContinue
        }
    }
    # Add from root forms
    if ($null -ne $formsRoot) {
        foreach ($p in $formsRoot.psobject.Properties) {
            $forms | Add-Member -MemberType NoteProperty -Name $p.Name -Value $p.Value -ErrorAction SilentlyContinue
        }
    }
    
    # Handle risk if it was inside forms
    if ($null -eq $risk) {
        $risk = $forms.risk
        $newForms = New-Object PSObject
        foreach ($p in $forms.psobject.Properties) {
            if ($p.Name -ne 'risk') {
                $newForms | Add-Member -MemberType NoteProperty -Name $p.Name -Value $p.Value
            }
        }
        $forms = $newForms
    }

    # 3. Build new settings object
    $newSettings = [Ordered]@{
        nav = $settings.nav
        general = $settings.general
        profile = $settings.profile
        api = [Ordered]@{
            form = $api.form
            integrations = $newIntegrations
        }
        assets = $assets
        assetTypes = $assetTypes
        accounts = $accounts
        currencies = $currencies
        fees = $fees
        risk = $risk
        modalities = $modalities
        forms = $forms
    }

    # 4. Assemble final data
    $finalData = [Ordered]@{
        general = $jsonContent.general
        filters = $jsonContent.filters
        strategy = $jsonContent.strategy
        sidebar = $jsonContent.sidebar
        auth = $jsonContent.auth
        home = $jsonContent.home
        journal = $jsonContent.journal
        trades = $jsonContent.trades
        finance = $jsonContent.finance
        psychology = $psychology_data
        settings = $newSettings
    }

    $finalData | ConvertTo-Json -Depth 10 | Set-Content $filePath -Encoding utf8
    Write-Host "Restructured $filePath"
}

Restructure-Json "c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\pt-BR.json"
Restructure-Json "c:\PROJETOS\TraderLogPro\src\lib\i18n\locales\en-US.json"
