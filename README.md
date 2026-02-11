# Terbulator

超軽量なGUI端末エミュレータ（Rust 2024 Edition実装）

## ステータス

**Phase 2 - ペイン管理 - 完成 ✅** (2026-02-11)

複数ペインによる分割表示、キーボードショートカット、Broadcastモード、ヘルプ表示が実装されました。
軽量性を保つため、メニューバーの代わりにF1キーでヘルプを表示する方式を採用しました。

### パフォーマンス最適化

- ✅ イベント駆動型レンダリング（CPU使用率を大幅削減）
- ✅ PTY出力時のみ再描画（無駄な再描画を削減）
- ✅ カーソル点滅タイマー最適化（500ms間隔）
- ✅ 非ブロッキングPTY読み取り（64KB/フレーム制限）
- ✅ **グリフキャッシュ**（文字ごとのshape結果をキャッシュ、描画高速化）
- ✅ **ペインレベル差分レンダリング**（変更のあったペインのみ描画判定）

これらの最適化により、アイドル時のCPU使用率をほぼ0%に抑え、高速な応答性を実現しています。
グリフキャッシュにより、同じ文字の描画が大幅に高速化されています。
CPUレンダリングでも十分な性能を発揮し、VMWare等の仮想環境でも快適に動作します。

### 実装済み機能

#### Phase 1: 基本端末エミュレータ

- ✅ VT100端末エミュレーション（256色サポート）
- ✅ CPU描画バックエンド（VM環境対応）
- ✅ PTY管理（bash/zsh等のシェル起動）
- ✅ キーボード入力（全キー、Ctrl組み合わせ）
- ✅ テキストレンダリング（日本語表示可能）
- ✅ ウィンドウ管理とリサイズ
- ✅ 設定ファイル（YAML）

#### Phase 2: ペイン管理

- ✅ ペイン分割（水平・垂直、任意回数）
- ✅ ペイン間フォーカス移動（方向指定、Next/Prev）
- ✅ ペイン閉じる機能（自動レイアウト調整）
- ✅ マルチペインレンダリング（全ペイン表示）
- ✅ アクティブペイン境界線表示
- ✅ Broadcastモード（全ペイン同時入力）
- ✅ キーボードショートカット（Ctrl-Shift-h/j/k/l/n/p/s/v/w/b）

#### Phase 3: 高度な機能（進行中）

- ✅ マウスドラッグによるテキスト選択
- ✅ 選択範囲のハイライト表示
- ✅ Copy機能（Ctrl-Shift-C）
- ✅ Paste機能（Ctrl-V）
- ✅ **Kittyプロトコル画像表示** - base64エンコード画像に対応
- ✅ **sixelプロトコル画像表示** - 簡易実装（プレースホルダー）
- ✅ **マークモード（Alt-Shift-M）** - キーボードのみでテキスト選択可能
- ✅ **IME対応（Ctrl-Space）** - 日本語入力サポート（UTF-8）
- ⏳ 文字コード対応（UTF-8/EUC-JP/Shift-JIS）

### キーボードショートカット

#### ヘルプ
- **F1**: ショートカット一覧を表示/非表示
- **ESC**: ヘルプを閉じる

#### ペイン管理
- **Ctrl-Shift-s**: 水平分割
- **Ctrl-Shift-v**: 垂直分割
- **Ctrl-Shift-w**: ペインを閉じる

#### フォーカス移動
- **Ctrl-Shift-h**: 左のペインに移動
- **Ctrl-Shift-j**: 下のペインに移動
- **Ctrl-Shift-k**: 上のペインに移動
- **Ctrl-Shift-l**: 右のペインに移動
- **Ctrl-Shift-n**: 次のペインに移動
- **Ctrl-Shift-p**: 前のペインに移動

#### フォント
- **Ctrl-+**: フォントサイズを大きくする
- **Ctrl--**: フォントサイズを小さくする

#### クリップボード
- **マウスドラッグ**: テキスト選択（選択範囲がハイライト表示）
- **Ctrl-Shift-C**: 選択範囲をコピー
- **Ctrl-V**: クリップボードから貼り付け

#### マークモード（キーボードベース選択）
- **Alt-Shift-M**: マークモードを開始/終了（ウィンドウタイトルに"[MARK]"表示）
- **矢印キー（←↑↓→）**: カーソルを移動して選択範囲を拡張
- **Enter**: 選択範囲をコピーしてマークモードを終了
- **ESC**: 選択をキャンセルしてマークモードを終了

#### IME（日本語入力）
- **OSのデフォルトIMEショートカット**: IMEを有効/無効に切り替え（例: Ctrl-Space、半角/全角キーなど）
- IME有効時はウィンドウタイトルに"[あ]"表示
- 日本語入力が可能（UTF-8エンコーディング）

#### その他
- **Ctrl-Shift-b**: Broadcastモード切り替え（有効時はウィンドウタイトルに"Broadcasting"表示）
- **マウスクリック**: ペイン選択
- **マウスドラッグ（境界）**: ペイン境界をドラッグしてサイズ変更

### 動作確認済み

- `bash`, `ls`, `cat`, `vim`, `top`, `clear`
- 色付き出力（`ls --color`）
- 日本語テキスト表示
- 複数ペイン分割・統合
- ペイン間フォーカス移動

## Building

```bash
# Check code
cargo check

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run (debug)
cargo run

# Run (release)
cargo run --release
```

## Running

```bash
# Run with default config
./target/debug/terbulator

# Or with cargo
cargo run

# Run with custom config file
./target/debug/terbulator --config /path/to/config.yaml
cargo run -- --config /path/to/config.yaml

# Show help
./target/debug/terbulator --help
```

The application will:

1. Create a default config at `~/.config/terbulator/config.yaml` if it doesn't exist (when using default path)
2. Open a window with a terminal emulator
3. Spawn your configured shell (or default from `SHELL` environment variable, or `/bin/bash`)
4. When you split panes (Ctrl-Shift-s/v), new shells are automatically spawned in split panes
5. When you exit a shell, that pane automatically closes
6. When the last pane closes, the application exits

## Configuration

Default configuration file: `~/.config/terbulator/config.yaml`

Example configuration:

```yaml
renderer:
  backend: "auto"  # Options: "auto", "gpu", "cpu"
  target_fps: 60

terminal:
  cols: 80
  rows: 24
  font_size: 14.0
  font_family: "monospace"
  scrollback: 10000
  shell: "/bin/bash"  # Shell to execute (default: $SHELL or /bin/bash)

window:
  title: "terbulator"
  width: 800
  height: 600
  maximize: true  # Set to true to start maximized (default)

startup:
  panes: 4  # Number of panes: 1, 2, or 4
  layout: "grid"  # Layout: "single", "horizontal", "vertical", "grid"
  split_ratio: 0.7  # Horizontal split ratio for top:bottom (7:3)
  vertical_ratio: 0.5  # Vertical split ratio for left:right (5:5)
```

### Configuration Options

#### Renderer

- `backend`: Backend selection
  - `auto` (default): Try GPU first, fall back to CPU if initialization fails
  - `gpu`: Use GPU rendering (wgpu) - *Not yet fully implemented*
  - `cpu`: Use CPU rendering (softbuffer) - Stable, works in VM environments
- `target_fps`: Target frame rate (default: 60)

#### Terminal

- `cols`: Number of columns (default: 80)
- `rows`: Number of rows (default: 24)
- `font_size`: Font size in pixels (default: 14.0)
- `font_family`: Font family name (default: "monospace")
- `scrollback`: Scrollback buffer size (default: 10000)
- `shell`: Path to shell executable (default: `$SHELL` environment variable or `/bin/bash`)
  - Examples: `/bin/bash`, `/bin/zsh`, `/usr/bin/fish`
  - Used when spawning initial pane and split panes

#### Window

- `title`: Window title (default: "terbulator")
- `width`: Initial window width in pixels (default: 800)
- `height`: Initial window height in pixels (default: 600)
- `maximize`: Start with maximized window (default: true)

#### Startup

- `panes`: Number of panes to create on startup (default: 4)
  - Supported values: 1, 2, 4
- `layout`: Layout type (default: "grid")
  - `single`: Single pane (for panes=1)
  - `horizontal`: Horizontal split (for panes=2)
  - `vertical`: Vertical split (for panes=2)
  - `grid`: 2x2 grid (for panes=4)
- `split_ratio`: Horizontal split ratio for top:bottom (default: 0.7 for 7:3 ratio)
- `vertical_ratio`: Vertical split ratio for left:right (default: 0.5 for 5:5 ratio)

## テスト結果

### Phase 1: 基本端末エミュレータ

- [x] ウィンドウが正しく表示される
- [x] CPUバックエンドが動作する
- [x] テキストが正しくレンダリングされる
- [x] bashが正常に起動する
- [x] 基本コマンドが動作する（ls, cat, vim, top）
- [x] 色付き出力が動作する（ls --color）
- [x] 日本語テキストが表示される
- [x] CPU描画で60fps維持
- [x] VMWare環境で動作確認
- [x] ウィンドウクローズが正常に動作する

### Phase 2: ペイン管理

- [x] ペイン水平分割（Ctrl-Shift-s）
- [x] ペイン垂直分割（Ctrl-Shift-v）
- [x] ペイン閉じる（Ctrl-Shift-w）
- [x] フォーカス移動（h/j/k/l/n/p）
- [x] マルチペインレンダリング
- [x] Broadcastモード（Ctrl-Shift-b）
- [x] アクティブペイン境界表示
- [x] マウスによるペイン選択
- [x] マウスドラッグによるペインサイズ変更
- [x] ヘルプ表示（F1キー）
- [x] 非ブロッキングPTY読み取り（tail -F中も別ペイン入力可能）
- [x] resetコマンド対応
- [x] 起動時のペイン数とレイアウトを設定可能
- [x] 各ペインの比率を設定可能（水平・垂直）
- [x] Broadcastモード状態表示（ウィンドウタイトル）
- [x] 動的フォントサイズ変更（Ctrl +/-）

### Phase 3: 高度な機能

- [x] マウスドラッグによるテキスト選択
- [x] 選択範囲のハイライト表示
- [x] Copy機能（Ctrl-Shift-C）
- [x] Paste機能（Ctrl-V）
- [x] マークモード（Alt-Shift-M）
  - [x] マークモード開始（カーソル位置から選択開始）
  - [x] 矢印キーで選択範囲拡張
  - [x] Enterキーでコピー＆終了
  - [x] ESCキーでキャンセル＆終了
  - [x] ウィンドウタイトルに"[MARK]"表示
- [x] Kittyプロトコル画像表示
- [x] sixelプロトコル画像表示（簡易実装）
- [x] IME（日本語入力）
  - [x] OSのデフォルトIMEショートカットで切り替え
  - [x] IME Enabled/Disabledイベント処理
  - [x] IME確定文字列をPTYに送信
  - [x] ウィンドウタイトルに"[あ]"表示
  - [x] UTF-8エンコーディング対応
- [ ] 文字コード自動検出（UTF-8/EUC-JP/Shift-JIS）

## 既知の制限事項

- GPUバックエンド（wgpu）は未実装 - CPUバックエンド推奨
- メニューバーなし（軽量性のため、ショートカット + F1ヘルプで代替）
- 文字コード自動検出は未実装（UTF-8固定）
- sixel画像表示は簡易実装（プレースホルダー）

## 次のステップ

### Phase 3（残りの機能） - 予定

- 文字コード自動検出（UTF-8/EUC-JP/Shift-JIS）
- sixelプロトコル完全実装

### Phase 4（最適化） - 予定

- デフォルト4ペインレイアウト
- 設定管理完全対応
- パフォーマンス最適化

## Project Structure

```
src/
├── main.rs              # Event loop and application entry
├── app.rs               # Central state management
├── config/              # Configuration (YAML)
│   ├── types.rs
│   └── loader.rs
├── terminal/            # VT100 emulation and PTY
│   ├── grid.rs
│   ├── emulator.rs
│   └── pty.rs
├── renderer/            # Rendering backends
│   ├── backend.rs
│   ├── wgpu_backend.rs
│   └── softbuffer_backend.rs
├── input/               # Input handling
│   └── keyboard.rs
└── utils/               # Error types
    └── error.rs
```

## License

TBD

## Contributing

TBD
