# Tool integrations — dot-sourced from $PROFILE stub (created by `just link`).

$env:XDG_CONFIG_HOME ??= Join-Path $HOME '.config'
$env:GIT_USERNAME = 'luan'

# Interactive shells only, like zsh conf.d.
if ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected) { return }

# -CommandType Application: a plain Get-Command miss falls back to module
# auto-discovery, which scans PSModulePath (a network share here) at ~700ms per miss.
function Test-Tool([string] $name) { [bool] (Get-Command $name -CommandType Application -ErrorAction SilentlyContinue) }

# Spawning each tool to emit its static init script costs 50-150ms per shell start;
# cache the output and invalidate when the binary changes (mirrors the zsh carapace cache).
function Get-CachedInit([string] $tool, [string[]] $arguments) {
    $bin = (Get-Command $tool -CommandType Application -ErrorAction SilentlyContinue).Source
    if (-not $bin) { return $null }
    $dir = Join-Path ($env:XDG_CACHE_HOME ?? (Join-Path $HOME '.cache')) 'powershell'
    $cache = Join-Path $dir "$tool.ps1"
    if (-not (Test-Path $cache) -or (Get-Item $cache).LastWriteTime -lt (Get-Item $bin).LastWriteTime) {
        New-Item -ItemType Directory -Force $dir | Out-Null
        (& $bin @arguments) -join "`n" | Set-Content $cache
    }
    $cache
}

# Prompt-critical path: starship owns the prompt, zoxide hooks it, mise prepends shims.
# starship's full-init skips its bootstrap stub, which would re-spawn the binary on load.
# mise shims instead of hooks: its pwsh command-not-found hook crashes on empty history.
if ($init = Get-CachedInit mise @('activate', 'pwsh', '--shims')) { . $init }
if ($init = Get-CachedInit zoxide @('init', 'powershell')) { . $init }
if ($init = Get-CachedInit starship @('init', 'powershell', '--print-full-init')) { . $init }

if (Test-Tool eza) {
    Remove-Alias ls -ErrorAction SilentlyContinue # built-in alias outranks the function
    function ls { eza --icons @args }
    function ll { eza --icons -l @args }
    function la { eza --icons -la @args }
    function lt { eza --icons --tree @args }
}

if (Test-Tool git-spice) { Set-Alias gs git-spice }
if (Test-Tool nvim) { Set-Alias vim nvim }

function lg {
    $env:LAZYGIT_NEW_DIR_FILE = Join-Path $HOME '.lazygit\newdir'
    lazygit @args
    if (Test-Path $env:LAZYGIT_NEW_DIR_FILE) {
        Set-Location (Get-Content $env:LAZYGIT_NEW_DIR_FILE)
        Remove-Item $env:LAZYGIT_NEW_DIR_FILE -Force
    }
}

# Completions and keybindings aren't needed for the first prompt — defer them to
# first idle to keep them off the startup critical path.
$null = Register-EngineEvent PowerShell.OnIdle -MaxTriggerCount 1 -Action {
    # History autosuggestions + syntax highlighting (parity with zsh-autosuggestions /
    # zsh-syntax-highlighting). Prediction requires VT, absent in redirected consoles.
    try { Set-PSReadLineOption -PredictionSource History -PredictionViewStyle ListView } catch {}
    Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete

    if ($init = Get-CachedInit carapace @('_carapace', 'powershell')) { . $init }
    if ($init = Get-CachedInit jj @('util', 'completion', 'power-shell')) { . $init }

    # fzf keybindings: Ctrl+R history search, Ctrl+T file picker
    if ((Test-Tool fzf) -and (Get-Module PSFzf -ListAvailable)) {
        Import-Module PSFzf -Global
        Set-PsFzfOption -PSReadlineChordProvider 'Ctrl+t' -PSReadlineChordReverseHistory 'Ctrl+r'
    }
}
