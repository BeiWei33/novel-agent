param(
    [string]$Idea = "urban rebirth business story: a courier station owner returns ten years earlier",
    [ValidateSet("general", "qidian", "fanqie")]
    [string]$Platform = "fanqie",
    [ValidateSet("smoke", "openai", "deepseek")]
    [string]$Provider = "smoke",
    [string]$Model = "",
    [uint32]$Chapters = 3,
    [uint32]$OutlineBatchSize = 2,
    [uint32]$RunsLimit = 80,
    [uint32]$Port = 0,
    [switch]$UseRealModel
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

function Get-FreePort {
    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), 0)
    $listener.Start()
    try {
        return $listener.LocalEndpoint.Port
    } finally {
        $listener.Stop()
    }
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
        $result = Invoke-RestMethod -Method $Method -Uri $uri
    } else {
        $json = $Body | ConvertTo-Json -Depth 20
        $result = Invoke-RestMethod -Method $Method -Uri $uri -Body $json -ContentType "application/json"
    }
    $result | ConvertTo-Json -Depth 12
    return $result
}

function Invoke-ApiText {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Path
    )

    Write-Host "=== $Name ==="
    $response = Invoke-WebRequest -UseBasicParsing -Method $Method -Uri "$BaseUrl$Path"
    Write-Host "status=$($response.StatusCode) chars=$($response.Content.Length)"
    return $response
}

function Wait-ApiJob {
    param(
        [string]$JobId,
        [int]$MaxAttempts = 40
    )

    for ($attempt = 1; $attempt -le $MaxAttempts; $attempt++) {
        $result = Invoke-RestMethod -Method Get -Uri "$BaseUrl/api/jobs/$JobId"
        $status = $result.job.status
        if ($status -eq "succeeded") {
            Write-Host "job_id=$JobId status=succeeded attempts=$attempt"
            return $result
        }
        if ($status -eq "failed") {
            throw "Job $JobId failed: $($result.job.error)"
        }
        if ($status -eq "cancelled") {
            throw "Job $JobId was cancelled: $($result.job.error)"
        }

        Start-Sleep -Milliseconds 250
    }

    throw "Job $JobId did not finish after $MaxAttempts attempts."
}

function Quote-ProcessArgument {
    param([string]$Value)

    if ($Value -notmatch '[\s"]') {
        return $Value
    }

    return '"' + ($Value -replace '\\', '\\' -replace '"', '\"') + '"'
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

$WorkDir = Join-Path $env:TEMP ("novel-agent-api-demo-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $WorkDir | Out-Null

$ConfigPath = Join-Path $WorkDir "novel-agent.toml"
$DatabasePath = (Join-Path $WorkDir "novel-agent.db").Replace("\", "/")
$StdoutPath = Join-Path $WorkDir "serve.out.log"
$StderrPath = Join-Path $WorkDir "serve.err.log"
if ($Port -eq 0) {
    $Port = Get-FreePort
}
$Bind = "127.0.0.1:$Port"
$BaseUrl = "http://$Bind"

@"
[model]
provider = "$Provider"
model = "$Model"

[storage]
database_url = "sqlite://$DatabasePath"
"@ | Set-Content -Path $ConfigPath -Encoding UTF8

$server = $null
try {
    Write-Host "Starting API server at $BaseUrl"
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Cargo
    $serveArgs = @("run", "--quiet", "--", "--config", $ConfigPath, "serve", "--bind", $Bind)
    $startInfo.Arguments = ($serveArgs | ForEach-Object { Quote-ProcessArgument $_ }) -join " "
    $startInfo.WorkingDirectory = $ProjectRoot
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $server = [System.Diagnostics.Process]::new()
    $server.StartInfo = $startInfo
    $server.Start() | Out-Null

    $ready = $false
    for ($attempt = 1; $attempt -le 60; $attempt++) {
        if ($server.HasExited) {
            $stdout = $server.StandardOutput.ReadToEnd()
            $stderr = $server.StandardError.ReadToEnd()
            Set-Content -Path $StdoutPath -Value $stdout -Encoding UTF8
            Set-Content -Path $StderrPath -Value $stderr -Encoding UTF8
            throw "API server exited early with code $($server.ExitCode). stdout=$stdout stderr=$stderr"
        }

        try {
            Invoke-RestMethod -Method Get -Uri "$BaseUrl/health" | Out-Null
            $ready = $true
            break
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    if (-not $ready) {
        throw "API server did not become ready at $BaseUrl."
    }

    Write-Host "=== cors preflight ==="
    $cors = Invoke-WebRequest `
        -UseBasicParsing `
        -Method Options `
        -Uri "$BaseUrl/api/novels" `
        -Headers @{
            Origin = "http://localhost:5173"
            "Access-Control-Request-Method" = "GET"
        }
    if (-not $cors.Headers["Access-Control-Allow-Origin"]) {
        throw "CORS preflight did not return Access-Control-Allow-Origin."
    }
    Write-Host "cors=ok"

    $created = Invoke-ApiJson "create novel" "Post" "/api/novels" @{
        idea = $Idea
        platform = $Platform
        chapters = $Chapters
        outline_batch_size = $OutlineBatchSize
    }
    $NovelId = $created.novel.id
    if ([string]::IsNullOrWhiteSpace($NovelId)) {
        throw "Could not parse novel id from create response."
    }

    Invoke-ApiJson "list novels" "Get" "/api/novels?limit=10" | Out-Null
    $facts = Invoke-ApiJson "list facts" "Get" "/api/novels/$NovelId/facts?limit=10"
    if ($facts.facts.Count -lt 1) {
        throw "Facts API returned no facts."
    }
    $wrongFact = $facts.facts | Where-Object { $_.novel_id -ne $NovelId } | Select-Object -First 1
    if ($null -ne $wrongFact) {
        throw "Facts API returned fact for another novel."
    }
    Invoke-ApiJson "write chapter" "Post" "/api/novels/$NovelId/chapters/1/write" | Out-Null
    $continuity = Invoke-ApiJson "get continuity report" "Get" "/api/novels/$NovelId/chapters/1/continuity"
    if ($continuity.chapter.chapter_index -ne 1) {
        throw "Continuity API returned wrong chapter index."
    }
    if ($null -eq $continuity.report -or $null -eq $continuity.report.new_facts) {
        throw "Continuity API did not return latest report facts."
    }
    Invoke-ApiJson "review chapter" "Post" "/api/novels/$NovelId/chapters/1/review" | Out-Null

    $sse = Invoke-ApiText "write stream" "Post" "/api/novels/$NovelId/chapters/2/write/stream"
    if ($sse.Content -notmatch "event: completed") {
        throw "SSE response did not include completed event."
    }

    $writeJob = Invoke-ApiJson "write chapter job" "Post" "/api/novels/$NovelId/chapters/3/write/jobs"
    $JobId = $writeJob.job.id
    if ([string]::IsNullOrWhiteSpace($JobId)) {
        throw "Could not parse job id from write job response."
    }
    if ($writeJob.job.status -ne "queued") {
        throw "Expected write job to start as queued, got status=$($writeJob.job.status)."
    }
    if ($writeJob.job.payload.chapter_index -ne 3) {
        throw "Write job payload returned wrong chapter_index=$($writeJob.job.payload.chapter_index)."
    }
    if ($writeJob.job.progress_current -ne 0 -or $writeJob.job.progress_total -ne 1) {
        throw "Write job progress should start at 0/1."
    }
    if ($null -ne $writeJob.job.source_job_id) {
        throw "New write job should not have source_job_id."
    }

    $completedJob = Wait-ApiJob $JobId
    if ($completedJob.job.result.draft.chapter_index -ne 3) {
        throw "Completed write job returned wrong chapter_index=$($completedJob.job.result.draft.chapter_index)."
    }
    if ($completedJob.job.payload.chapter_index -ne 3) {
        throw "Completed write job payload returned wrong chapter_index=$($completedJob.job.payload.chapter_index)."
    }
    if ($null -ne $completedJob.job.source_job_id) {
        throw "Completed write job should not have source_job_id."
    }
    if ($completedJob.job.progress_current -ne 1 -or $completedJob.job.progress_total -ne 1) {
        throw "Completed write job progress should be 1/1."
    }

    $batchWriteJob = Invoke-ApiJson "write chapter batch job" "Post" "/api/novels/$NovelId/chapters/write/jobs" @{
        chapter_start = 4
        chapter_end = 5
    }
    $BatchJobId = $batchWriteJob.job.id
    if ([string]::IsNullOrWhiteSpace($BatchJobId)) {
        throw "Could not parse job id from batch write job response."
    }
    if ($batchWriteJob.job.kind -ne "write_chapters") {
        throw "Expected batch write job kind write_chapters, got kind=$($batchWriteJob.job.kind)."
    }
    if ($batchWriteJob.job.payload.chapter_start -ne 4 -or $batchWriteJob.job.payload.chapter_end -ne 5) {
        throw "Batch write job payload returned wrong range."
    }
    if ($batchWriteJob.job.payload.chapter_indexes.Count -ne 2) {
        throw "Batch write job payload did not return two chapter indexes."
    }
    if ($batchWriteJob.job.progress_current -ne 0 -or $batchWriteJob.job.progress_total -ne 2) {
        throw "Batch write job progress should start at 0/2."
    }
    if ($null -ne $batchWriteJob.job.source_job_id) {
        throw "New batch write job should not have source_job_id."
    }

    $completedBatchJob = Wait-ApiJob $BatchJobId
    if ($completedBatchJob.job.result.drafts.Count -ne 2) {
        throw "Completed batch write job should return two drafts."
    }
    if ($completedBatchJob.job.result.drafts[0].chapter_index -ne 4 -or $completedBatchJob.job.result.drafts[1].chapter_index -ne 5) {
        throw "Completed batch write job returned wrong chapter indexes."
    }
    if ($completedBatchJob.job.progress_current -ne 2 -or $completedBatchJob.job.progress_total -ne 2) {
        throw "Completed batch write job progress should be 2/2."
    }

    Write-Host "=== retry completed job rejected ==="
    $retryRejected = $false
    try {
        Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/jobs/$JobId/retry" | Out-Null
    } catch {
        $statusCode = [int]$_.Exception.Response.StatusCode
        if ($statusCode -eq 400) {
            $retryRejected = $true
        } else {
            throw
        }
    }
    if (-not $retryRejected) {
        throw "Retrying a completed job should have returned 400."
    }
    Write-Host "retry_completed_job=400"

    Write-Host "=== cancel completed job rejected ==="
    $cancelRejected = $false
    try {
        Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/jobs/$JobId/cancel" | Out-Null
    } catch {
        $statusCode = [int]$_.Exception.Response.StatusCode
        if ($statusCode -eq 400) {
            $cancelRejected = $true
        } else {
            throw
        }
    }
    if (-not $cancelRejected) {
        throw "Cancelling a completed job should have returned 400."
    }
    Write-Host "cancel_completed_job=400"

    $jobs = Invoke-ApiJson "list jobs" "Get" "/api/jobs?limit=10"
    $listedJob = $jobs.jobs | Where-Object { $_.id -eq $JobId } | Select-Object -First 1
    if ($null -eq $listedJob) {
        throw "Created job $JobId was not returned by /api/jobs."
    }
    $novelJobs = Invoke-ApiJson "list novel jobs" "Get" "/api/jobs?limit=10&novel_id=$NovelId"
    $listedNovelJob = $novelJobs.jobs | Where-Object { $_.id -eq $JobId } | Select-Object -First 1
    if ($null -eq $listedNovelJob) {
        throw "Created job $JobId was not returned by novel_id filtered /api/jobs."
    }
    $wrongNovelJob = $novelJobs.jobs | Where-Object { $_.novel_id -ne $NovelId } | Select-Object -First 1
    if ($null -ne $wrongNovelJob) {
        throw "novel_id filtered /api/jobs returned job for another novel."
    }
    $filteredJobs = Invoke-ApiJson "list succeeded batch jobs" "Get" "/api/jobs?limit=10&status=succeeded&kind=write_chapters&novel_id=$NovelId"
    $listedBatchJob = $filteredJobs.jobs | Where-Object { $_.id -eq $BatchJobId } | Select-Object -First 1
    if ($null -eq $listedBatchJob) {
        throw "Batch job $BatchJobId was not returned by filtered /api/jobs."
    }
    $wrongFilteredJob = $filteredJobs.jobs | Where-Object { $_.status -ne "succeeded" -or $_.kind -ne "write_chapters" } | Select-Object -First 1
    if ($null -ne $wrongFilteredJob) {
        throw "Filtered /api/jobs returned unexpected job kind/status."
    }

    $export = Invoke-ApiJson "export markdown" "Get" "/api/novels/$NovelId/export/markdown"
    if ([string]::IsNullOrWhiteSpace($export.markdown)) {
        throw "Export markdown response was empty."
    }

    $runs = Invoke-ApiJson "agent runs" "Get" "/api/novels/$NovelId/runs?limit=$RunsLimit"
    if ($runs.summary.fallback -gt 0 -or $runs.summary.parse_error -gt 0) {
        throw "API demo observed failing AgentRun status."
    }
    $writerRuns = Invoke-ApiJson "filtered agent runs" "Get" "/api/novels/$NovelId/runs?limit=20&role=writer&task=generate_chapter&status=ok"
    if ($writerRuns.runs.Count -lt 1) {
        throw "Filtered AgentRun API returned no writer generate_chapter runs."
    }
    $wrongWriterRun = $writerRuns.runs | Where-Object { $_.role -ne "writer" -or $_.task -ne "generate_chapter" -or $_.status -ne "ok" } | Select-Object -First 1
    if ($null -ne $wrongWriterRun) {
        throw "Filtered AgentRun API returned unexpected role/task/status."
    }

    Write-Host "=== result ==="
    Write-Host "novel_id=$NovelId"
    Write-Host "base_url=$BaseUrl"
    Write-Host "work_dir=$WorkDir"
    Write-Host "job_id=$JobId"
    Write-Host "batch_job_id=$BatchJobId"
    Write-Host "markdown_chars=$($export.markdown.Length)"
    Write-Host "agent_run_total=$($runs.summary.total)"
    Write-Host "agent_run_total_tokens=$($runs.summary.total_tokens)"
} finally {
    if ($null -ne $server -and -not $server.HasExited) {
        $server.Kill()
        $server.WaitForExit(5000) | Out-Null
    }
}
