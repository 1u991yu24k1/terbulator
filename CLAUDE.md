# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`terbulator`はRust 2024 Editionで実装された, 超軽量なタブ表示可能なGUI端末エミュレータです。

## Current Status (Phase 2 - COMPLETED ✅)

**Phase 1 MVP - 基本端末エミュレータ（完了）:**

- ✅ 基本的なVT100端末エミュレーション（vteパーサー使用）
- ✅ PTY管理（portable-pty使用）
- ✅ CPU描画バックエンド（softbuffer使用、VMWare対応）
- ✅ テキストレンダリング（cosmic-text使用、日本語表示可能）
- ✅ キーボード入力処理（全キー、Ctrl組み合わせ対応）
- ✅ 256色サポート
- ✅ ウィンドウ管理とリサイズ
- ✅ カーソル表示（アンダーライン型）
- ✅ 設定ファイル（YAML形式、`~/.config/terbulator/config.yaml`）
- ✅ ウィンドウクローズ処理

**Phase 2 - ペイン管理とGUI機能（完了 2026-02-11）:**

- ✅ マルチペインサポート（水平・垂直分割、任意回数）
- ✅ フォーカス移動（Ctrl-Shift-h/j/k/l/n/p）
- ✅ マウスによるペイン選択
- ✅ マウスドラッグによるペインサイズ変更（境界線ドラッグ）
- ✅ アクティブペイン境界線表示
- ✅ Broadcastモード（Ctrl-Shift-b、全ペイン同時入力）
- ✅ ウィンドウタイトルにBroadcast状態表示
- ✅ ヘルプオーバーレイ（F1キー）
- ✅ 動的フォントサイズ変更（Ctrl +/-、8-32pt）
- ✅ 起動時ペイン設定（数、レイアウト、比率）
- ✅ ペイン終了時の自動統合
- ✅ イベント駆動型レンダリング（アイドル時CPU ~0%）
- ✅ 非ブロッキングPTY読み取り（64KB/フレーム）

**動作確認済みコマンド：**

- `bash`, `ls`, `cat`, `vim`, `top`, `clear`, `echo`, `tail -f`
- 色付き出力（`ls --color`）
- 複数ペイン同時操作

**次のフェーズ：**

- **Phase 3（次の目標）:** クリップボード（Copy/Paste）、文字コード対応、マークモード、画像表示
- Phase 4: 設定管理完全対応、最適化

### Functional Features (Target)

- X11/Waylandの両方をサポート.
- VT100/256 Colorをサポート.
- クリップボードへのアクセスをサポート.
  - Copy/Paste可能
- キーボードだけのカーソルのドラッグ&ドロップ機能をサポート.
  - Alt-Shift-mキーをショートカットとして, Windows コマンドプロンプトのマーク機能相当を持つ.
- 日本語入力サポート
  - UTF-8/EUC-JP/Shift-JISをサポート.
- 外部fontのインポート・インストールをサポート
- ペイン分割によるタブ表示をサポート.
　* ウィンドウの任意回数の水平分割, 垂直分割をサポート
  - Copy/Pasteする際に, tmuxのようにペインをまたいでしまう問題を解決.
  - GUIのメニューバーでウィンドウ内のペイン分割・統合を制御
    - メニューバーはグローバルに1つWindow丈夫に表示される.
    - 各ペインはID(番号)で管理される.
  - ショートカットで各ペイン間の移動を制御.
    - 左のペインへの移動: Ctrl-Shift-h
    - 下のペインへの移動: Ctrl-Shift-j
    - 上のペインへの移動: Ctrl-Shift-k
    - 右のペインへの移動: Ctrl-Shift-l
    - 次のペインへの移動: Ctrl-Shift-n
    - 前のペインへの移動: Ctrl-Shift-p
- 各ペインに同時にキー入力可能なBroadcastモードをサポート
  - BoardcastのOn/OffはGUIのメニューバーにボタンを設置
- 端末上に画像表示できる機能を各種サポート. (`img2sixel`などの画像表示を端末上でできるように.)
  - sixelプロトコルをサポート.
  - Kittyプロトコルをサポート
- ペインへの終了時(bashの`exit`など), そのペインを閉じて他のペイン間のレイアウトを適切に統合.
- 各種Config(キーボード・ショートカット, デフォルトのレイアウト)をyamlで管理できるように
  - デフォルトconfigファイルを`~/.config/terbulator/config.yaml`とする.
  - デフォルトは4ペイン構成とし, 垂直分割, 水平分割で縦2, 横2構成.
  - 水平分割時の上2ペインと下2ペインの高さの比は7:3とする.

- 動的なフォントサイズの変更をサポート.
  - カーソルのあっているペインでCtrl + '+'の組み合わせでフォントサイズを1上げる.
  - カーソルのあっているペインでCtrl + '-'の組み合わせでフォントサイズを1下げる.

### Implementation Guidelines

Rust 2024 Editionを使用する.

- unsafeブロックはなるべく使わない. やむなく気をつける場合は, メモリ破壊に気をつけること.
- Rustの参照を使用する場合, unsoundなissueに該当するような実装をしないよう気をつけること.
  - 参考: <https://github.com/rust-lang/rust/issues/57893>
- サードパーティクレートを入れる場合は、`Cargo.toml`にバージョンを明記すること.
- セキュリティチェックによるフィードバックを行い, OSコマンドインジェクションなどに気をつけること.

## Build and Run Commands

**Check before build project(fast check):**

```bash
cargo check
```

**Build the project:**

```bash
cargo build
```

**Run the application:**

```bash
cargo run
```

**Build optimized release version:**

```bash
cargo build --release
```

**Run release version:**

```bash
cargo run --release
```

## Testing

**Run all tests:**

```bash
cargo test
```

**Run a specific test:**

```bash
cargo test test_name
```

**Run tests with output:**

```bash
cargo test -- --nocapture
```

## Code Quality

**Check code without building:**

```bash
cargo check
```

**Format code:**

```bash
cargo fmt
```

**Run linter:**

```bash
cargo clippy
```

**Run linter with all warnings:**

```bash
cargo clippy -- -W clippy::all
```

## Project Structure

```
src/
├── main.rs              # イベントループ、アプリケーションエントリポイント
├── app.rs               # 中央状態管理
├── config/              # 設定管理（YAML）
│   ├── mod.rs
│   ├── types.rs         # Config構造体定義
│   └── loader.rs        # YAML読み込み
├── pane/                # ペイン管理（Phase 2で追加）
│   ├── mod.rs
│   ├── pane.rs          # Pane構造体
│   ├── manager.rs       # PaneManager（分割、統合、フォーカス管理）
│   └── layout.rs        # Layout（ツリー構造、矩形計算）
├── terminal/            # VT100エミュレーションとPTY
│   ├── mod.rs
│   ├── grid.rs          # セルグリッド（ダーティトラッキング付き）
│   ├── emulator.rs      # VT100パーサー（vte::Perform実装）
│   └── pty.rs           # PTYコントローラー（スレッド化読み取り）
├── renderer/            # 描画パイプライン
│   ├── mod.rs
│   ├── backend.rs       # RenderBackendトレイト定義
│   ├── wgpu_backend.rs  # GPU描画（未完成）
│   └── softbuffer_backend.rs  # CPU描画（現在使用中）
├── input/               # 入力処理
│   ├── mod.rs
│   ├── keyboard.rs      # キーボードハンドラー
│   └── shortcuts.rs     # ショートカットマッチング（Phase 2で追加）
└── utils/               # エラー型
    ├── mod.rs
    └── error.rs         # TerbulatorError定義

config.yaml              # ユーザー設定
config.yaml.example      # 設定例
README.md                # プロジェクト説明
Cargo.toml               # 依存関係
```

### Key Dependencies

- `winit` 0.30 - ウィンドウ管理（X11/Wayland対応）
- `softbuffer` 0.4 - CPU描画バックエンド
- `cosmic-text` 0.11 - テキストレンダリング
- `vte` 0.13 - VT100パーサー
- `portable-pty` 0.8 - PTY管理
- `serde` + `serde_yaml` - 設定ファイル
- `tokio` 1.0 - 非同期ランタイム（将来の機能用）

## Configuration

デフォルト設定ファイル: `~/.config/terbulator/config.yaml`

```yaml
renderer:
  backend: "auto"  # "auto", "gpu", "cpu"（現在はcpuのみ動作）
  target_fps: 60

terminal:
  cols: 80
  rows: 24
  font_size: 14.0
  font_family: "monospace"
  scrollback: 10000
  shell: "/bin/bash"  # シェルのパス

window:
  title: "terbulator"
  width: 800
  height: 600
  maximize: true  # 起動時に最大化（Phase 2で追加）

startup:  # Phase 2で追加
  panes: 4  # 起動時のペイン数（1, 2, 4）
  layout: "grid"  # レイアウト（"single", "horizontal", "vertical", "grid"）
  split_ratio: 0.7  # 水平分割の比率（上:下 = 7:3）
  vertical_ratio: 0.5  # 垂直分割の比率（左:右 = 5:5）
```

## Known Issues

- GPU描画バックエンド（wgpu）は未実装
- クリップボード機能は未実装（Phase 3予定）
- 文字コード自動検出は未実装（Phase 3予定）
- マークモード（Alt-Shift-m）は未実装（Phase 3予定）
- 画像表示（sixel/Kitty）は未実装（Phase 3予定）
- メニューバーなし（軽量化のため、ショートカット + F1ヘルプで代替）

## Implementation Notes

### レンダリング

- CPU描画（softbuffer）を使用
- セル幅: `font_size * 0.6`
- セル高さ: `font_size * 1.3`
- ベースラインオフセット: `font_size * 1.1`
- カーソル位置: セル高さの80%
- **イベント駆動型レンダリング**: `ControlFlow::WaitUntil`でアイドル時CPU ~0%
- マルチペインレンダリング: バッファ全体クリア後、各ペインを順次描画

### ペイン管理（Phase 2）

- **レイアウト**: ツリー構造（`LayoutNode::Split` / `LayoutNode::Leaf`）
- **分割**: 水平/垂直分割、カスタム比率対応
- **フォーカス**: 方向ベース（上下左右）+ 順次（Next/Prev）
- **境界ドラッグ**: マウス位置から最寄りの境界を検出（5px許容範囲）
- **統合**: ペイン終了時、兄弟ノードが親領域を占有
- **Broadcast**: 有効時は全ペインの`pty.write()`に同じ入力を送信

### PTY読み取り

- 別スレッドでブロッキング読み取り
- チャンネル経由でメインスレッドにデータ送信
- WouldBlockエラーで適切にハンドリング
- **非ブロッキング**: 1フレームあたり最大64KB読み取り制限

### VT100サポート

- 基本的なカーソル移動（上下左右、Home/End）
- 画面クリア（J, K）
- SGR（色、太字、イタリック、アンダーライン、反転）
- 256色パレット
- カーソル保存/復元
- `reset`コマンド対応

### キーボードショートカット（Phase 2）

- **ペイン分割**: Ctrl-Shift-s（水平）、Ctrl-Shift-v（垂直）
- **ペイン閉じる**: Ctrl-Shift-w
- **フォーカス移動**: Ctrl-Shift-h/j/k/l（方向）、Ctrl-Shift-n/p（順次）
- **Broadcast**: Ctrl-Shift-b（トグル）
- **フォント**: Ctrl-+（拡大）、Ctrl--（縮小）
- **ヘルプ**: F1（表示/非表示）、ESC（閉じる）

## Notes

- This project uses Rust edition 2024, which may include newer language features and improved diagnostics.
- **Phase 1 MVP completed**: 基本的な端末エミュレータとして動作可能
- **Phase 2 completed**: マルチペイン、フォーカス管理、Broadcast、動的フォントサイズ
- VM環境（VMWare等）でのCPU描画動作確認済み
- イベント駆動型レンダリングによりアイドル時のCPU使用率は ~0%
- **軽量性の目標**: メニューバー等の巨大ライブラリへの依存を避け、ショートカット + ヘルプオーバーレイで代替
