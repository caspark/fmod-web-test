
# FMOD on Web and Desktop

A hacked up demo of some Rust code using a single abstraction over FMOD that works for both Desktop and Web.

* Desktop support is based on [fmod-oxide](https://github.com/melody-rs/fmod-oxide/)
* Web support is added by handwriting JS wrappers (see [./fmod-web.js]) which call FMOD's HTML5 SDK; these handwritten wrapper functions are invoked using [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/).

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

### Linux
```sh
./run-linux.sh fmod/linux/api/studio/examples/media
# or
./run-linux.sh --release -- fmod/linux/api/studio/examples/media
```

### Web

```sh
./run-web.sh
```

### Windows

Install fmod sdk 2.02.22 to the default path of C:\Program Files (x86)\FMOD SoundSystem\ then do:

```sh
./run-windows.ps1
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
