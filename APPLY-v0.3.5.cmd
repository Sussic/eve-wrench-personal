@echo off
setlocal
cd /d "%~dp0"

if not exist package.json (
    echo Extract this overlay into the root of eve-wrench-personal first.
    exit /b 1
)

del /q src\components\UpdateModal.vue 2>nul
del /q src\composables\useUpdateChecker.ts 2>nul

echo EVE Wrench Personal v0.3.5 files are in place.
echo Run the verification commands from the patch notes before building.

