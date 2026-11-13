[简体中文](README.md) · **繁體中文** · [English](README.en.md) · [日本語](README.ja.md)

# monad2steamvr

在 Linux 的 WiVRn + xrizer 環境裡，把 Rebocap 當成 VRChat 全身追蹤用的 CLI。不使用 VRChat OSC，而是以一般追蹤器的形式傳送姿態，**不需要 SteamVR**。

本 repo fork 自 [akazdayo/rebocap-wivrn-bridge](https://github.com/akazdayo/rebocap-wivrn-bridge)，在原版之外做了這些修改：

- **支援官方 Linux 版 Rebocap**：官方 Linux 版自己會在 `127.0.0.1:20480` 監聽，用 `--bridge 127.0.0.1:20480` 直接連就好，不需要 Wine 和 `bridge.exe`。
- **更低的延遲**：原版裡追蹤器姿態要在兩個 10 ms 的輪詢迴圈裡排隊（平均多約 10 ms、最多約 20 ms），現在一收到就馬上送給 WiVRn。
- **遊玩區移動修正**：透過 libmonado 讀取 WiVRn 的 STAGE 偏移。用 WayVR 等工具拖動遊玩區後，追蹤器不會再偏離身體。

> [!TIP]
> 完整的安裝教學和啟動腳本 `rebocap-fbt`（自動偵測官方版／Wine、部位檢查、USB 連線）在
> [TW527E/rebocap-linux-wivrn-fbt](https://github.com/TW527E/rebocap-linux-wivrn-fbt)。

```text
官方 Linux 版：
WiVRn 的頭部姿態 → 本 CLI → Rebocap（:20480）
Rebocap 的追蹤器姿態 → 本 CLI → SolarXR → WiVRn → xrizer → VRChat

Wine 版：
WiVRn 的頭部姿態 → 本 CLI → Wine 橋接 bridge.exe（:36850）→ Rebocap
Rebocap 的追蹤器姿態 → bridge.exe → 本 CLI → SolarXR → WiVRn → xrizer → VRChat
```

## 需求

- x86_64 Linux
- WiVRn + xrizer，以及能連上的頭顯
- Rebocap 接收器和追蹤器，加上下列其中一種：
  - **官方 Linux 版 Rebocap**（推薦）
  - Wine 裡的 Windows 版 Rebocap，加上 [rebocap-linux-compat](https://github.com/TW527E/rebocap-linux-compat) 的 `bridge.exe`
    （原作者是用 [rebocap-linux](https://github.com/akazdayo/rebocap-linux) 架 Wine 環境；Bottles 也可以，參考 [rebocap-wine](https://github.com/TW527E/rebocap-wine)）
- 建置需要 Rust 和 OpenXR loader 的開發函式庫（Arch：`sudo pacman -S rust openxr pkgconf`）

在 WiVRn 26.9 + xrizer 0.5／0.6 上驗證過。會用到 WiVRn 的 SolarXR 和 xrizer 的 `monado` 功能（這個組合預設就有開）。

## 建置

```sh
git clone https://github.com/TW527E/rebocap-wivrn-bridge
cd rebocap-wivrn-bridge
cargo build --release      # 產物：target/release/monad2steamvr
```

## 使用方式

先啟動 WiVRn 伺服器，**頭顯先不要連**，再照下面的順序進行。

### 1. 啟動 Rebocap

- **官方 Linux 版**：直接打開 Rebocap 就好。
- **Wine 版**：啟動 Rebocap 之後，在**同一個 Wine prefix** 裡啟動 `bridge.exe`，並讓這個終端機保持開著：

  ```sh
  WINEPREFIX="${XDG_DATA_HOME:-$HOME/.local/share}/rebocap/wineprefix" wine ./bridge.exe
  ```

### 2. 在另一個終端機啟動 CLI

```sh
# 官方 Linux 版
./target/release/monad2steamvr --solarxr --bridge 127.0.0.1:20480

# Wine 版（--bridge 預設就是 127.0.0.1:36850）
./target/release/monad2steamvr --solarxr
```

系統沒有設定 OpenXR 預設 runtime（沒有 `active_runtime.json`）時，前面加上
`XR_RUNTIME_JSON=/usr/share/openxr/1/openxr_wivrn.json`。

看到 `SolarXR ready` 之後，連上並戴上頭顯。

### 3. 校準

把 Rebocap 切到 VR 模式並校準，再透過 xrizer 開 VRChat，做 FBT 校準。預設只發布**腰和左右腳 3 個點**，用 `--tracker-roles` 修改。

**重啟 CLI 或改了發布的部位之後，頭顯要重新連線，VRChat 也要重開。** WiVRn 只在頭顯連線時讀取追蹤器清單。

## 選項

| 選項 | 用途 | 預設值 |
|---|---|---|
| `--solarxr` | 把追蹤器發布給 WiVRn | 關閉 |
| `--tracker-roles` | 要發布的部位，用逗號分隔 | `0,5,6` |
| `--bridge` | Rebocap 的連線位址。官方 Linux 版用 `127.0.0.1:20480` | `127.0.0.1:36850`（`bridge.exe`） |
| `--solarxr-offset X,Y,Z` | 輸出位置修正（公尺，在旋轉之後加上） | `0,0,0` |
| `--solarxr-yaw DEGREES` | 輸出方向修正（繞 +Y 軸的角度） | `0` |

部位編號：`0` 腰、`1/2` 左右大腿、`3/4` 左右小腿、`5/6` 左右腳、`7` 胸、`9/10` 左右上臂、`11/12` 左右前臂。

發布所有支援部位的例子：

```sh
./target/release/monad2steamvr --solarxr --tracker-roles 0,1,2,3,4,5,6,7,9,10,11,12
```

部位要和實際綁的追蹤器一致。6 點套組沒有腳的實體追蹤器，`5/6` 只是 Rebocap 推算出來的虛擬節點，資料斷斷續續，VRChat 會抓不到腳；這種情況把小腿 `3/4` 發布出去，VRChat 會依位置把它當成腳。

姿態也會以 JSON Lines 輸出到標準輸出：位置單位是公尺，+Y 朝上，-Z 朝前，旋轉是 `[w,x,y,z]`。連線 log 輸出到標準錯誤。

## 疑難排解

| 症狀 | 檢查項目 |
|---|---|
| Rebocap 認不到 VR 頭顯 | Wine 版：bridge 有沒有顯示 `pipe open, relaying`；CLI 有沒有顯示 `WiVRn headset tracking valid: true`。也確認 Wine prefix 是同一個 |
| Rebocap 的姿態沒有傳過來 | 是否已切到 VR 模式並校準；CLI 有沒有顯示 `Received a Rebocap protocol message`。官方 Linux 版在更新韌體時也不會送資料 |
| 官方 Linux 版連不上 | `20480` 同一時間只收一個連線。確認沒有開著官方 SteamVR 驅動或另一個 CLI |
| VRChat 認不到追蹤器 | CLI 有沒有顯示 `SolarXR runtime connected`。讓 CLI 繼續跑，頭顯重新連線，再重開 VRChat |
| `Failed to connect to monado service process` | WiVRn 伺服器沒在跑（dashboard 開著不代表伺服器開著） |
| 拖動遊玩區後追蹤器偏離身體 | CLI 有沒有顯示 `libmonado unavailable`。確認 WiVRn 的 runtime JSON 裡有 `MND_libmonado_path` |

log 裡的 `Wine pipe`、`bridge.exe` 這些字是沿用原版的，連官方 Linux 版時也會出現，可以忽略。

## 開發

```sh
cargo build --release
cargo test
```

`monado_wire_compatibility` 測試需要 WiVRn 的 Monado 原始碼，預設會略過：`MONADO_SOURCE=<路徑> cargo test -- --ignored`。

參考：[WiVRn 與 OpenVR](https://github.com/WiVRn/WiVRn/blob/master/docs/steamvr.md) · [Rebocap 通訊協定](https://github.com/dogesoulseller/rebocap-linux-compat/blob/main/docs/protocol.md)

## 外部元件的授權

Wine 橋接 `bridge.exe` 使用了 [rebocap-linux-compat](https://github.com/dogesoulseller/rebocap-linux-compat) 的程式碼。Copyright (c) 2026 Marcin Czerwonka，MIT License。全文見 [LICENSES/rebocap-linux-compat.txt](LICENSES/rebocap-linux-compat.txt)。

這個 CLI 的原作者是 [akazdayo](https://github.com/akazdayo)。原 repo 目前還沒有為自己的程式碼附上授權條款。
