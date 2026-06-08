param(
    [string]$Idea = "",
    [ValidateSet("general", "qidian", "fanqie")]
    [string]$Platform = "fanqie",
    [uint32]$Chapter = 1,
    [string]$Model = "gpt-5",
    [switch]$UseRealModel
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Idea)) {
    $Idea = [System.Text.Encoding]::UTF8.GetString(
        [System.Convert]::FromBase64String(
            "6YO95biC6YeN55Sf5ZWG5Lia5paH77yM5Li76KeS5Zue5Yiw5Y2B5bm05YmN77yM5LuO5aSW5Y2W56uZ5byA5aeL6YCG6KKt"
        )
    )
}

function Resolve-Cargo {
    if ($env:CARGO -and (Test-Path $env:CARGO)) {
        return $env:CARGO
    }

    $homeCargo = Join-Path $HOME ".cargo\bin\cargo.exe"
    if (Test-Path $homeCargo) {
        return $homeCargo
    }

    return "cargo"
}

function Invoke-DemoStep {
    param(
        [string]$Name,
        [string[]]$CliArgs
    )

    Write-Host "=== $Name ==="
    $output = & $Cargo run --quiet -- --config $ConfigPath @CliArgs 2>&1
    $text = $output -join "`n"
    Write-Host $text

    if ($LASTEXITCODE -ne 0) {
        throw "Step '$Name' failed with exit code $LASTEXITCODE."
    }

    return $text
}

$Cargo = Resolve-Cargo
$WorkDir = Join-Path $env:TEMP ("novel-agent-mvp-demo-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $WorkDir | Out-Null

$ConfigPath = Join-Path $WorkDir "novel-agent.toml"
$DatabasePath = (Join-Path $WorkDir "novel-agent.db").Replace("\", "/")
$ExportPath = Join-Path $WorkDir "export.md"

@"
[model]
provider = "openai"
model = "$Model"

[storage]
database_url = "sqlite://$DatabasePath"
"@ | Set-Content -Path $ConfigPath -Encoding UTF8

if (-not $UseRealModel) {
    Remove-Item Env:OPENAI_API_KEY -ErrorAction SilentlyContinue
    Write-Host "Running offline smoke fallback demo. Pass -UseRealModel to keep OPENAI_API_KEY."
}

$newOutput = Invoke-DemoStep "new" @("new", $Idea, "--platform", $Platform)
if ($newOutput -notmatch "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})") {
    throw "Could not parse novel id from new command output."
}

$NovelId = $Matches[1]
Invoke-DemoStep "outline" @("outline", "--novel-id", $NovelId, "--chapters", "30") | Out-Null
Invoke-DemoStep "write" @("write", "--novel-id", $NovelId, "--chapter", "$Chapter") | Out-Null
Invoke-DemoStep "review" @("review", "--novel-id", $NovelId, "--chapter", "$Chapter") | Out-Null
Invoke-DemoStep "rewrite" @("rewrite", "--novel-id", $NovelId, "--chapter", "$Chapter") | Out-Null
Invoke-DemoStep "export" @("export", "--novel-id", $NovelId, "--format", "markdown", "--output", $ExportPath) | Out-Null

if (-not (Test-Path $ExportPath)) {
    throw "Export file was not created."
}

$ExportSize = (Get-Item $ExportPath).Length
if ($ExportSize -le 0) {
    throw "Export file is empty."
}

Write-Host "=== result ==="
Write-Host "novel_id=$NovelId"
Write-Host "work_dir=$WorkDir"
Write-Host "export_path=$ExportPath"
Write-Host "export_size=$ExportSize"
