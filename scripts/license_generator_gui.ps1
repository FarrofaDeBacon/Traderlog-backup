Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# --- CONFIG ---
$SECRET_KEY = "TRADERLOGPRO_SECRET_KEY_2026"
$VERSION = "v1"
$PREFIX = "TLP"

# --- COLORS ---
$BG_COLOR = "#09090b"
$CARD_COLOR = "#18181b"
$TEXT_COLOR = "#fafafa"
$PRIMARY_COLOR = "#3b82f6"

# --- FORM SETUP ---
$Form = New-Object Windows.Forms.Form
$Form.Text = "TraderLogPro | License Generator"
$Form.Size = New-Object Drawing.Size(500, 600)
$Form.BackColor = $BG_COLOR
$Form.StartPosition = "CenterScreen"
$Form.FormBorderStyle = "FixedDialog"
$Form.MaximizeBox = $false

$FontHeader = New-Object Drawing.Font("Segoe UI", 16, [Drawing.FontStyle]::Bold)
$FontBold = New-Object Drawing.Font("Segoe UI", 10, [Drawing.FontStyle]::Bold)
$FontMono = New-Object Drawing.Font("Consolas", 9)

# Header
$Header = New-Object Windows.Forms.Label
$Header.Text = "TraderLogPro"
$Header.Font = $FontHeader
$Header.ForeColor = [Drawing.ColorTranslator]::FromHtml($PRIMARY_COLOR)
$Header.Size = New-Object Drawing.Size(480, 40)
$Header.Location = New-Object Drawing.Point(10, 20)
$Header.TextAlign = "MiddleCenter"
$Form.Controls.Add($Header)

$SubHeader = New-Object Windows.Forms.Label
$SubHeader.Text = "GERADOR DE LICENCAS NATIVO - v1.1 (FIX)"
$SubHeader.Font = New-Object Drawing.Font("Segoe UI", 8)
$SubHeader.ForeColor = [Drawing.ColorTranslator]::FromHtml("#a1a1aa")
$SubHeader.Size = New-Object Drawing.Size(480, 20)
$SubHeader.Location = New-Object Drawing.Point(10, 60)
$SubHeader.TextAlign = "MiddleCenter"
$Form.Controls.Add($SubHeader)

# Card Container
$Card = New-Object Windows.Forms.Panel
$Card.Location = New-Object Drawing.Point(30, 100)
$Card.Size = New-Object Drawing.Size(425, 420)
$Card.BackColor = [Drawing.ColorTranslator]::FromHtml($CARD_COLOR)
$Card.BorderStyle = "FixedSingle"
$Form.Controls.Add($Card)

# Plan Selection
$LblPlan = New-Object Windows.Forms.Label
$LblPlan.Text = "PLANO DE ACESSO"
$LblPlan.Location = New-Object Drawing.Point(20, 20)
$LblPlan.Size = New-Object Drawing.Size(200, 20)
$LblPlan.ForeColor = [Drawing.ColorTranslator]::FromHtml($TEXT_COLOR)
$Card.Controls.Add($LblPlan)

$ComboPlan = New-Object Windows.Forms.ComboBox
$ComboPlan.Items.AddRange(@("Pro", "Trial", "Enterprise", "Lifetime"))
$ComboPlan.SelectedIndex = 0
$ComboPlan.Location = New-Object Drawing.Point(20, 45)
$ComboPlan.Size = New-Object Drawing.Size(380, 30)
$ComboPlan.DropDownStyle = "DropDownList"
$Card.Controls.Add($ComboPlan)

# Customer ID
$LblCid = New-Object Windows.Forms.Label
$LblCid.Text = "ID DO CLIENTE (OPCIONAL)"
$LblCid.Location = New-Object Drawing.Point(20, 80)
$LblCid.Size = New-Object Drawing.Size(200, 20)
$LblCid.ForeColor = [Drawing.ColorTranslator]::FromHtml($TEXT_COLOR)
$Card.Controls.Add($LblCid)

$TxtCid = New-Object Windows.Forms.TextBox
$TxtCid.Location = New-Object Drawing.Point(20, 105)
$TxtCid.Size = New-Object Drawing.Size(380, 30)
$Card.Controls.Add($TxtCid)

# Days Selection
$LblDays = New-Object Windows.Forms.Label
$LblDays.Text = "DURACAO (DIAS)"
$LblDays.Location = New-Object Drawing.Point(20, 140)
$LblDays.Size = New-Object Drawing.Size(200, 20)
$LblDays.ForeColor = [Drawing.ColorTranslator]::FromHtml($TEXT_COLOR)
$Card.Controls.Add($LblDays)

$TxtDays = New-Object Windows.Forms.TextBox
$TxtDays.Text = "7"
$TxtDays.Location = New-Object Drawing.Point(20, 165)
$TxtDays.Size = New-Object Drawing.Size(380, 30)
$Card.Controls.Add($TxtDays)

# Lifetime Toggle
$ChkLifetime = New-Object Windows.Forms.CheckBox
$ChkLifetime.Text = "Ativar Licenca Vitalicia"
$ChkLifetime.Location = New-Object Drawing.Point(20, 200)
$ChkLifetime.Size = New-Object Drawing.Size(300, 30)
$ChkLifetime.ForeColor = [Drawing.ColorTranslator]::FromHtml($TEXT_COLOR)
$ChkLifetime.Add_CheckedChanged({
        if ($ChkLifetime.Checked) {
            $TxtDays.Enabled = $false
            $ComboPlan.SelectedItem = "Lifetime"
        }
        else {
            $TxtDays.Enabled = $true
            $ComboPlan.SelectedItem = "Pro"
        }
    })
$Card.Controls.Add($ChkLifetime)

# Generate Button
$BtnGen = New-Object Windows.Forms.Button
$BtnGen.Text = "GERAR CHAVE LINKADA"
$BtnGen.Location = New-Object Drawing.Point(20, 240)
$BtnGen.Size = New-Object Drawing.Size(380, 45)
$BtnGen.BackColor = [Drawing.ColorTranslator]::FromHtml($PRIMARY_COLOR)
$BtnGen.ForeColor = [Drawing.Color]::White
$BtnGen.FlatStyle = "Flat"
$BtnGen.Font = $FontBold
$BtnGen.Add_Click({
        try {
            $IsLife = $ChkLifetime.Checked -or ($ComboPlan.Text -eq "Lifetime")
            $Plan = if ($IsLife) { "Lifetime" } else { $ComboPlan.Text }
            $Cid = $TxtCid.Text.Trim().ToUpper()
            $ExpIso = $null
        
            if (-not $IsLife) {
                [int]$Days = 0
                if ([int]::TryParse($TxtDays.Text, [ref]$Days)) {
                    $ExpDate = [DateTime]::UtcNow.AddDays($Days)
                    $ExpIso = $ExpDate.ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
                }
            }
        
            $CreatedAt = [DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
        
            $Payload = @{
                plan       = $Plan
                exp        = $ExpIso
                cid        = $Cid # Added Customer ID
                created_at = $CreatedAt
            }
        
            $PayloadJson = $Payload | ConvertTo-Json -Compress
            $PayloadB64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($PayloadJson))
        
            $DataToSign = "$PREFIX-$VERSION-$PayloadB64"
        
            # HMAC SHA 256
            $hmacsha = New-Object System.Security.Cryptography.HMACSHA256
            $hmacsha.Key = [System.Text.Encoding]::UTF8.GetBytes($SECRET_KEY)
            $signature = $hmacsha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($DataToSign))
            $signatureHex = [System.BitConverter]::ToString($signature).Replace("-", "").ToLower()
        
            $FinalKey = "$DataToSign-$signatureHex"
        
            $TxtResult.Text = $FinalKey
            $BtnExport.Enabled = $true
        }
        catch {
            [System.Windows.Forms.MessageBox]::Show("Erro ao gerar chave: $($_.Exception.Message)")
        }
    })
$Card.Controls.Add($BtnGen)

# Result area
$TxtResult = New-Object Windows.Forms.TextBox
$TxtResult.Multiline = $true
$TxtResult.Location = New-Object Drawing.Point(20, 295)
$TxtResult.Size = New-Object Drawing.Size(380, 50)
$TxtResult.Font = $FontMono
$TxtResult.ForeColor = [Drawing.ColorTranslator]::FromHtml("#4ade80")
$TxtResult.BackColor = [Drawing.ColorTranslator]::FromHtml($BG_COLOR)
$TxtResult.ReadOnly = $true
$Card.Controls.Add($TxtResult)

# Export Button
$BtnExport = New-Object Windows.Forms.Button
$BtnExport.Text = "EXPORTAR ARQUIVO .LIC"
$BtnExport.Location = New-Object Drawing.Point(20, 355)
$BtnExport.Size = New-Object Drawing.Size(380, 40)
$BtnExport.BackColor = [Drawing.ColorTranslator]::FromHtml("#10b981") # Emerald 500
$BtnExport.ForeColor = [Drawing.Color]::White
$BtnExport.FlatStyle = "Flat"
$BtnExport.Font = $FontBold
$BtnExport.Enabled = $false
$BtnExport.Add_Click({
        if ($TxtResult.Text -ne "") {
            $SaveDialog = New-Object Windows.Forms.SaveFileDialog
            $SaveDialog.Filter = "License Files (*.lic)|*.lic|All files (*.*)|*.*"
            $SaveDialog.FileName = "license.lic"
            if ($SaveDialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                $UTF8NoBOM = New-Object System.Text.UTF8Encoding $false
                [System.IO.File]::WriteAllText($SaveDialog.FileName, $TxtResult.Text, $UTF8NoBOM)
                [System.Windows.Forms.MessageBox]::Show("Licença exportada para: $($SaveDialog.FileName)")
            }
        }
    })
$Card.Controls.Add($BtnExport)

# Footer
$Footer = New-Object Windows.Forms.Label
$Footer.Text = "Clique com o botão direito para copiar no Windows"
$Footer.ForeColor = [Drawing.ColorTranslator]::FromHtml("#a1a1aa")
$Footer.Font = New-Object Drawing.Font("Segoe UI", 8)
$Footer.Size = New-Object Drawing.Size(480, 20)
$Footer.Location = New-Object Drawing.Point(10, 530)
$Footer.TextAlign = "MiddleCenter"
$Form.Controls.Add($Footer)

[void]$Form.ShowDialog()
