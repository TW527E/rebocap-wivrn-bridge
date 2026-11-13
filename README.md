**简体中文** · [繁體中文](README.zh-TW.md) · [English](README.en.md) · [日本語](README.ja.md)

# monad2steamvr

在 Linux 的 WiVRn + xrizer 环境里，把 Rebocap 用作 VRChat 全身追踪的 CLI。不使用 VRChat OSC，而是以普通追踪器的形式传递姿态，**不需要 SteamVR**。

本仓库 fork 自 [akazdayo/rebocap-wivrn-bridge](https://github.com/akazdayo/rebocap-wivrn-bridge)，在原版基础上做了这些修改：

- **支持官方 Linux 版 Rebocap**：官方 Linux 版自己在 `127.0.0.1:20480` 监听，用 `--bridge 127.0.0.1:20480` 直接连接即可，不需要 Wine 和 `bridge.exe`。
- **更低的延迟**：原版里追踪器姿态要在两个 10 ms 的轮询循环里排队（平均多约 10 ms、最多约 20 ms），现在收到就立刻送给 WiVRn。
- **游玩区移动修正**：通过 libmonado 读取 WiVRn 的 STAGE 偏移。用 WayVR 等工具拖动游玩区后，追踪器不会再偏离身体。

> [!TIP]
> 完整的安装教程和启动脚本 `rebocap-fbt`（自动检测官方版／Wine、部位检查、USB 连接）在
> [TW527E/rebocap-linux-wivrn-fbt](https://github.com/TW527E/rebocap-linux-wivrn-fbt)。

```text
官方 Linux 版：
WiVRn 的头部姿态 → 本 CLI → Rebocap（:20480）
Rebocap 的追踪器姿态 → 本 CLI → SolarXR → WiVRn → xrizer → VRChat

Wine 版：
WiVRn 的头部姿态 → 本 CLI → Wine 桥接 bridge.exe（:36850）→ Rebocap
Rebocap 的追踪器姿态 → bridge.exe → 本 CLI → SolarXR → WiVRn → xrizer → VRChat
```

## 需求

- x86_64 Linux
- WiVRn + xrizer，以及能连上它们的头显
- Rebocap 接收器和追踪器，加上以下其中一种：
  - **官方 Linux 版 Rebocap**（推荐）
  - Wine 里的 Windows 版 Rebocap，加上 [rebocap-linux-compat](https://github.com/TW527E/rebocap-linux-compat) 的 `bridge.exe`
    （原作者是用 [rebocap-linux](https://github.com/akazdayo/rebocap-linux) 搭建 Wine 环境；Bottles 也可以，参考 [rebocap-wine](https://github.com/TW527E/rebocap-wine)）
- 构建需要 Rust 和 OpenXR loader 的开发库（Arch：`sudo pacman -S rust openxr pkgconf`）

已在 WiVRn 26.9 + xrizer 0.5／0.6 上验证。使用 WiVRn 的 SolarXR 和 xrizer 的 `monado` 功能（这个组合下默认启用）。

## 构建

```sh
git clone https://github.com/TW527E/rebocap-wivrn-bridge
cd rebocap-wivrn-bridge
cargo build --release      # 产物：target/release/monad2steamvr
```

## 使用方法

先启动 WiVRn 服务器，**头显先不要连接**，然后按下面的顺序进行。

### 1. 启动 Rebocap

- **官方 Linux 版**：直接打开 Rebocap 即可。
- **Wine 版**：启动 Rebocap 后，在**同一个 Wine prefix** 里启动 `bridge.exe`，并保持这个终端开着：

  ```sh
  WINEPREFIX="${XDG_DATA_HOME:-$HOME/.local/share}/rebocap/wineprefix" wine ./bridge.exe
  ```

### 2. 在另一个终端启动 CLI

```sh
# 官方 Linux 版
./target/release/monad2steamvr --solarxr --bridge 127.0.0.1:20480

# Wine 版（--bridge 默认就是 127.0.0.1:36850）
./target/release/monad2steamvr --solarxr
```

系统没有设置 OpenXR 默认 runtime（没有 `active_runtime.json`）时，前面加上
`XR_RUNTIME_JSON=/usr/share/openxr/1/openxr_wivrn.json`。

看到 `SolarXR ready` 后，连接并戴上头显。

### 3. 校准

把 Rebocap 切到 VR 模式并校准，再通过 xrizer 启动 VRChat，进行 FBT 校准。默认只发布**腰和左右脚 3 个点**，用 `--tracker-roles` 修改。

**重启 CLI 或修改发布的部位后，头显要重新连接，VRChat 也要重启。** WiVRn 只在头显连接时读取追踪器列表。

## 选项

| 选项 | 用途 | 默认值 |
|---|---|---|
| `--solarxr` | 把追踪器发布给 WiVRn | 关闭 |
| `--tracker-roles` | 要发布的部位，用逗号分隔 | `0,5,6` |
| `--bridge` | Rebocap 的连接地址。官方 Linux 版用 `127.0.0.1:20480` | `127.0.0.1:36850`（`bridge.exe`） |
| `--solarxr-offset X,Y,Z` | 输出位置修正（米，在旋转之后加上） | `0,0,0` |
| `--solarxr-yaw DEGREES` | 输出方向修正（绕 +Y 轴的角度） | `0` |

部位编号：`0` 腰、`1/2` 左右大腿、`3/4` 左右小腿、`5/6` 左右脚、`7` 胸、`9/10` 左右上臂、`11/12` 左右前臂。

发布所有支持部位的例子：

```sh
./target/release/monad2steamvr --solarxr --tracker-roles 0,1,2,3,4,5,6,7,9,10,11,12
```

部位要和实际绑的追踪器一致。6 点套组没有脚的实体追踪器，`5/6` 只是 Rebocap 推算出的虚拟节点，数据断断续续，VRChat 会抓不到脚；这种情况下把小腿 `3/4` 发布出去，VRChat 会按位置把它当成脚。

姿态也会以 JSON Lines 输出到标准输出：位置单位是米，+Y 向上，-Z 向前，旋转是 `[w,x,y,z]`。连接日志输出到标准错误。

## 排障

| 症状 | 检查项目 |
|---|---|
| Rebocap 认不到 VR 头显 | Wine 版：bridge 是否显示 `pipe open, relaying`；CLI 是否显示 `WiVRn headset tracking valid: true`。另外确认 Wine prefix 是否相同 |
| Rebocap 的姿态没有传过来 | 是否已切到 VR 模式并校准；CLI 是否显示 `Received a Rebocap protocol message`。官方 Linux 版在更新固件时也不会送数据 |
| 官方 Linux 版连不上 | `20480` 同一时间只接受一个连接。确认没有开着官方 SteamVR 驱动或另一个 CLI |
| VRChat 认不到追踪器 | CLI 是否显示 `SolarXR runtime connected`。保持 CLI 运行，重新连接头显，再重启 VRChat |
| `Failed to connect to monado service process` | WiVRn 服务器没在运行（dashboard 开着不代表服务器开着） |
| 拖动游玩区后追踪器偏离身体 | CLI 是否显示 `libmonado unavailable`。确认 WiVRn 的 runtime JSON 里有 `MND_libmonado_path` |

日志里的 `Wine pipe`、`bridge.exe` 等字样沿用自原版，连接官方 Linux 版时也会出现，可以忽略。

## 开发

```sh
cargo build --release
cargo test
```

`monado_wire_compatibility` 测试需要 WiVRn 的 Monado 源代码，默认跳过：`MONADO_SOURCE=<路径> cargo test -- --ignored`。

参考：[WiVRn 与 OpenVR](https://github.com/WiVRn/WiVRn/blob/master/docs/steamvr.md) · [Rebocap 通信协议](https://github.com/dogesoulseller/rebocap-linux-compat/blob/main/docs/protocol.md)

## 外部组件的许可

Wine 桥接 `bridge.exe` 使用了 [rebocap-linux-compat](https://github.com/dogesoulseller/rebocap-linux-compat) 的代码。Copyright (c) 2026 Marcin Czerwonka，MIT License。全文见 [LICENSES/rebocap-linux-compat.txt](LICENSES/rebocap-linux-compat.txt)。

本 CLI 的原作者是 [akazdayo](https://github.com/akazdayo)。原仓库目前没有为自己的代码附上许可条款。
