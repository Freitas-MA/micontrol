param(
  [Parameter(Mandatory=$true)][string]$ScenarioName,
  [Parameter(Mandatory=$true)][string]$TraceVal
)
$ErrorActionPreference = 'Stop'
$benchDir = Join-Path (Get-Location) '.bench'
New-Item -ItemType Directory -Path $benchDir -Force | Out-Null

function Stop-Conflicts {
  $proj = (Get-Location).Path
  $targets = Get-CimInstance Win32_Process | Where-Object {
    $_.Name -in @('micontrol.exe','cargo.exe','rustc.exe') -or
    ($_.Name -eq 'node.exe' -and $_.CommandLine -like "*$proj*") -or
    ($_.Name -in @('cmd.exe','conhost.exe','powershell.exe','pwsh.exe') -and $_.CommandLine -like '*npm run tauri dev*' -and $_.CommandLine -like "*$proj*")
  }
  $pids = @($targets.ProcessId | Sort-Object -Unique)
  if ($pids.Count -gt 0) {
    foreach ($procId in $pids) { try { Stop-Process -Id $procId -ErrorAction SilentlyContinue } catch {} }
    [System.Threading.Thread]::Sleep(3000)
    foreach ($procId in $pids) {
      if (Get-Process -Id $procId -ErrorAction SilentlyContinue) {
        try { Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue } catch {}
      }
    }
  }
}

function Get-DescendantPids([int]$rootPid) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
  $childrenByParent = @{}
  foreach ($p in $all) {
    if (-not $childrenByParent.ContainsKey($p.ParentProcessId)) {
      $childrenByParent[$p.ParentProcessId] = New-Object System.Collections.Generic.List[int]
    }
    $childrenByParent[$p.ParentProcessId].Add([int]$p.ProcessId)
  }
  $seen = New-Object System.Collections.Generic.HashSet[int]
  $q = New-Object System.Collections.Generic.Queue[int]
  $q.Enqueue($rootPid)
  [void]$seen.Add($rootPid)
  while ($q.Count -gt 0) {
    $cur = $q.Dequeue()
    if ($childrenByParent.ContainsKey($cur)) {
      foreach ($ch in $childrenByParent[$cur]) {
        if ($seen.Add($ch)) { $q.Enqueue($ch) }
      }
    }
  }
  @($seen)
}

function Run-Scenario([string]$name,[string]$traceVal) {
  Stop-Conflicts
  $start = Get-Date
  $logFile = Join-Path $benchDir ("$name.log")
  $outJson = Join-Path $benchDir ("$name.json")
  if (Test-Path $logFile) { Remove-Item $logFile -Force }
  if (Test-Path $outJson) { Remove-Item $outJson -Force }

  $cmd = "/c set MICONTROL_DEV_TRACE=$traceVal&& npm run tauri dev > `"$logFile`" 2>&1"
  $runner = Start-Process -FilePath cmd.exe -ArgumentList $cmd -PassThru -WindowStyle Hidden

  $root = $null
  $deadline = (Get-Date).AddSeconds(120)
  while ((Get-Date) -lt $deadline -and -not $root) {
    $cands = Get-CimInstance Win32_Process | Where-Object {
      $_.Name -eq 'micontrol.exe' -and
      $_.ExecutablePath -like '*\\micontrol\\src-tauri\\target\\debug\\micontrol.exe' -and
      $_.CreationDate -ge $start
    } | Sort-Object CreationDate
    if ($cands) { $root = $cands[0] }
    if (-not $root) { [System.Threading.Thread]::Sleep(1000) }
    if ($runner.HasExited) { break }
  }

  if (-not $root) {
    cmd /c "taskkill /PID $($runner.Id) /T /F" | Out-Null
    $res = [ordered]@{ scenario=$name; success=$false; reason='micontrol.exe not detected'; logFile=$logFile }
    $res | ConvertTo-Json -Depth 5 | Set-Content -Path $outJson -Encoding UTF8
    return $res
  }

  [System.Threading.Thread]::Sleep(20000)
  $lp = [Environment]::ProcessorCount
  $samples = New-Object System.Collections.Generic.List[object]
  $perPidCpu = @{}
  $prevCpu = @{}
  $prevT = Get-Date
  $totalElapsed = 0.0

  for ($i=0; $i -lt 30; $i++) {
    [System.Threading.Thread]::Sleep(2000)
    $now = Get-Date
    $dt = ($now - $prevT).TotalSeconds
    if ($dt -le 0) { $dt = 2.0 }
    $treePids = Get-DescendantPids -rootPid ([int]$root.ProcessId)
    $plist = Get-Process -ErrorAction SilentlyContinue | Where-Object { $treePids -contains $_.Id }
    $sumDelta = 0.0
    foreach ($p in $plist) {
      $cpuNow = [double]$p.CPU
      $delta = 0.0
      if ($prevCpu.ContainsKey($p.Id)) {
        $delta = $cpuNow - [double]$prevCpu[$p.Id]
        if ($delta -lt 0) { $delta = 0.0 }
      }
      $prevCpu[$p.Id] = $cpuNow
      $sumDelta += $delta
      if (-not $perPidCpu.ContainsKey($p.Id)) { $perPidCpu[$p.Id] = [ordered]@{ Name=$p.ProcessName; CpuSec=0.0 } }
      $perPidCpu[$p.Id].CpuSec += $delta
    }
    $pct = if ($lp -gt 0) { ($sumDelta / $dt / $lp) * 100.0 } else { 0.0 }
    $samples.Add([pscustomobject]@{ Pct=$pct; Count=$plist.Count }) | Out-Null
    $totalElapsed += $dt
    $prevT = $now
    if ($runner.HasExited) { break }
  }

  cmd /c "taskkill /PID $($runner.Id) /T /F" | Out-Null
  [System.Threading.Thread]::Sleep(3000)

  if ($samples.Count -eq 0) {
    $res = [ordered]@{ scenario=$name; success=$false; reason='no samples collected'; logFile=$logFile }
    $res | ConvertTo-Json -Depth 6 | Set-Content -Path $outJson -Encoding UTF8
    return $res
  }

  $pcts = @($samples | ForEach-Object { [double]$_.Pct })
  $sorted = @($pcts | Sort-Object)
  $idx95 = [Math]::Ceiling(0.95 * $sorted.Count) - 1
  if ($idx95 -lt 0) { $idx95 = 0 }
  $top = $perPidCpu.GetEnumerator() | ForEach-Object {
    $avgPct = if ($totalElapsed -gt 0 -and $lp -gt 0) { ($_.Value.CpuSec / $totalElapsed / $lp) * 100.0 } else { 0.0 }
    [pscustomobject]@{ Pid=$_.Key; Name=$_.Value.Name; AvgPct=[Math]::Round($avgPct,4) }
  } | Sort-Object AvgPct -Descending | Select-Object -First 5

  $res = [ordered]@{
    scenario=$name
    success=$true
    trace=$traceVal
    unit='% total machine CPU'
    avg=[Math]::Round(($pcts | Measure-Object -Average).Average,4)
    p95=[Math]::Round($sorted[$idx95],4)
    max=[Math]::Round(($pcts | Measure-Object -Maximum).Maximum,4)
    avgProcessCount=[Math]::Round((($samples | ForEach-Object { $_.Count }) | Measure-Object -Average).Average,2)
    sampleCount=$samples.Count
    totalElapsedSec=[Math]::Round($totalElapsed,3)
    topContributors=$top
    logFile=$logFile
  }
  $res | ConvertTo-Json -Depth 8 | Set-Content -Path $outJson -Encoding UTF8
  $res
}

Run-Scenario -name $ScenarioName -traceVal $TraceVal | ConvertTo-Json -Depth 8
