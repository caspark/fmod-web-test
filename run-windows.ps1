#!/usr/bin/env powershell.exe
# PowerShell script to run FMOD test on Windows

param(
    [Parameter(Mandatory=$false)]
    [string]$Path = "C:\Program Files (x86)\FMOD SoundSystem\FMOD Studio API Windows\api\studio\examples"
)

# FMOD installation directory
$FMOD_DIR = "C:\Program Files (x86)\FMOD SoundSystem\FMOD Studio API Windows"

# Get the directory where this script is located
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path

# Create target directory if it doesn't exist
$TARGET_DIR = "$SCRIPT_DIR\target\debug"
if (-not (Test-Path $TARGET_DIR)) {
    New-Item -ItemType Directory -Force -Path $TARGET_DIR
}

# Copy FMOD DLLs to the target directory
Copy-Item "$FMOD_DIR\api\studio\lib\x64\fmodstudio.dll" -Destination $TARGET_DIR -Force
Copy-Item "$FMOD_DIR\api\studio\lib\x64\fmodstudioL.dll" -Destination $TARGET_DIR -Force
Copy-Item "$FMOD_DIR\api\core\lib\x64\fmod.dll" -Destination $TARGET_DIR -Force
Copy-Item "$FMOD_DIR\api\core\lib\x64\fmodL.dll" -Destination $TARGET_DIR -Force

# Run the project with the provided path
cargo run -- $Path
