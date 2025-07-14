@echo off
setlocal enabledelayedexpansion

REM Script for automatically updating dependencies in a "vendored" Rust project.
REM
REM Usage:
REM   vendor-update.bat add <crate-name> [other cargo add flags]
REM   vendor-update.bat update [other cargo update flags]
REM
REM Examples:
REM   vendor-update.bat add serde --features derive
REM   vendor-update.bat update -p tokio
REM   vendor-update.bat update

REM Check that the script is run from project root and arguments are provided
if not exist "Cargo.toml" (
    echo Error: Cargo.toml not found. Please run the script from your project's root directory.
    exit /b 1
)

if "%~1"=="" (
    echo Error: no Cargo command specified.
    echo Usage: %0 ^<add^|update^> [arguments...]
    echo Example: %0 add serde --features derive
    exit /b 1
)

set CONFIG_FILE=.cargo\config.toml

REM --- STEP 1: Temporarily disable offline mode ---
echo --^> Step 1: Temporarily disabling offline mode...
if not exist "%CONFIG_FILE%" (
    echo Warning: %CONFIG_FILE% not found. Skipping this step.
    echo The project may not have been vendored yet.
) else (
    move "%CONFIG_FILE%" "%CONFIG_FILE%.bak" >nul
    echo File %CONFIG_FILE% renamed to %CONFIG_FILE%.bak
)
echo.

REM --- STEP 2: Add or update dependencies ---
set CARGO_COMMAND=%1
shift
set CARGO_ARGS=
:parse_args
if "%~1"=="" goto args_done
set CARGO_ARGS=%CARGO_ARGS% %1
shift
goto parse_args
:args_done

echo --^> Step 2: Executing 'cargo %CARGO_COMMAND%%CARGO_ARGS%'...
cargo %CARGO_COMMAND%%CARGO_ARGS%
if errorlevel 1 (
    echo Error: cargo command failed
    goto restore_config
)
echo.

REM --- STEP 3: Update vendor directory contents ---
echo --^> Step 3: Updating local sources ^(vendoring^)...
REM Remove old directory if it exists to avoid garbage
if exist "vendor" (
    echo Removing old vendor directory...
    rmdir /s /q vendor
)
cargo vendor
if errorlevel 1 (
    echo Error: cargo vendor failed
    goto restore_config
)
echo Vendor directory successfully created/updated.
echo.

REM --- STEP 4: Restore offline mode ---
:restore_config
echo --^> Step 4: Restoring offline mode...
if not exist "%CONFIG_FILE%.bak" (
    echo Warning: %CONFIG_FILE%.bak not found. Skipping.
) else (
    move "%CONFIG_FILE%.bak" "%CONFIG_FILE%" >nul
    echo Offline mode configuration restored.
)
echo.

REM --- STEP 5: Verification ---
echo --^> Step 5: Testing offline build...
cargo build --offline
if errorlevel 1 (
    echo Error: Offline build test failed
    exit /b 1
)
echo Offline build test passed successfully!
echo.

REM --- STEP 6: Commit changes to Git ---
echo --^> Step 6: Preparing for Git commit.
echo Changes ready for commit:
git status -s
echo.

set /p "COMMIT_CHOICE=Do you want to commit these changes now? (y/N) "
if /i not "%COMMIT_CHOICE%"=="y" (
    echo Commit skipped. Please review and commit changes manually.
    goto end
)

set /p "COMMIT_MESSAGE=Enter commit message (or press Enter for default): "
if "%COMMIT_MESSAGE%"=="" (
    set COMMIT_MESSAGE=build: Update vendored dependencies
)

echo Adding files to index: Cargo.toml, Cargo.lock, vendor/, .cargo\config.toml
REM Add all necessary files, even if they haven't changed (git will figure it out)
git add Cargo.toml Cargo.lock vendor/ .cargo\config.toml

echo Committing with message: '%COMMIT_MESSAGE%'
git commit -m "%COMMIT_MESSAGE%"
if errorlevel 1 (
    echo Error: Git commit failed
    exit /b 1
)
echo Changes successfully committed.

:end
echo.
echo All steps completed.
