param(
    [string]$Idea = "Urban rebirth business story, protagonist returns ten years earlier and starts from a delivery station.",
    [ValidateSet("general", "qidian", "fanqie")]
    [string]$Platform = "fanqie",
    [uint32]$Chapters = 6,
    [uint32]$OutlineBatchSize = 3,
    [uint32]$Chapter = 1,
    [ValidateSet("smoke", "openai", "deepseek")]
    [string]$Provider = "smoke",
    [string]$Model = "",
    [string]$ReasoningEffort = "",
    [uint32]$RunsLimit = 100,
    [uint32]$Port = 0,
    [string]$WorkDir = "",
    [switch]$UseRealModel,
    [switch]$SkipWebBuild
)

$ErrorActionPreference = "Stop"

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
    param([string]$ProviderName)

    switch ($ProviderName) {
        "smoke" { return "smoke" }
        "deepseek" { return "deepseek-chat" }
        default { return "gpt-5" }
    }
}

function Resolve-ProviderKeyName {
    param([string]$ProviderName)

    switch ($ProviderName) {
        "deepseek" { return "DEEPSEEK_API_KEY" }
        default { return "OPENAI_API_KEY" }
    }
}

function Resolve-TargetDir {
    if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
        return (Join-Path $ProjectRoot "target")
    }
    if ([System.IO.Path]::IsPathRooted($env:CARGO_TARGET_DIR)) {
        return $env:CARGO_TARGET_DIR
    }
    return (Join-Path $ProjectRoot $env:CARGO_TARGET_DIR)
}

function Resolve-CliPath {
    Write-Host "=== build cli ==="
    & $Cargo build --quiet
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE."
    }

    $targetDir = Resolve-TargetDir
    $windowsExe = Join-Path $targetDir "debug\novel-agent.exe"
    if (Test-Path $windowsExe) {
        return $windowsExe
    }

    $unixExe = Join-Path $targetDir "debug\novel-agent"
    if (Test-Path $unixExe) {
        return $unixExe
    }

    throw "Could not find built novel-agent binary under $targetDir."
}

function Get-FreePort {
    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), 0)
    $listener.Start()
    try {
        return $listener.LocalEndpoint.Port
    } finally {
        $listener.Stop()
    }
}

function Quote-ProcessArgument {
    param([string]$Value)

    if ($Value -notmatch '[\s"]') {
        return $Value
    }

    return '"' + ($Value -replace '\\', '\\' -replace '"', '\"') + '"'
}

function Invoke-ApiJson {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Path,
        [object]$Body = $null
    )

    Write-Host "=== $Name ==="
    $uri = "$BaseUrl$Path"
    if ($null -eq $Body) {
        return Invoke-RestMethod -Method $Method -Uri $uri
    }

    $json = $Body | ConvertTo-Json -Depth 50
    return Invoke-RestMethod -Method $Method -Uri $uri -ContentType "application/json" -Body $json
}

function Wait-ApiReady {
    for ($attempt = 1; $attempt -le 80; $attempt++) {
        if ($Server.HasExited) {
            throw "API server exited early with code $($Server.ExitCode)."
        }

        try {
            Invoke-RestMethod -Method Get -Uri "$BaseUrl/health" | Out-Null
            return
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }

    throw "API server did not become ready at $BaseUrl."
}

function Wait-JobTerminal {
    param([string]$JobId)

    for ($attempt = 1; $attempt -le 120; $attempt++) {
        $payload = Invoke-ApiJson "poll job $JobId" "Get" "/api/jobs/$JobId"
        $job = $payload.job
        if ($job.status -in @("succeeded", "failed", "cancelled")) {
            return $job
        }
        Start-Sleep -Milliseconds 500
    }

    throw "Job $JobId did not reach a terminal state."
}

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Run-WebBuild {
    if ($SkipWebBuild) {
        Write-Host "=== web build ==="
        Write-Host "Skipped web build."
        return
    }

    $webRoot = Join-Path $ProjectRoot "apps\web"
    if (-not (Test-Path $webRoot)) {
        throw "Web workspace was not found at $webRoot."
    }

    Write-Host "=== web build ==="
    Push-Location $webRoot
    try {
        & npm.cmd run build
        if ($LASTEXITCODE -ne 0) {
            throw "npm.cmd run build failed with exit code $LASTEXITCODE."
        }
    } finally {
        Pop-Location
    }
}

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Cargo = Resolve-Cargo
if ([string]::IsNullOrWhiteSpace($Model)) {
    $Model = Resolve-DefaultModel $Provider
}
if ($Chapters -lt 1) {
    throw "Chapters must be at least 1."
}
if ($OutlineBatchSize -lt 1) {
    throw "OutlineBatchSize must be at least 1."
}
if ($Chapter -lt 1 -or $Chapter -gt $Chapters) {
    throw "Chapter must be within 1..Chapters."
}
if ($RunsLimit -lt 1) {
    throw "RunsLimit must be at least 1."
}
if ($Provider -eq "smoke" -and $UseRealModel) {
    throw "UseRealModel requires provider openai or deepseek, not smoke."
}
if ($Provider -ne "smoke" -and $UseRealModel) {
    $keyName = Resolve-ProviderKeyName $Provider
    if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($keyName))) {
        throw "UseRealModel with provider=$Provider requires $keyName to be set."
    }
}
if ($Provider -ne "smoke" -and -not $UseRealModel) {
    Remove-Item Env:OPENAI_API_KEY -ErrorAction SilentlyContinue
    Remove-Item Env:DEEPSEEK_API_KEY -ErrorAction SilentlyContinue
}

$CliPath = Resolve-CliPath
if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $env:TEMP ("novel-agent-v03-e2e-" + [guid]::NewGuid().ToString("N"))
}
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null

$ConfigPath = Join-Path $WorkDir "novel-agent.toml"
$DatabasePath = (Join-Path $WorkDir "novel-agent.db").Replace("\", "/")
$StdoutPath = Join-Path $WorkDir "serve.out.log"
$StderrPath = Join-Path $WorkDir "serve.err.log"
if ($Port -eq 0) {
    $Port = Get-FreePort
}
$Bind = "127.0.0.1:$Port"
$BaseUrl = "http://$Bind"

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
[System.IO.File]::WriteAllText($ConfigPath, ($configLines -join [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false))

$Server = $null
try {
    Run-WebBuild

    Write-Host "=== start api ==="
    Write-Host "base_url=$BaseUrl"
    Write-Host "work_dir=$WorkDir"
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $CliPath
    $serveArgs = @("--config", $ConfigPath, "serve", "--bind", $Bind)
    $startInfo.Arguments = ($serveArgs | ForEach-Object { Quote-ProcessArgument $_ }) -join " "
    $startInfo.WorkingDirectory = $ProjectRoot
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $Server = [System.Diagnostics.Process]::new()
    $Server.StartInfo = $startInfo
    $Server.Start() | Out-Null
    Wait-ApiReady

    $health = Invoke-ApiJson "health" "Get" "/health"
    Assert-True ($health.status -eq "ok") "Health API did not return ok."

    $modelUpdate = @{
        provider = $Provider
        model = $Model
    }
    if (-not [string]::IsNullOrWhiteSpace($ReasoningEffort)) {
        $modelUpdate.reasoning_effort = $ReasoningEffort
    }
    if ($Provider -eq "smoke") {
        $modelUpdate.pricing = @{
            prompt_cost_micro_usd_per_million_tokens = 1000000
            completion_cost_micro_usd_per_million_tokens = 2000000
        }
    }
    $modelResponse = Invoke-ApiJson "model settings" "Put" "/api/model" $modelUpdate
    Assert-True ($modelResponse.model.provider -eq $Provider) "Model provider was not preserved."

    $create = Invoke-ApiJson "create novel" "Post" "/api/novels" @{
        idea = $Idea
        platform = $Platform
        chapters = $Chapters
        outline_batch_size = $OutlineBatchSize
    }
    $NovelId = $create.novel.id
    Assert-True (-not [string]::IsNullOrWhiteSpace($NovelId)) "Create novel did not return a novel id."

    $detail = Invoke-ApiJson "get novel detail" "Get" "/api/novels/$NovelId"
    Assert-True ($detail.novel.id -eq $NovelId) "Novel detail returned the wrong id."

    $write = Invoke-ApiJson "write chapter" "Post" "/api/novels/$NovelId/chapters/$Chapter/write"
    Assert-True ($write.draft.chapter_index -eq $Chapter) "Write chapter returned the wrong chapter."

    $review = Invoke-ApiJson "review chapter" "Post" "/api/novels/$NovelId/chapters/$Chapter/review"
    Assert-True ($null -ne $review.report.total_score) "Review did not return total_score."

    $rewrite = Invoke-ApiJson "rewrite chapter" "Post" "/api/novels/$NovelId/chapters/$Chapter/rewrite"
    Assert-True ($rewrite.draft.version -ge 2) "Rewrite did not create a new version."

    $manualContent = "Manual edit: preserve the generated conflict, clarify the next action, and mark this as the v0.3 e2e accepted draft."
    $manual = Invoke-ApiJson "manual edit" "Put" "/api/novels/$NovelId/chapters/$Chapter/content" @{
        title = "$($rewrite.draft.title) - e2e edit"
        content = $manualContent
        summary = "v0.3 e2e manual edit"
    }
    Assert-True ($manual.draft.version -ge 3) "Manual edit did not create version 3."

    $versions = Invoke-ApiJson "chapter versions" "Get" "/api/novels/$NovelId/chapters/$Chapter/versions"
    Assert-True ($versions.versions.Count -ge 3) "Expected at least three chapter versions."

    $export = Invoke-ApiJson "export markdown" "Post" "/api/novels/$NovelId/export"
    Assert-True ($export.markdown.Length -gt 0) "Export markdown was empty."

    $jobsCreated = @()
    if ($Chapters -ge 3) {
        $jobResponse = Invoke-ApiJson "create batch write job" "Post" "/api/novels/$NovelId/chapters/write/jobs" @{
            chapter_start = 2
            chapter_end = 3
        }
        $job = Wait-JobTerminal $jobResponse.job.id
        Assert-True ($job.status -eq "succeeded") "Batch write job did not succeed. status=$($job.status)"
        $jobsCreated += $job
    }

    $runsPath = "/api/runs?limit=$RunsLimit&novel_id=$NovelId"
    $runs = Invoke-ApiJson "agent runs" "Get" $runsPath
    Assert-True ($runs.summary.total -gt 0) "AgentRun summary was empty."
    Assert-True ($runs.summary.fallback -eq 0 -and $runs.summary.parse_error -eq 0) "AgentRun summary contained bad statuses."

    $jobs = Invoke-ApiJson "jobs summary" "Get" "/api/jobs?limit=100&novel_id=$NovelId"
    $jobSucceeded = @($jobs.jobs | Where-Object { $_.status -eq "succeeded" }).Count
    $jobFailed = @($jobs.jobs | Where-Object { $_.status -eq "failed" }).Count
    $jobCancelled = @($jobs.jobs | Where-Object { $_.status -eq "cancelled" }).Count

    Write-Host "=== result ==="
    Write-Host "novel_id=$NovelId"
    Write-Host "base_url=$BaseUrl"
    Write-Host "work_dir=$WorkDir"
    Write-Host "review_score=$($review.report.total_score)"
    Write-Host "export_chars=$($export.markdown.Length)"
    Write-Host "agent_run_total=$($runs.summary.total)"
    Write-Host "agent_run_bad=$($runs.summary.fallback + $runs.summary.parse_error)"
    Write-Host "agent_run_duration_ms=$($runs.summary.duration_ms_total)"
    Write-Host "agent_run_total_tokens=$($runs.summary.total_tokens)"
    Write-Host "agent_run_total_cost_micro_usd=$($runs.summary.total_cost_micro_usd)"
    Write-Host "jobs_succeeded=$jobSucceeded"
    Write-Host "jobs_failed=$jobFailed"
    Write-Host "jobs_cancelled=$jobCancelled"
    Write-Host "status=ok"
} finally {
    if ($null -ne $Server) {
        if (-not $Server.HasExited) {
            $Server.Kill()
            $Server.WaitForExit()
        }
        try {
            $stdout = $Server.StandardOutput.ReadToEnd()
            $stderr = $Server.StandardError.ReadToEnd()
            [System.IO.File]::WriteAllText($StdoutPath, $stdout, [System.Text.UTF8Encoding]::new($false))
            [System.IO.File]::WriteAllText($StderrPath, $stderr, [System.Text.UTF8Encoding]::new($false))
        } catch {
            Write-Warning "Could not capture server logs: $($_.Exception.Message)"
        }
    }
}
