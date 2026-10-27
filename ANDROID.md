# Horizon on Android (arm64, experimental)

The game runs as a `NativeActivity` (no Java/Kotlin code): `crates/horizon-android` builds
horizon-client's game (`src/game.rs` + `src/game/`) as `libhorizon.so`, with the entry point
`crates/horizon-client/src/android_main.rs` instead of `main.rs`. Rendering is wgpu (Vulkan, GLES 3 fallback);
sound is cpal over AAudio. Status: compiles, links and packages; **not yet run on a device**.

## Build the APK (Linux host)

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
# Android SDK: cmdline-tools, then
sdkmanager "platforms;android-34" "build-tools;34.0.0" "ndk;27.2.12479018" "platform-tools"
export ANDROID_HOME=~/android/sdk            # NDK: the newest under $ANDROID_HOME/ndk, or ANDROID_NDK_HOME
./packaging/android/build_apk.sh             # -> target/android/horizon.apk (debug-signed)
PROFILE=release ./packaging/android/build_apk.sh   # slower build, faster game
CARGO_FLAGS="--features mods" ./packaging/android/build_apk.sh   # wasm code mods (untested on Android)
```

The script runs `cargo ndk` (API 26, arm64-v8a), strips the library, links the manifest
(`packaging/android/AndroidManifest.xml`) with `aapt2`, zipaligns and signs with
`~/.android/debug.keystore` (created if missing; password `android`). No Gradle.

## Install

```sh
adb install -r target/android/horizon.apk
```

Or copy the APK to the phone and open it (allow "install unknown apps"). Needs Android 8.0+ (API 26), arm64.

## Game files

Horizon never ships Minecraft's files. Prepare them **on a PC** (needs Java, see README.md):

```sh
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
cargo run --release -p horizon-prepare -- sounds     # optional
```

Then copy the *contents* of `.local/horizon-client/26.3/` (the folder holding `horizon-resources.json`) to the phone:

```
/sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

Start the app once first so Android creates `/sdcard/Android/data/dev.horizonlab.horizon/files/`.

```sh
adb shell mkdir -p /sdcard/Android/data/dev.horizonlab.horizon/files/resources
adb push .local/horizon-client/26.3/. /sdcard/Android/data/dev.horizonlab.horizon/files/resources/
```

That `files/` folder is the game home (`HORIZON_HOME` and the working directory): `options.txt`, `saves/`,
`logs/latest.log`, `crash-reports/`, `resourcepacks/` and so on live there, as in `~/.local/share/horizon` on
Linux. Existing worlds can be copied into `files/saves/`. If there is no external storage, the internal app
folder is used instead (reachable only with `adb shell run-as` on a debug build).

## Controls

- **Touch, in menus:** a finger is the mouse (tap = left click, drag = drag).
- **Touch, in the world:** left half = floating move stick (where you put your finger is its centre); right half =
  drag to look. Invisible button zones on the right edge (fractions of the screen, `src/game/touch.rs`):
  top-right corner = pause (Start), just left of it = inventory; right edge from top to bottom =
  next hotbar slot, use/place, attack/break, then at the bottom jump (outermost) and sneak.
  They go through the virtual controller, so they follow the controller bindings (`horizon-controller.txt`).
- **Keyboard/mouse** (USB/Bluetooth) arrive through winit like on desktop.
- **Gamepads:** gilrs has no Android backend, so the `controllers` feature is off. A Bluetooth pad's buttons
  may reach the game as key events only; not mapped yet.

## What differs from desktop

| Area | Android |
|---|---|
| Controllers (`gilrs`, DualSense output) | off (`controllers` feature not enabled) |
| System clipboard (`arboard`) | stubbed: in-game clipboard only |
| Text input | no on-screen keyboard (chat, sign editing, world names need a physical keyboard) |
| Surface | dropped on `suspended`, recreated on `resumed` (`Renderer::lose_surface/regain_surface`) |
| Resources | `HORIZON_RESOURCES` (set by `android_main`) |
| Logs | `files/logs/latest.log` (nothing in logcat) |
| Launcher, horizon-prepare, server binary, telemetry server | not built |

## Known risks

- Never run on hardware: the first start may fail on a GPU limit (wgpu downlevel limits, texture sizes, storage
  buffers on Mali/Adreno GLES) or on memory (`largeHeap` is set; the world uses a lot of RAM: try render distance 4–6).
- HDR (`horizon-hdr`) and Discord presence compile but do nothing useful on Android.
- Touch zones are not drawn on screen.
