# TmuxCC

AI Agent Dashboard for tmux - 複数のAIエージェントを一元管理するターミナルダッシュボード

## 概要

TmuxCCは、tmux上で実行されているAIエージェント（Claude Code, OpenCode, Codex CLI, Gemini CLI）を監視・操作するためのTUIアプリケーションです。

### 主な機能

- **マルチエージェント監視**: 複数のAIエージェントの状態をリアルタイムで表示
- **階層表示**: Session/Window/Agent の階層構造でツリー表示
- **承認操作**: ダッシュボードから直接承認（Y）/却下（N）を送信
- **入力サポート**: 数字キーによる選択肢の回答、フリーテキスト入力
- **複数選択**: 複数エージェントへの一括操作
- **Subagent表示**: Claude Codeのサブエージェント状態も表示

## スクリーンショット

```
┌──────────────────────────────────────────────────────────────────┐
│  TmuxCC - AI Agent Dashboard                   Agents: 3 Active: 1│
├──────────────────────────────────────────────────────────────────┤
│ main (Session)                    │ Preview: main:0.0             │
│ ├─ 0: code                        │                               │
│ │  ├─ ~/project1                  │ Claude Code wants to edit:    │
│ │  │  ● Claude Code  ⚠ [Edit]    │ src/main.rs                   │
│ │  │     └─ ▶ Explore (Running)   │                               │
│ │  └─ ~/project2                  │ - fn main() {                 │
│ │     ○ OpenCode   ◐ Processing   │ + fn main() -> Result<()> {   │
│ └─ 1: shell                       │                               │
│    └─ ~/tools                     │ Do you want to allow this     │
│       ○ Codex CLI  ● Idle         │ edit? [y/n]                   │
├──────────────────────────────────────────────────────────────────┤
│ [Y] Approve [N] Reject [A] All │ [1-9] Choice [I] Input │ [Space] │
└──────────────────────────────────────────────────────────────────┘
```

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/yourusername/tmuxcc.git
cd tmuxcc

# ビルド
cargo build --release

# インストール（オプション）
cargo install --path .
```

## 使い方

```bash
# tmuxセッション内で実行
tmuxcc

# ヘルプを表示
tmuxcc --help
```

### コマンドラインオプション

| オプション | 短縮 | 説明 | デフォルト |
|------------|------|------|------------|
| `--poll-interval <MS>` | `-p` | ポーリング間隔（ミリ秒） | 500 |
| `--capture-lines <LINES>` | `-c` | ペインからキャプチャする行数 | 100 |
| `--config <FILE>` | `-f` | 設定ファイルのパス | - |
| `--debug` | `-d` | デバッグログを `tmuxcc.log` に出力 | - |
| `--show-config-path` | - | 設定ファイルのパスを表示 | - |
| `--init-config` | - | デフォルト設定ファイルを生成 | - |

### 使用例

```bash
# ポーリング間隔を1秒に設定
tmuxcc -p 1000

# キャプチャ行数を増やす（より多くのコンテキストを取得）
tmuxcc -c 200

# カスタム設定ファイルを使用
tmuxcc -f ~/.config/tmuxcc/custom.toml

# デバッグモードで実行
tmuxcc --debug

# 設定ファイルを初期化
tmuxcc --init-config
```

## キーバインド

### ナビゲーション

| キー | 説明 |
|------|------|
| `j` / `↓` | 次のエージェント |
| `k` / `↑` | 前のエージェント |
| `Tab` | 次のエージェント（循環） |

### 選択

| キー | 説明 |
|------|------|
| `Space` | 現在のエージェントの選択をトグル |
| `Ctrl+a` | 全エージェントを選択 |
| `Esc` | 選択をクリア / サブエージェントログを閉じる |

### アクション

| キー | 説明 |
|------|------|
| `y` / `Y` | 承認（選択されたエージェントまたは現在のエージェント） |
| `n` / `N` | 却下（選択されたエージェントまたは現在のエージェント） |
| `a` / `A` | 全ての承認待ちを承認 |
| `1`-`9` | 数字の選択肢を送信（AskUserQuestion用） |
| `i` / `I` | 入力モードに入る（フリーテキスト入力） |
| `Enter` | 選択したペインにtmuxでフォーカス |

### 入力モード

| キー | 説明 |
|------|------|
| 任意の文字 | 入力バッファに追加 |
| `Backspace` | 最後の文字を削除 |
| `Enter` | 入力を送信 |
| `Esc` | 入力モードをキャンセル |

### 表示

| キー | 説明 |
|------|------|
| `s` / `S` | サブエージェントログの表示をトグル |
| `r` | リフレッシュ / エラーをクリア |
| `h` / `?` | ヘルプを表示 |
| `q` | 終了 |

## サポートしているエージェント

| エージェント | 検出方法 |
|--------------|----------|
| Claude Code | コマンド名 `claude` またはウィンドウタイトル（✳アイコン）|
| OpenCode | コマンド名 `opencode` |
| Codex CLI | コマンド名 `codex` |
| Gemini CLI | コマンド名 `gemini` |

## ステータス表示

| アイコン | 状態 |
|----------|------|
| ⚠ `[Edit]` | ファイル編集の承認待ち |
| ⚠ `[Bash]` | シェルコマンドの承認待ち |
| ⚠ `[?]` | ユーザー質問への回答待ち |
| ◐ | 処理中 |
| ● | アイドル状態 |
| ○ | 不明 |

## 設定ファイル

設定ファイルは `~/.config/tmuxcc/config.toml` に保存されます。

```bash
# 設定ファイルを初期化
tmuxcc --init-config

# 設定ファイルのパスを確認
tmuxcc --show-config-path
```

### 設定例

```toml
# ポーリング間隔（ミリ秒）
poll_interval_ms = 500

# キャプチャする行数
capture_lines = 100

# カスタムエージェントパターン（オプション）
[[agent_patterns]]
pattern = "my-custom-agent"
agent_type = "custom"
```

## 技術スタック

- **言語**: Rust (Edition 2021)
- **TUI**: [Ratatui](https://github.com/ratatui/ratatui) 0.29
- **ターミナル**: [Crossterm](https://github.com/crossterm-rs/crossterm) 0.28
- **非同期**: [Tokio](https://tokio.rs/)
- **CLI**: [Clap](https://github.com/clap-rs/clap) 4

## 開発

```bash
# ビルド
cargo build

# テスト実行
cargo test

# Lint
cargo clippy

# フォーマット
cargo fmt
```

## プロジェクト構造

```
tmuxcc/
├── src/
│   ├── main.rs          # エントリーポイント
│   ├── lib.rs           # ライブラリルート
│   ├── agents/          # エージェント型定義
│   │   ├── types.rs     # AgentType, AgentStatus, MonitoredAgent
│   │   └── subagent.rs  # Subagent, SubagentType, SubagentStatus
│   ├── app/             # アプリケーションロジック
│   │   ├── state.rs     # AppState, AgentTree, InputMode
│   │   ├── actions.rs   # Action enum
│   │   └── config.rs    # 設定
│   ├── monitor/         # モニタリング
│   │   └── task.rs      # 非同期監視タスク
│   ├── parsers/         # エージェント出力パーサー
│   │   ├── mod.rs       # AgentParser trait
│   │   ├── claude_code.rs
│   │   ├── opencode.rs
│   │   ├── codex_cli.rs
│   │   └── gemini_cli.rs
│   ├── tmux/            # tmux連携
│   │   ├── client.rs    # TmuxClient
│   │   └── pane.rs      # PaneInfo, プロセス検出
│   └── ui/              # UI実装
│       ├── app.rs       # メインループ
│       ├── layout.rs    # レイアウト定義
│       └── components/  # UIコンポーネント
│           ├── header.rs
│           ├── footer.rs
│           ├── agent_tree.rs
│           ├── pane_preview.rs
│           ├── subagent_log.rs
│           └── help.rs
└── Cargo.toml
```

## ライセンス

MIT License
