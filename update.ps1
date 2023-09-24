# タグ名を引数から取得
$tagName = $args[0]

# Extract version (vを取り除く)
$version = $tagName -replace '^v', ''

# Update package.version in tauri.conf.json
$tauriConfPath = "src-tauri\tauri.conf.json"
$tauriConf = Get-Content -Path $tauriConfPath | ConvertFrom-Json
$tauriConf.package.version = $version
$tauriConf | ConvertTo-Json -Depth 100 | Set-Content -Path $tauriConfPath

# Set Tauri Environment Variables
$env:TAURI_PRIVATE_KEY = Get-Content -Path "~\.tauri\launcherg-actions.key"
$env:TAURI_KEY_PASSWORD = Get-Content -Path "~\.tauri\launcherg-actions-pass.key"

# Install dependencies and build
Invoke-Expression "npm i"
Invoke-Expression "npm run tauri build"

# Update .tauri-updater.json
$updaterData = @{
    version = $version
    notes = "See the assets to download this version and install."
    pub_date = (Get-Date -Format s) + "Z"
    signature = Get-Content -Path ".\src-tauri\target\release\bundle\msi\Launcherg_${version}_x64_ja-JP.msi.zip.sig"
    url = "https://github.com/ryoha000/launcherg/releases/download/${tagName}/Launcherg_${version}_x64_ja-JP.msi.zip"
}
$updaterData | ConvertTo-Json | Set-Content -Path ".tauri-updater.json"

# Format files (Prettier のCLIを使ってファイルをフォーマット)
Invoke-Expression "npx -y prettier $tauriConfPath .tauri-updater.json --write"

# Push updated files to main
git add $tauriConfPath .tauri-updater.json
git commit -m "Update for release $version"
git push origin main

git tag $tagName
git push origin $tagName
