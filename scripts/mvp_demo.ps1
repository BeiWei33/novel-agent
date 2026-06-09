param(
    [string]$Idea = "",
    [ValidateSet("general", "qidian", "fanqie")]
    [string]$Platform = "fanqie",
    [uint32]$Chapter = 1,
    [ValidateSet("smoke", "openai", "deepseek")]
    [string]$Provider = "smoke",
    [string]$Model = "",
    [string]$ReasoningEffort = "",
    [uint32]$NewChapters = 30,
    [uint32]$NewOutlineBatchSize = 5,
    [uint32]$OutlineChapters = 30,
    [uint32]$OutlineBatchSize = 5,
    [uint32]$RunsLimit = 80,
    [uint32]$StepRetries = 1,
    [string]$WorkDir = "",
    [string]$ResumeNovelId = "",
    [switch]$SkipOutline,
    [switch]$SkipRewrite,
    [switch]$StreamWrite,
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

function Resolve-DefaultModel {
    param([string]$Provider)

    switch ($Provider) {
        "openai" { return "gpt-5" }
        "deepseek" { return "deepseek-chat" }
        default { return "smoke" }
    }
}

function Resolve-ProviderKeyName {
    param([string]$Provider)

    switch ($Provider) {
        "openai" { return "OPENAI_API_KEY" }
        "deepseek" { return "DEEPSEEK_API_KEY" }
        default { return "" }
    }
}

function Invoke-DemoStep {
    param(
        [string]$Name,
        [string[]]$CliArgs
    )

    for ($attempt = 1; $attempt -le ($script:EffectiveStepRetries + 1); $attempt++) {
        if ($script:EffectiveStepRetries -gt 0) {
            Write-Host "=== $Name (attempt $attempt) ==="
        } else {
            Write-Host "=== $Name ==="
        }

        $previousErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            $output = & $Cargo run --quiet -- --config $ConfigPath @CliArgs 2>&1
            $exitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
        $text = $output -join "`n"
        Write-Host $text

        if ($text -match "(?i)status=fallback|status=parse_error|fallback=[1-9][0-9]*|parse_error=[1-9][0-9]*|Agent 调用失败|smoke fallback|解析失败") {
            $script:ObservedFallback = $true
            Write-Warning "Step '$Name' reported fallback or parse failure output."
        }

        if ($exitCode -eq 0) {
            return $text
        }

        if ($attempt -le $script:EffectiveStepRetries) {
            Write-Warning "Step '$Name' failed with exit code $exitCode; retrying."
            Start-Sleep -Seconds 3
        } else {
            throw "Step '$Name' failed with exit code $exitCode."
        }
    }
}

function Resolve-LatestNovelIdFromRuns {
    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & $Cargo run --quiet -- --config $ConfigPath runs --limit "$RunsLimit" --summary 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    if ($exitCode -ne 0) {
        return ""
    }
    $text = $output -join "`n"
    $match = [regex]::Match($text, "novel=([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})")
    if ($match.Success) {
        return $match.Groups[1].Value
    }
    return ""
}

function Invoke-NewStep {
    $newArgs = @("new", $Idea, "--platform", $Platform, "--chapters", "$NewChapters", "--outline-batch-size", "$NewOutlineBatchSize")
    try {
        return Invoke-DemoStep "new" $newArgs
    } catch {
        if (-not $UseRealModel) {
            throw
        }
        $candidateNovelId = Resolve-LatestNovelIdFromRuns
        if ([string]::IsNullOrWhiteSpace($candidateNovelId)) {
            throw
        }
        Write-Warning "Step 'new' failed; attempting checkpoint resume for novel_id=$candidateNovelId."
        $resumeArgs = $newArgs + @("--resume-novel-id", $candidateNovelId)
        return Invoke-DemoStep "new resume" $resumeArgs
    }
}

$Cargo = Resolve-Cargo
$ObservedFallback = $false
if ([string]::IsNullOrWhiteSpace($Model)) {
    $Model = Resolve-DefaultModel $Provider
}
if ($NewChapters -lt 1) {
    throw "NewChapters must be at least 1."
}
if ($NewOutlineBatchSize -lt 1) {
    throw "NewOutlineBatchSize must be at least 1."
}
if ($OutlineChapters -lt 1) {
    throw "OutlineChapters must be at least 1."
}
if ($OutlineBatchSize -lt 1) {
    throw "OutlineBatchSize must be at least 1."
}
if ($RunsLimit -lt 1) {
    throw "RunsLimit must be at least 1."
}
if ($StepRetries -lt 0) {
    throw "StepRetries must be at least 0."
}
$script:EffectiveStepRetries = if ($UseRealModel) { $StepRetries } else { 0 }
if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $env:TEMP ("novel-agent-mvp-demo-" + [guid]::NewGuid().ToString("N"))
}
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null

$ConfigPath = Join-Path $WorkDir "novel-agent.toml"
$DatabasePath = (Join-Path $WorkDir "novel-agent.db").Replace("\", "/")
$ExportPath = Join-Path $WorkDir "export.md"
$ManualEditPath = Join-Path $WorkDir "manual-edit.txt"

$configLines = @(
    "[model]",
    "provider = `"$Provider`"",
    "model = `"$Model`""
)
if (-not [string]::IsNullOrWhiteSpace($ReasoningEffort)) {
    $configLines += "reasoning_effort = `"$ReasoningEffort`""
}
$configLines += @(
    "",
    "[storage]",
    "database_url = `"sqlite://$DatabasePath`""
)
$configLines | Set-Content -Path $ConfigPath -Encoding UTF8

if ($Provider -eq "smoke") {
    if ($UseRealModel) {
        throw "UseRealModel requires provider openai or deepseek, not smoke."
    }
    Write-Host "Running local smoke provider demo. Pass -Provider openai or -Provider deepseek with -UseRealModel to call a real provider."
} elseif (-not $UseRealModel) {
    Remove-Item Env:OPENAI_API_KEY -ErrorAction SilentlyContinue
    Remove-Item Env:DEEPSEEK_API_KEY -ErrorAction SilentlyContinue
    Write-Host "Running $Provider fallback demo without provider key. Pass -UseRealModel to keep the real provider key."
} else {
    $keyName = Resolve-ProviderKeyName $Provider
    $keyValue = [Environment]::GetEnvironmentVariable($keyName)
    if ([string]::IsNullOrWhiteSpace($keyValue)) {
        throw "UseRealModel with provider=$Provider requires $keyName to be set. Refusing to run a real-model validation that would silently fall back."
    }
    Write-Host "Running real model demo with provider=$Provider model=$Model."
}

if ([string]::IsNullOrWhiteSpace($ResumeNovelId)) {
    $newOutput = Invoke-NewStep
    if ($newOutput -notmatch "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})") {
        throw "Could not parse novel id from new command output."
    }
    $NovelId = $Matches[1]
} else {
    $NovelId = $ResumeNovelId
    Write-Host "=== new ==="
    Write-Host "Skipped new step; resuming novel_id=$NovelId."
}
if ($SkipOutline) {
    Write-Host "=== outline ==="
    Write-Host "Skipped outline step."
} else {
    Invoke-DemoStep "outline" @("outline", "--novel-id", $NovelId, "--chapters", "$OutlineChapters", "--batch-size", "$OutlineBatchSize") | Out-Null
}
$writeArgs = @("write", "--novel-id", $NovelId, "--chapter", "$Chapter")
if ($StreamWrite) {
    $writeArgs += "--stream"
}
Invoke-DemoStep "write" $writeArgs | Out-Null
Invoke-DemoStep "review" @("review", "--novel-id", $NovelId, "--chapter", "$Chapter") | Out-Null
if ($SkipRewrite) {
    Write-Host "=== rewrite ==="
    Write-Host "Skipped rewrite step."
} else {
    $rewriteArgs = @("rewrite", "--novel-id", $NovelId, "--chapter", "$Chapter")
    if ($StreamWrite) {
        $rewriteArgs += "--stream"
    }
    Invoke-DemoStep "rewrite" $rewriteArgs | Out-Null
    Invoke-DemoStep "versions" @("versions", "--novel-id", $NovelId, "--chapter", "$Chapter", "--from", "1", "--to", "2") | Out-Null
    $ManualEditContent = [System.Text.Encoding]::UTF8.GetString(
        [System.Convert]::FromBase64String(
            "5Lq65bel57yW6L6R54mI5pys77ya5p6X6Iif5Zyo6YeN5YaZ56i/5LmL5ZCO6KGl5LiK5LiA5q615Lq657G757yW6L6R56Gu6K6k55qE5LyP56yU44CCCuS7luaKiuS4i+S4gOatpeebruagh+aYjuehruWGmeaIkOWFiOS/neS9j+WkluWNluerme+8jOWGjei/veafpeaPkOWJjeWHuueOsOeahOS6uuOAgg=="
        )
    )
    $ManualEditSummary = [System.Text.Encoding]::UTF8.GetString(
        [System.Convert]::FromBase64String("5Lq65bel57yW6L6R5ZCO6KGl5by655uu5qCH5ZKM5LyP56yU")
    )
    [System.IO.File]::WriteAllText($ManualEditPath, $ManualEditContent, [System.Text.UTF8Encoding]::new($false))
    Invoke-DemoStep "edit" @("edit", "--novel-id", $NovelId, "--chapter", "$Chapter", "--input", $ManualEditPath, "--summary", $ManualEditSummary) | Out-Null
    Invoke-DemoStep "versions manual" @("versions", "--novel-id", $NovelId, "--chapter", "$Chapter", "--from", "2", "--to", "3") | Out-Null
}
Invoke-DemoStep "export" @("export", "--novel-id", $NovelId, "--format", "markdown", "--output", $ExportPath) | Out-Null
$runsArgs = @("runs", "--novel-id", $NovelId, "--limit", "$RunsLimit", "--summary")
if ($UseRealModel) {
    $runsArgs += "--fail-on-bad-status"
}
Invoke-DemoStep "runs" $runsArgs | Out-Null

if (-not (Test-Path $ExportPath)) {
    throw "Export file was not created."
}

$ExportSize = (Get-Item $ExportPath).Length
if ($ExportSize -le 0) {
    throw "Export file is empty."
}

if ($UseRealModel -and $ObservedFallback) {
    throw "Real model demo observed fallback or parse failure output. Inspect the command output and agent_runs before accepting this as a real-model pass."
}

Write-Host "=== result ==="
Write-Host "novel_id=$NovelId"
Write-Host "work_dir=$WorkDir"
Write-Host "export_path=$ExportPath"
Write-Host "export_size=$ExportSize"
Write-Host "runs_limit=$RunsLimit"
