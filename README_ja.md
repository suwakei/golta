<p align="center">
  <img src="https://raw.githubusercontent.com/suwakei/logo/main/Golta/GoltaLogo.png" alt="GoltaLogo" width="450">
</p>

<p align="center">
  <b>Voltaスタイルのシームレスな切り替え機能を備えた、高速でクロスプラットフォームなGoバージョンマネージャー。</b>
</p>

<p align="center">
  <a href="https://github.com/suwakei/golta/actions/workflows/test.yml"><img src="https://github.com/suwakei/golta/actions/workflows/test.yml/badge.svg" alt="Test Status"></a>
  <a href="https://github.com/suwakei/golta/releases"><img src="https://img.shields.io/github/v/release/suwakei/golta" alt="GitHub release"></a>
  <a href="https://github.com/suwakei/golta/blob/main/LICENSE"><img src="https://img.shields.io/github/license/suwakei/golta" alt="License"></a>
</p>

---

<table>
	<thead>
    	<tr>
      		<th style="text-align:center"><a href="README.md">English</a></th>
      		<th style="text-align:center">日本語</th>
    	</tr>
  	</thead>
</table>

Golta
Rust製の高速でシームレスなGoバージョンマネージャー。🦀

VoltaにインスパイアされたGoltaは、プロジェクトディレクトリに基づいてGoのバージョンを自動的に切り替えることができます。

## ✨ 特徴

- **高速**: Rustで構築されており、最適なパフォーマンスを発揮します。
- **クロスプラットフォーム**: Windows、macOS、Linuxで動作します。
- **シームレスな切り替え**: .golta.json またはプロジェクトの go.mod ファイルに基づいて、Goのバージョンを自動的に切り替えます。
- **簡単なインストール**: コマンド1つで使い始めることができます。
- **依存関係なし**: 単一のバイナリとして配布されます。

## 🚀 はじめに

### インストール

#### macOS / Linux

インストーラースクリプトを使用してGoltaをインストールできます：

```bash
curl -fsSL https://golta-website.vercel.app/install | bash
```

#### Windows

```powershell
iwr -useb https://golta-website.vercel.app/install_win | iex
```

[!TIP] インストール後、goltaを使用開始するにはターミナルを再起動してください。

## 📖 使い方

### Goのインストール

```shell
golta install go
```

or

```shell
golta install go@latest
```

```shell
go run main.go
```

より詳細なガイドやトラブルシューティングについては、ドキュメントをご覧ください: golta.dev

## ❤️ 貢献について

貢献は大歓迎です！お気軽にプルリクエストを送信したり、Issueを作成してください。

## 📜 ライセンス

このプロジェクトは MIT ライセンスの下でライセンスされています。
