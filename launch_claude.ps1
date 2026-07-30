# launch_claude.ps1 - Launch Claude Code with Alibaba Cloud MaaS backend
# Usage: .\launch_claude.ps1 [model-id]
# Models (Alibaba Cloud Token Plan - Standard):
#   qwen3.8-max-preview  - Reasoning, visual, text (10x credits discount, preview)
#   qwen3.7-max          - Reasoning, text
#   qwen3.7-plus         - Reasoning, visual, text  [DEFAULT]
#   qwen3.6-flash        - Reasoning, visual, text
#   glm-5.2              - Reasoning, text (Zhipu AI)
#   deepseek-v4-pro      - Reasoning, text (DeepSeek)

$config = Get-Content "$PSScriptRoot\.claude\alibaba_cloud_config.env" |
    Where-Object { $_ -notmatch '^\s*#' -and $_ -match '=' } |
    ForEach-Object {
        $parts = $_ -split '=', 2
        [PSCustomObject]@{ Key = $parts[0].Trim(); Value = $parts[1].Trim() }
    }

foreach ($entry in $config) {
    [System.Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, 'Process')
}

if (-not $env:ANTHROPIC_API_KEY -or $env:ANTHROPIC_API_KEY -eq 'REPLACE_ME') {
    Write-Host "[ERROR] Set ANTHROPIC_API_KEY in .claude\alibaba_cloud_config.env first."
    exit 1
}

# Model selection: pass as first arg, else default to qwen3.7-plus
$model = if ($args[0]) { $args[0] } else { "qwen3.7-plus" }

Write-Host "[OK] Backend : $env:ANTHROPIC_BASE_URL"
Write-Host "[OK] Model   : $model"
Write-Host "[OK] Launching Claude Code..."

claude --model $model --dangerously-skip-permissions
