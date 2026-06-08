param(
    [string]$ServiceDxc = "",
    [string]$XportBin = "xport",
    [int]$BasePort = 28710
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $PSCommandPath }
$ServiceRoot = [System.IO.Path]::GetFullPath((Join-Path $ScriptDir ".."))
$WorkspaceRoot = [System.IO.Path]::GetFullPath((Join-Path $ServiceRoot ".."))
$TemplateNode = [System.IO.Path]::GetFullPath((Join-Path $WorkspaceRoot "nodes\us-option-strategy-task-node"))
$SmokeRoot = [System.IO.Path]::GetFullPath((Join-Path $ServiceRoot "target\runtime-smoke\sim_account_runtime_xrpc"))
$NodeDir = [System.IO.Path]::GetFullPath((Join-Path $SmokeRoot "node"))
$ServiceDataDir = [System.IO.Path]::GetFullPath((Join-Path $SmokeRoot "service_data"))
$DefaultServiceDxc = Join-Path $ServiceRoot "UsOptionSimAccountService-0.0.1-win-x86_64.dxc"
$ServiceDxcFileName = "UsOptionSimAccountService-0.0.1-win-x86_64.dxc"
$ServiceDxcDirName = "UsOptionSimAccountService-0.0.1-win-x86_64"

if ([string]::IsNullOrWhiteSpace($ServiceDxc)) {
    $ServiceDxcPath = $DefaultServiceDxc
} elseif ([System.IO.Path]::IsPathRooted($ServiceDxc)) {
    $ServiceDxcPath = $ServiceDxc
} else {
    $ServiceDxcPath = Join-Path $ServiceRoot $ServiceDxc
}
$ServiceDxcPath = [System.IO.Path]::GetFullPath($ServiceDxcPath)

function Assert-PathInside {
    param(
        [string]$Path,
        [string]$Root,
        [string]$Label
    )

    $FullPath = [System.IO.Path]::GetFullPath($Path)
    $FullRoot = [System.IO.Path]::GetFullPath($Root)
    $Separator = [System.IO.Path]::DirectorySeparatorChar.ToString()
    $RootPrefix = $FullRoot
    if (-not $RootPrefix.EndsWith($Separator, [System.StringComparison]::Ordinal)) {
        $RootPrefix += $Separator
    }
    if (
        -not $FullPath.Equals($FullRoot, [System.StringComparison]::OrdinalIgnoreCase) -and
        -not $FullPath.StartsWith($RootPrefix, [System.StringComparison]::OrdinalIgnoreCase)
    ) {
        throw "$Label path is outside allowed root: $FullPath"
    }
    return $FullPath
}

function Remove-SmokePath {
    param([string]$Path)

    $FullPath = Assert-PathInside -Path $Path -Root $SmokeRoot -Label "smoke scratch"
    if (Test-Path -LiteralPath $FullPath) {
        Remove-Item -LiteralPath $FullPath -Recurse -Force
    }
}

function Resolve-ToolForUse {
    param(
        [string]$ToolName,
        [string]$Label
    )

    if ([string]::IsNullOrWhiteSpace($ToolName)) {
        throw "$Label cannot be empty"
    }

    $HasPathSeparator = $ToolName.Contains("\") -or $ToolName.Contains("/") -or $ToolName.Contains(":")
    if ($HasPathSeparator) {
        if ([System.IO.Path]::IsPathRooted($ToolName)) {
            $Candidate = $ToolName
        } else {
            $Candidate = Join-Path $ServiceRoot $ToolName
        }
        $Candidate = [System.IO.Path]::GetFullPath($Candidate)
        if (-not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
            throw "$Label not found: $Candidate"
        }
        return $Candidate
    }

    $Command = Get-Command $ToolName -ErrorAction SilentlyContinue
    if ($null -eq $Command) {
        throw "$Label not found on PATH: $ToolName"
    }
    return $ToolName
}

function Test-PortBindable {
    param([int]$Port)

    $Listener = $null
    try {
        $Address = [System.Net.IPAddress]::Parse("127.0.0.1")
        $Listener = [System.Net.Sockets.TcpListener]::new($Address, $Port)
        $Listener.Start()
        return $true
    } catch {
        return $false
    } finally {
        if ($null -ne $Listener) {
            $Listener.Stop()
        }
    }
}

function Test-PortListening {
    param([int]$Port)

    $Client = [System.Net.Sockets.TcpClient]::new()
    $WaitHandle = $null
    try {
        $Async = $Client.BeginConnect("127.0.0.1", $Port, $null, $null)
        $WaitHandle = $Async.AsyncWaitHandle
        if (-not $WaitHandle.WaitOne(300, $false)) {
            return $false
        }
        $Client.EndConnect($Async)
        return $true
    } catch {
        return $false
    } finally {
        if ($null -ne $WaitHandle) {
            $WaitHandle.Close()
        }
        $Client.Close()
    }
}

function Assert-PortsFree {
    param([int[]]$Ports)

    foreach ($Port in $Ports) {
        if ($Port -lt 1 -or $Port -gt 65535) {
            throw "port is outside TCP range: $Port"
        }
        if (Test-PortListening -Port $Port) {
            throw "port has a listener before smoke starts: 127.0.0.1:$Port"
        }
        if (-not (Test-PortBindable -Port $Port)) {
            throw "port is not free before smoke starts: 127.0.0.1:$Port"
        }
    }
}

function Wait-PortListening {
    param(
        [int]$Port,
        [int]$TimeoutSeconds = 20
    )

    $Deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $Deadline) {
        if (Test-PortListening -Port $Port) {
            return $true
        }
        Start-Sleep -Milliseconds 200
    }
    return $false
}

function Wait-PortsWithoutListeners {
    param(
        [int[]]$Ports,
        [int]$TimeoutSeconds = 10
    )

    $Deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $Deadline) {
        $AnyListening = $false
        foreach ($Port in $Ports) {
            if (Test-PortListening -Port $Port) {
                $AnyListening = $true
                break
            }
        }
        if (-not $AnyListening) {
            return $true
        }
        Start-Sleep -Milliseconds 200
    }
    return $false
}

function Invoke-NativeCapture {
    param(
        [string]$File,
        [string[]]$Arguments,
        [string]$WorkingDirectory
    )

    $PreviousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    Push-Location $WorkingDirectory
    try {
        $Output = & $File @Arguments 2>&1 | ForEach-Object { $_.ToString() }
        $ExitCode = $LASTEXITCODE
    } finally {
        Pop-Location
        $ErrorActionPreference = $PreviousErrorActionPreference
    }

    return [pscustomobject]@{
        ExitCode = $ExitCode
        Output = [string[]]$Output
    }
}

function Write-CapturedOutput {
    param([string[]]$Output)

    foreach ($Line in $Output) {
        Write-Host $Line
    }
}

function Format-TomlString {
    param([string]$Value)

    $Escaped = $Value.Replace("\", "\\").Replace('"', '\"')
    return '"' + $Escaped + '"'
}

function Write-Dxmesh {
    param([int[]]$Ports)

    $ServiceDataPath = ([System.IO.Path]::GetFullPath($ServiceDataDir)).Replace("\", "/")
    $Text = @"
[xport]
name = "UsOptionSimAccountRuntimeSmokeNode"
xrpc-ip = "127.0.0.1"
xrpc-port = "$($Ports[0])"
log-path = "./log"
log-level = "info"
log-writer = ["console"]
msg-timeout = 30
default-dxc = ["XComService-0.0.1", "DnsResolverService-0.0.1", "UsOptionSimAccountService-0.0.1"]
message-max-size = "10M"

["XComService-0.0.1"]
system = true
only-in-node = true
mode = "single"
file-port = $($Ports[1])
refresh-register-time = 20
validate-component = false
compatible-versions = []

["DnsResolverService-0.0.1"]
system = true

["UsOptionSimAccountService-0.0.1"]
path = $(Format-TomlString -Value $ServiceDataPath)
"@
    $Utf8NoBom = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText((Join-Path $NodeDir "dxmesh.toml"), $Text + [System.Environment]::NewLine, $Utf8NoBom)
}

function Copy-SystemDxc {
    param([string]$Name)

    $TemplateDxcDir = Join-Path $TemplateNode "dxc"
    foreach ($Item in @("$Name-win-x86_64.dxc", "$Name-win-x86_64")) {
        $Source = Join-Path $TemplateDxcDir $Item
        if (-not (Test-Path -LiteralPath $Source)) {
            throw "template system dxc missing: $Source"
        }
        Copy-Item -LiteralPath $Source -Destination (Join-Path $NodeDir "dxc") -Recurse
    }
}

function Prepare-Node {
    if (-not (Test-Path -LiteralPath $TemplateNode -PathType Container)) {
        throw "template node does not exist: $TemplateNode"
    }
    Remove-SmokePath -Path $SmokeRoot
    New-Item -ItemType Directory -Path (Join-Path $NodeDir "dxc") -Force | Out-Null
    New-Item -ItemType Directory -Path $ServiceDataDir -Force | Out-Null
    Copy-SystemDxc -Name "XComService-0.0.1"
    Copy-SystemDxc -Name "DnsResolverService-0.0.1"
    Write-Dxmesh -Ports @($BasePort, ($BasePort + 1))
}

function Install-ServiceDxcWithoutInstallCommand {
    $DxcDir = Join-Path $NodeDir "dxc"
    $InstalledDxc = Join-Path $DxcDir $ServiceDxcFileName
    Copy-Item -LiteralPath $ServiceDxcPath -Destination $InstalledDxc -Force
    $UnpackDir = Join-Path $DxcDir $ServiceDxcDirName
    if (Test-Path -LiteralPath $UnpackDir) {
        Remove-SmokePath -Path $UnpackDir
    }
    New-Item -ItemType Directory -Path $UnpackDir -Force | Out-Null
    $Result = Invoke-NativeCapture `
        -File "tar" `
        -Arguments @("-xf", $InstalledDxc, "-C", $UnpackDir) `
        -WorkingDirectory $ServiceRoot
    if ($Result.ExitCode -ne 0) {
        Write-CapturedOutput -Output $Result.Output
        throw "service dxc tar unpack failed"
    }
    $SourceHash = (Get-FileHash -LiteralPath $ServiceDxcPath -Algorithm SHA256).Hash
    $InstalledHash = (Get-FileHash -LiteralPath $InstalledDxc -Algorithm SHA256).Hash
    if ($SourceHash -ne $InstalledHash) {
        Write-Host "installed_dxc_hash_matches = false"
        throw "installed service dxc hash does not match source package"
    }
    Write-Host "installed_dxc_hash_matches = true"
}

function Assert-OutputLine {
    param(
        [string[]]$Output,
        [string]$ExpectedLine
    )

    if (-not ($Output -contains $ExpectedLine)) {
        throw "missing output line: $ExpectedLine"
    }
}

function Start-XportProcess {
    param([string]$XportForSmoke)

    $Stdout = Join-Path $NodeDir "sim_account_runtime_stdout.log"
    $Stderr = Join-Path $NodeDir "sim_account_runtime_stderr.log"
    return Start-Process `
        -FilePath $XportForSmoke `
        -WorkingDirectory $NodeDir `
        -RedirectStandardOutput $Stdout `
        -RedirectStandardError $Stderr `
        -WindowStyle Hidden `
        -PassThru
}

function Stop-XportProcess {
    param([System.Diagnostics.Process]$Process)

    if ($null -eq $Process) {
        return
    }
    try {
        if (-not $Process.HasExited) {
            Stop-Process -Id $Process.Id -Force
            $null = $Process.WaitForExit(5000)
        }
    } catch {
        Write-Host ("runtime_stop_warning = " + $_.Exception.Message)
    }
}

function Invoke-Smoke {
    $Result = Invoke-NativeCapture `
        -File "cargo" `
        -Arguments @(
            "run",
            "--features",
            "runtime-smoke",
            "--bin",
            "sim_account_runtime_xrpc_smoke",
            "--",
            "--xrpc-port",
            [string]$BasePort
        ) `
        -WorkingDirectory $ServiceRoot
    if ($Result.ExitCode -ne 0) {
        Write-CapturedOutput -Output $Result.Output
        throw "sim account runtime typed XRPC smoke failed"
    }

    foreach ($Line in @(
        "runtime_typed_invoke_ok = true",
        "create_sim_account_ok = true",
        "get_sim_account_ok = true",
        "list_sim_accounts_ok = true",
        "update_sim_account_ok = true",
        "list_sim_account_audit_events_ok = true",
        "get_sim_account_service_health_ok = true",
        "account_count_matches = true",
        "audit_event_count_matches = true",
        "PASS SimAccount runtime typed XRPC smoke"
    )) {
        Assert-OutputLine -Output $Result.Output -ExpectedLine $Line
    }
    Write-CapturedOutput -Output $Result.Output
}

function Invoke-Verify {
    Write-Host "UsOptionSimAccountService runtime typed XRPC smoke"
    Write-Host "service_root = $ServiceRoot"
    Write-Host "template_node = $TemplateNode"
    Write-Host "smoke_root = $SmokeRoot"
    Write-Host "service_dxc = $ServiceDxcPath"
    Write-Host "base_port = $BasePort"

    if (-not (Test-Path -LiteralPath $ServiceDxcPath -PathType Leaf)) {
        throw "service dxc does not exist: $ServiceDxcPath"
    }
    Assert-PathInside -Path $SmokeRoot -Root (Join-Path $ServiceRoot "target") -Label "smoke root" | Out-Null
    Assert-PathInside -Path $NodeDir -Root $SmokeRoot -Label "node dir" | Out-Null
    Assert-PathInside -Path $ServiceDataDir -Root $SmokeRoot -Label "service data dir" | Out-Null

    $Ports = @($BasePort, ($BasePort + 1))
    Assert-PortsFree -Ports $Ports

    $XportForSmoke = Resolve-ToolForUse -ToolName $XportBin -Label "xport"
    Resolve-ToolForUse -ToolName "adxbuilder" -Label "adxbuilder" | Out-Null
    Write-Host "xport = $XportForSmoke"
    Write-Host "preflight = PASS"

    Prepare-Node
    Install-ServiceDxcWithoutInstallCommand

    $Process = $null
    try {
        $Process = Start-XportProcess -XportForSmoke $XportForSmoke
        Write-Host "runtime_started = true"
        if (-not (Wait-PortListening -Port $BasePort -TimeoutSeconds 20)) {
            throw "runtime xrpc port did not become ready: 127.0.0.1:$BasePort"
        }
        Write-Host "runtime_ready = true"
        Invoke-Smoke
    } finally {
        Stop-XportProcess -Process $Process
    }

    Write-Host "runtime_stopped = true"
    if (-not (Wait-PortsWithoutListeners -Ports $Ports -TimeoutSeconds 10)) {
        throw "one or more sim account runtime smoke ports still have listeners"
    }
    Write-Host "runtime_ports_closed = true"
    Write-Host "PASS UsOptionSimAccountService runtime typed XRPC smoke"
}

try {
    Invoke-Verify
    exit 0
} catch {
    Write-Host "FAIL UsOptionSimAccountService runtime typed XRPC smoke"
    Write-Host $_.Exception.Message
    exit 1
}
