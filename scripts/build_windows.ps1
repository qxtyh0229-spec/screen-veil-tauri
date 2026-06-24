# Screen Veil Tauri - Windows 构建脚本
# 前置条件: 已装 Rust + Visual Studio Build Tools (C++)

$ErrorActionPreference = "Stop"
$ScriptDir = $PSScriptRoot
$ProjectDir = Split-Path $ScriptDir -Parent

Set-Location $ProjectDir

# 1. 检查 Rust
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustc) {
    Write-Host "[X] 错误: 未找到 rustc, 请先安装 Rust: winget install Rustlang.Rustup" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Rust: $(rustc --version)" -ForegroundColor Green

# 2. 检查 VS Build Tools
$cl = Get-Command cl.exe -ErrorAction SilentlyContinue
if (-not $cl) {
    Write-Host "[!] 警告: 未找到 cl.exe, 可能没有 C++ 编译器" -ForegroundColor Yellow
    Write-Host "    装法: winget install Microsoft.VisualStudio.2022.BuildTools --override `"--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`"" -ForegroundColor Yellow
}

# 3. 安装 npm 依赖
Write-Host ""
Write-Host "[1/3] 安装 npm 依赖..." -ForegroundColor Blue
npm install 2>&1 | Select-Object -Last 5

# 4. 构建前端
Write-Host ""
Write-Host "[2/3] 构建前端..." -ForegroundColor Blue
npm run build 2>&1 | Select-Object -Last 5

# 5. 构建 Tauri 应用
Write-Host ""
Write-Host "[3/3] 构建 Tauri 应用 (首次约 5-10 分钟, 编译 Rust 依赖)..." -ForegroundColor Blue
npm run tauri build 2>&1 | Select-Object -Last 20

# 6. 验证产物
Write-Host ""
Write-Host "=== 构建完成 ===" -ForegroundColor Green
$exe = Join-Path $ProjectDir "src-tauri\target\release\screen-veil.exe"
if (Test-Path $exe) {
    $size = (Get-Item $exe).Length
    $sizeMB = [math]::Round($size / 1MB, 2)
    Write-Host "[OK] 产物: $exe ($sizeMB MB)" -ForegroundColor Green
} else {
    Write-Host "[!] 未找到 $exe" -ForegroundColor Yellow
    Get-ChildItem "$ProjectDir\src-tauri\target\release\" -Filter "*.exe" -ErrorAction SilentlyContinue
}

$bundle = "$ProjectDir\src-tauri\target\release\bundle\msi"
if (Test-Path $bundle) {
    Get-ChildItem $bundle -Filter "*.msi" | ForEach-Object {
        Write-Host "[OK] 安装包: $($_.FullName)" -ForegroundColor Green
    }
}
