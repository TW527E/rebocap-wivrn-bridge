[简体中文](README.md) · [繁體中文](README.zh-TW.md) · [English](README.en.md) · **日本語**

# monad2steamvr

Linux の WiVRn + xrizer 環境で、Rebocap を VRChat のフルボディトラッキングに使うための CLI です。VRChat OSC は使わず、通常のトラッカーとして姿勢を渡します。**SteamVR は不要です。**

このリポジトリは [akazdayo/rebocap-wivrn-bridge](https://github.com/akazdayo/rebocap-wivrn-bridge) のフォークで、次の変更を加えています。

- **公式 Linux 版 Rebocap に対応**：公式 Linux 版は自分で `127.0.0.1:20480` を待ち受けるので、`--bridge 127.0.0.1:20480` で直接つなげます。Wine も `bridge.exe` も不要です。
- **遅延の削減**：元の実装では、トラッカーの姿勢が 10 ms 間隔のポーリングループを 2 つ通っていました（平均で約 10 ms、最大で約 20 ms の遅れ）。受け取ったらすぐ WiVRn に送るようにしました。
- **プレイエリア移動の補正**：libmonado で WiVRn の STAGE オフセットを読み取ります。WayVR などでプレイエリアを動かしても、トラッカーが体からずれません。

> [!TIP]
> インストール手順の全体と起動スクリプト `rebocap-fbt`（公式版／Wine の自動判別、部位チェック、USB 接続）は
> [TW527E/rebocap-linux-wivrn-fbt](https://github.com/TW527E/rebocap-linux-wivrn-fbt) にあります。

```text
公式 Linux 版：
WiVRn の頭部姿勢 → この CLI → Rebocap（:20480）
Rebocap のトラッカー姿勢 → この CLI → SolarXR → WiVRn → xrizer → VRChat

Wine 版：
WiVRn の頭部姿勢 → この CLI → Wine ブリッジ bridge.exe（:36850）→ Rebocap
Rebocap のトラッカー姿勢 → bridge.exe → この CLI → SolarXR → WiVRn → xrizer → VRChat
```

## 必要なもの

- x86_64 Linux
- WiVRn + xrizer と接続できるヘッドセット
- Rebocap のレシーバーとセンサー、および次のどちらか
  - **公式 Linux 版 Rebocap**（おすすめ）
  - Wine 上の Windows 版 Rebocap と、[rebocap-linux-compat](https://github.com/TW527E/rebocap-linux-compat) の `bridge.exe`
    （元の作者は [rebocap-linux](https://github.com/akazdayo/rebocap-linux) で Wine 環境を作っています。Bottles でも動きます。[rebocap-wine](https://github.com/TW527E/rebocap-wine) を参照）
- ビルドには Rust と OpenXR loader の開発用ライブラリが必要です（Arch：`sudo pacman -S rust openxr pkgconf`）

WiVRn 26.9 + xrizer 0.5／0.6 で動作確認済みです。WiVRn の SolarXR と xrizer の `monado` 機能を使用します（この組み合わせでは既定で有効）。

## ビルド

```sh
git clone https://github.com/TW527E/rebocap-wivrn-bridge
cd rebocap-wivrn-bridge
cargo build --release      # 成果物：target/release/monad2steamvr
```

## 使い方

WiVRn サーバーを起動し、**ヘッドセットはまだ接続せずに**、次の順で進めます。

### 1. Rebocap を起動する

- **公式 Linux 版**：Rebocap を開くだけです。
- **Wine 版**：Rebocap を起動したあと、**同じ Wine prefix** で `bridge.exe` を起動し、その端末は開いたままにします。

  ```sh
  WINEPREFIX="${XDG_DATA_HOME:-$HOME/.local/share}/rebocap/wineprefix" wine ./bridge.exe
  ```

### 2. 別の端末で CLI を起動する

```sh
# 公式 Linux 版
./target/release/monad2steamvr --solarxr --bridge 127.0.0.1:20480

# Wine 版（--bridge の既定値が 127.0.0.1:36850）
./target/release/monad2steamvr --solarxr
```

OpenXR の既定ランタイムが設定されていない（`active_runtime.json` がない）場合は、先頭に
`XR_RUNTIME_JSON=/usr/share/openxr/1/openxr_wivrn.json` を付けてください。

`SolarXR ready` が表示されたら、ヘッドセットを接続・装着します。

### 3. キャリブレーションする

Rebocap を VR モードにしてキャリブレーションし、xrizer 経由で VRChat を起動して FBT キャリブレーションを行います。既定では **腰・左右足の 3 点**だけを公開します。変更は `--tracker-roles` で行います。

**CLI を再起動したときや公開部位を変更したときは、ヘッドセットを再接続して VRChat も再起動してください。** WiVRn はヘッドセット接続時にトラッカー一覧を取得します。

## オプション

| オプション | 用途 | 既定値 |
|---|---|---|
| `--solarxr` | WiVRn にトラッカーを公開 | 無効 |
| `--tracker-roles` | 公開する部位をカンマ区切りで指定 | `0,5,6` |
| `--bridge` | Rebocap の接続先。公式 Linux 版は `127.0.0.1:20480` | `127.0.0.1:36850`（`bridge.exe`） |
| `--solarxr-offset X,Y,Z` | 出力位置の補正（メートル、回転後に加算） | `0,0,0` |
| `--solarxr-yaw DEGREES` | 出力方向の補正（+Y 軸回りの度数） | `0` |

部位番号は `0`: 腰、`1/2`: 左右上腿、`3/4`: 左右下腿、`5/6`: 左右足、`7`: 胸、`9/10`: 左右上腕、`11/12`: 左右前腕です。

全対応部位を公開する例:

```sh
./target/release/monad2steamvr --solarxr --tracker-roles 0,1,2,3,4,5,6,7,9,10,11,12
```

部位は実際に装着しているトラッカーと合わせてください。6 点セットには足の実機トラッカーがなく、`5/6` は Rebocap が推定した仮想ノードなのでデータが途切れがちで、VRChat が足を認識できません。その場合は下腿 `3/4` を公開すれば、VRChat が位置から足として扱います。

姿勢は JSON Lines として標準出力にも流れます。位置はメートル単位、+Y が上、-Z が前、回転は `[w,x,y,z]` です。接続ログは標準エラーに出ます。

## うまく動かないとき

| 症状 | 確認すること |
|---|---|
| Rebocap が VR ヘッドセットを認識しない | Wine 版：ブリッジに `pipe open, relaying` が出ているか。CLI に `WiVRn headset tracking valid: true` が出ているか。Wine prefix が同じかも確認 |
| Rebocap の姿勢が届かない | VR モードでキャリブレーション済みか。CLI に `Received a Rebocap protocol message` が出ているか。公式 Linux 版はファームウェア更新中もデータを送りません |
| 公式 Linux 版につながらない | `20480` は同時に 1 つの接続しか受け付けません。公式の SteamVR ドライバーや別の CLI が接続していないか確認 |
| VRChat がトラッカーを認識しない | CLI に `SolarXR runtime connected` が出ているか。CLI を動かしたままヘッドセットを再接続し、VRChat を再起動 |
| `Failed to connect to monado service process` | WiVRn サーバーが動いていない（ダッシュボードを開いただけではサーバーは起動しません） |
| プレイエリアを動かすとトラッカーが体からずれる | CLI に `libmonado unavailable` が出ていないか。WiVRn のランタイム JSON に `MND_libmonado_path` があるか確認 |

ログ中の `Wine pipe` や `bridge.exe` といった表現は元の実装のままで、公式 Linux 版に接続しているときも表示されます。気にしなくて大丈夫です。

## 開発

```sh
cargo build --release
cargo test
```

`monado_wire_compatibility` テストは WiVRn の Monado ソースが必要なので、既定ではスキップされます：`MONADO_SOURCE=<パス> cargo test -- --ignored`

参考: [WiVRn と OpenVR](https://github.com/WiVRn/WiVRn/blob/master/docs/steamvr.md) · [Rebocap 通信仕様](https://github.com/dogesoulseller/rebocap-linux-compat/blob/main/docs/protocol.md)

## 外部コンポーネントのライセンス

Wine ブリッジ `bridge.exe` は [rebocap-linux-compat](https://github.com/dogesoulseller/rebocap-linux-compat) のコードを使用しています。Copyright (c) 2026 Marcin Czerwonka、MIT License。全文は [LICENSES/rebocap-linux-compat.txt](LICENSES/rebocap-linux-compat.txt) を参照してください。

この CLI の元の作者は [akazdayo](https://github.com/akazdayo) です。元のリポジトリには、作者自身のコードに対するライセンスがまだ付いていません。
