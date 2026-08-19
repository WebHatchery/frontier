<#
.SYNOPSIS
    Headless screenshot harness for Frontier Kingdom.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (FRONTIER_KINGDOM_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes seed a GameState via Game::begin_capture_scene:
    "menu" (title screen), "base" (kingdom command table), "missions" (mission
    select screen), "mission" (expedition route), "event" (narrative choice),
    "combat" (combat arena), "results" (mission aftermath), and "recruit"
    (recruitment screen).

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "base", "missions", "mission", "event", "combat", "results", "recruit"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
