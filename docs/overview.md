# TalkWithRustGPT - プロジェクト概要

## アプリケーション概要

**TalkWithRustGPT** は、Tauri フレームワークを使用したデスクトップ AI チャットアプリケーションです。
Rust バックエンド + React/TypeScript フロントエンドで構成され、複数の AI サービスに統一インターフェースでアクセスできます。

## 対応 AI サービス

| サービス | プロバイダ | 主な用途 |
|---------|-----------|---------|
| ChatGPT | OpenAI | テキスト会話・画像理解 |
| Claude | Anthropic | テキスト会話・画像理解 |
| Gemini | Google | テキスト会話・画像理解 |
| DALL-E 3 | OpenAI | 画像生成（`/image` コマンド経由） |

## 主な機能

- **マルチ AI 切り替え**: Claude / ChatGPT / Gemini をボタン一つで切り替え
- **モデル性能切り替え**: 高性能モデル / 経済的モデルの二段階切り替え
- **音声入力**: Web Speech API を使った日本語音声認識
- **画像入力**: クリップボードへの貼り付けやドラッグ＆ドロップによる画像送信
- **ファイルドラッグ＆ドロップ**: テキストファイルをチャットに読み込み
- **会話履歴の保存**: メモファイルとしてドキュメントフォルダへ自動保存
- **システムプロンプト**: 会話トーンを変えるプリセットプロンプト
- **音声読み上げ**: 棒読みちゃん連携による AI 応答の読み上げ
- **マークダウンレンダリング**: AI の応答を HTML に変換して表示

## 技術スタック

### フロントエンド

| ライブラリ | バージョン | 役割 |
|-----------|-----------|------|
| React | 18.3.x | UI フレームワーク |
| TypeScript | 5.6.x | 型付き JavaScript |
| Ant Design | 5.24.x | UI コンポーネントライブラリ |
| Vite | 6.0.x | ビルドツール |
| react-speech-recognition | 4.0.x | 音声認識 |
| highlight.js | 11.x | コードハイライト |

### バックエンド

| クレート | バージョン | 役割 |
|---------|-----------|------|
| tauri | 2.x | デスクトップアプリフレームワーク |
| reqwest | 0.12.x | HTTP クライアント |
| serde / serde_json | 1.x | シリアライゼーション |
| markdown | 1.0.0-alpha | マークダウン→HTML 変換 |
| tiktoken-rs | 0.6.x | トークン数計算 |
| chrono | 0.4.x | 日時処理 |
| directories | 6.0.x | ユーザーディレクトリ取得 |
| bouyomi4rs | 0.2.x | 棒読みちゃん連携 |
| dotenv | 0.15.x | 環境変数管理 |

## プロジェクト構成

```
TalkWithRustGPT/
├── src/                          # フロントエンド (React/TypeScript)
│   ├── App.tsx                   # メインコンポーネント
│   ├── components/
│   │   ├── drug.tsx              # ファイルD&Dコンポーネント
│   │   ├── image.tsx             # 画像表示・変換コンポーネント
│   │   └── prompts.tsx           # プリセットプロンプト定義
│   └── common/
│       └── string.tsx            # 文字列ユーティリティ
├── src-tauri/                    # バックエンド (Rust)
│   └── src/
│       ├── lib.rs                # エントリポイント・Tauriコマンド登録
│       ├── manage/
│       │   ├── chatgpt.rs        # ChatGPT API クライアント
│       │   ├── claude.rs         # Claude API クライアント
│       │   ├── gemini.rs         # Gemini API クライアント
│       │   ├── message.rs        # メッセージ履歴管理
│       │   ├── filetitle.rs      # ファイル名生成
│       │   └── utils.rs          # 共通ユーティリティ
│       └── sub/
│           ├── prompts.rs        # システムプロンプト定義
│           └── voice.rs          # 棒読みちゃん連携
└── public/                       # 静的アセット (アイコン等)
```
