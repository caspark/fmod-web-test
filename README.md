
## FMOD SDK

Download fmod sdk version 2.02.22 and extract like this:

* fmod/linux/
* fmod/web/

Example paths:

* fmod/linux/api/core/inc/fmod.h
* fmod/linux/api/core/lib/x86_64/libfmod.so

The include files are looked for by the fmod-oxide (specifically fmod-audio-sys) build scripts for bindgen.

The .so files are loaded at runtime by fmod.

## Run

```sh
# sets FMOD_SYS_FMOD_DIRECTORY and LD_LIBRARY_PATH as needed
./run-linux.sh

# or

./run-web.sh
```

## Vscode config

In `.vscode/settings.json`, for linux:

```jsonc
{
    // "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
    "editor.formatOnSave": true,
    "rust-analyzer.cargo.extraEnv": {
        "LD_LIBRARY_PATH": "${workspaceFolder}/fmod/linux/api/core/lib/x86_64:${workspaceFolder}/fmod/linux/api/studio/lib/x86_64",
        "FMOD_SYS_FMOD_DIRECTORY": "${workspaceFolder}/fmod/linux"
    }
}
```
