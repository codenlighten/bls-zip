$files = @(
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\audit.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\asset.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\wallet.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\application.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\auth.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\events.rs',
    'C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\src\services\hardware.rs'
)

foreach ($file in $files) {
    $content = Get-Content $file -Raw
    $newContent = $content -replace 'DatabaseError\(e\.to_string\(\)\)', 'from_db_error(e)'
    Set-Content -Path $file -Value $newContent -NoNewline
    Write-Output "Fixed: $file"
}

Write-Output "Replacement complete in all 7 files"
