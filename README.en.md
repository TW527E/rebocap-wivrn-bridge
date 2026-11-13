[简体中文](README.md) · [繁體中文](README.zh-TW.md) · **English** · [日本語](README.ja.md)

# monad2steamvr

A CLI that uses Rebocap for VRChat full-body tracking on Linux with WiVRn + xrizer. It does not use VRChat OSC;
it passes poses as ordinary trackers. **SteamVR is not needed.**

This repository is a fork of [akazdayo/rebocap-wivrn-bridge](https://github.com/akazdayo/rebocap-wivrn-bridge) with these changes:

- **Works with the official Linux Rebocap**: the official Linux version listens on `127.0.0.1:20480` itself.
  Connect to it with `--bridge 127.0.0.1:20480`; no Wine and no `bridge.exe`.
- **Lower latency**: tracker poses used to wait in two 10 ms polling loops (about 10 ms on average, up to ~20 ms).
  They are now sent to WiVRn as soon as they arrive.
- **Moved play space**: reads WiVRn's STAGE offset through libmonado, so the trackers stay on the body after you
  drag the play space (for example with WayVR).

> [!TIP]
> The complete setup guide and the `rebocap-fbt` launcher (detects official / Wine, checks body parts, USB connection)
> are in [TW527E/rebocap-linux-wivrn-fbt](https://github.com/TW527E/rebocap-linux-wivrn-fbt).

```text
Official Linux version:
WiVRn head pose → this CLI → Rebocap (:20480)
Rebocap tracker poses → this CLI → SolarXR → WiVRn → xrizer → VRChat

Wine:
WiVRn head pose → this CLI → Wine bridge bridge.exe (:36850) → Rebocap
Rebocap tracker poses → bridge.exe → this CLI → SolarXR → WiVRn → xrizer → VRChat
```

## Requirements

- x86_64 Linux
- WiVRn + xrizer and a headset that connects to them
- A Rebocap receiver and trackers, plus one of:
  - **the official Linux Rebocap** (recommended)
  - the Windows Rebocap in Wine with `bridge.exe` from [rebocap-linux-compat](https://github.com/TW527E/rebocap-linux-compat)
    (the original author sets Wine up with [rebocap-linux](https://github.com/akazdayo/rebocap-linux); Bottles also works, see [rebocap-wine](https://github.com/TW527E/rebocap-wine))
- Building needs Rust and the OpenXR loader development files (Arch: `sudo pacman -S rust openxr pkgconf`)

Tested with WiVRn 26.9 + xrizer 0.5 / 0.6. It uses WiVRn's SolarXR and xrizer's `monado` feature (both enabled by default in this combination).

## Build

```sh
git clone https://github.com/TW527E/rebocap-wivrn-bridge
cd rebocap-wivrn-bridge
cargo build --release      # output: target/release/monad2steamvr
```

## Usage

Start the WiVRn server, **do not connect the headset yet**, then:

### 1. Start Rebocap

- **Official Linux version**: just open Rebocap.
- **Wine**: after Rebocap is running, start `bridge.exe` in **the same Wine prefix** and leave that terminal open:

  ```sh
  WINEPREFIX="${XDG_DATA_HOME:-$HOME/.local/share}/rebocap/wineprefix" wine ./bridge.exe
  ```

### 2. Start the CLI in another terminal

```sh
# official Linux version
./target/release/monad2steamvr --solarxr --bridge 127.0.0.1:20480

# Wine (--bridge defaults to 127.0.0.1:36850)
./target/release/monad2steamvr --solarxr
```

If your system has no default OpenXR runtime (no `active_runtime.json`), prefix the command with
`XR_RUNTIME_JSON=/usr/share/openxr/1/openxr_wivrn.json`.

When `SolarXR ready` appears, connect and put on the headset.

### 3. Calibrate

Switch Rebocap to VR mode and calibrate, then start VRChat through xrizer and run FBT calibration. By default
only **the waist and both feet** are published; change that with `--tracker-roles`.

**After restarting the CLI or changing the published body parts, reconnect the headset and restart VRChat.**
WiVRn reads the tracker list only when the headset connects.

## Options

| Option | Purpose | Default |
|---|---|---|
| `--solarxr` | Publish trackers to WiVRn | off |
| `--tracker-roles` | Comma-separated body parts to publish | `0,5,6` |
| `--bridge` | Where Rebocap is. Use `127.0.0.1:20480` for the official Linux version | `127.0.0.1:36850` (`bridge.exe`) |
| `--solarxr-offset X,Y,Z` | Output translation in metres (added after rotation) | `0,0,0` |
| `--solarxr-yaw DEGREES` | Output yaw around +Y | `0` |

Body parts: `0` waist, `1/2` left/right upper leg, `3/4` left/right lower leg, `5/6` left/right foot, `7` chest,
`9/10` left/right upper arm, `11/12` left/right forearm.

Publish every supported part:

```sh
./target/release/monad2steamvr --solarxr --tracker-roles 0,1,2,3,4,5,6,7,9,10,11,12
```

Match the parts to the trackers you actually wear. The 6-point kit has no foot trackers: `5/6` are virtual nodes
Rebocap estimates, their data comes and goes, and VRChat won't find your feet. Publish the lower legs `3/4` instead;
VRChat assigns them to the feet by position.

Poses are also written to stdout as JSON Lines: positions in metres, +Y up, -Z forward, rotations as `[w,x,y,z]`.
Connection logs go to stderr.

## Troubleshooting

| Symptom | What to check |
|---|---|
| Rebocap doesn't see the VR headset | Wine: does the bridge print `pipe open, relaying`? Does the CLI print `WiVRn headset tracking valid: true`? Is it the same Wine prefix? |
| No poses arrive from Rebocap | Is Rebocap in VR mode and calibrated? Does the CLI print `Received a Rebocap protocol message`? The official Linux version also sends nothing during a firmware update |
| Cannot connect to the official Linux version | `20480` accepts one connection at a time. Make sure the official SteamVR driver or another CLI isn't connected |
| VRChat doesn't see the trackers | Does the CLI print `SolarXR runtime connected`? Keep the CLI running, reconnect the headset, restart VRChat |
| `Failed to connect to monado service process` | The WiVRn server isn't running (an open dashboard doesn't mean the server is running) |
| Trackers drift off the body after moving the play space | Does the CLI print `libmonado unavailable`? Check that WiVRn's runtime JSON has `MND_libmonado_path` |

Log lines mentioning `Wine pipe` or `bridge.exe` come from the original code and also appear with the official
Linux version. You can ignore them.

## Development

```sh
cargo build --release
cargo test
```

The `monado_wire_compatibility` test needs WiVRn's Monado source and is skipped by default:
`MONADO_SOURCE=<path> cargo test -- --ignored`.

References: [WiVRn and OpenVR](https://github.com/WiVRn/WiVRn/blob/master/docs/steamvr.md) · [Rebocap protocol](https://github.com/dogesoulseller/rebocap-linux-compat/blob/main/docs/protocol.md)

## Third-party licenses

The Wine bridge `bridge.exe` uses code from [rebocap-linux-compat](https://github.com/dogesoulseller/rebocap-linux-compat).
Copyright (c) 2026 Marcin Czerwonka, MIT License. See [LICENSES/rebocap-linux-compat.txt](LICENSES/rebocap-linux-compat.txt).

The original author of this CLI is [akazdayo](https://github.com/akazdayo). The original repository does not yet
carry a license for the author's own code.
