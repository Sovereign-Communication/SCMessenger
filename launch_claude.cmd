@echo off
REM launch_claude.cmd - Launch Claude Code CLI with Alibaba Cloud MaaS free tier backend
REM Usage: launch_claude.cmd [model-id]

set "CONFIG_FILE=%~dp0.claude\alibaba_cloud_config.env"

if not exist "%CONFIG_FILE%" (
    echo [ERROR] Config file %CONFIG_FILE% not found.
    exit /b 1
)

for /f "usebackq tokens=1,* delims==" %%A in ("%CONFIG_FILE%") do (
    if not "%%A"=="" (
        set "LINE_START=%%A"
        if not "!LINE_START:~0,1!"=="#" (
            set "%%A=%%B"
        )
    )
)

set "MODEL=%~1"
if "%MODEL%"=="" set "MODEL=qwen3.5-flash-2026-02-23"

echo [OK] Backend : %ANTHROPIC_BASE_URL%
echo [OK] Model   : %MODEL%
echo [OK] Launching Claude Code...

claude --model %MODEL% --dangerously-skip-permissions
