<p align="center">
  <img src="https://raw.githubusercontent.com/suwakei/logo/main/Golta/GoltaLogo.png" alt="GoltaLogo" width="450">
</p>

<p align="center">
  <b>高速でクロスプラットフォームなGoバージョンマネージャー。Voltaスタイルのシームレスな切り替えが可能。</b>
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

Goltaは、Goツールチェーンを管理するための手間のかからない方法です。[Volta](https://volta.sh/)のシームレスな体験にインスパイアされた、Goバージョンのインストールと切り替えのための高速でクロスプラットフォームなソリューションを提供します。

## ✨ 特徴

- **高速**: Rustで構築されており、最適なパフォーマンスを実現します。
- **クロスプラットフォーム**: Windows、macOS、Linuxで動作します。
- **シームレスな切り替え**: プロジェクトの `go.mod` ファイルに基づいて、Goのバージョンを自動的に切り替えます。
- **簡単なインストール**: コマンド一つで使い始めることができます。
- **依存関係なし**: シングルバイナリとして配布されます。

## 🚀 はじめに

### インストール

#### macOS / Linux

インストーラースクリプトを使用してGoltaをインストールできます：

```sh
curl https://raw.githubusercontent.com/suwakei/golta/main/install.sh | sh
```

またはHomebrew経由で：

```sh
# TODO: Homebrewに追加
brew install suwakei/tap/golta
```

#### Windows

Scoopを使用してGoltaをインストールできます：

```sh
# TODO: Scoopバケットに追加
scoop bucket add suwakei https://github.com/suwakei/scoop-bucket.git
scoop install suwakei/golta
```

または、Releasesページから最新のリリースをダウンロードしてください。

### シェルの設定

インストールを完了するには、Goltaのホームディレクトリをシェルの `PATH` に追加する必要があります。

**Bash/Zsh:**

`~/.bashrc` または `~/.zshrc` に以下を追加してください：

```sh
export GOLTA_HOME="$HOME/.golta"
export PATH="$GOLTA_HOME/bin:$PATH"
```

**PowerShell:**

PowerShellプロファイル (`$PROFILE`) に以下を追加してください：

```powershell
$env:GOLTA_HOME = "$HOME\.golta"
$env:PATH = "$env:GOLTA_HOME\bin;$env:PATH"
```

## 📖 使い方

使い始めるための一般的なコマンドをいくつか紹介します：

- **特定のGoバージョンをインストールする:**

  ```sh
  golta install 1.21.5
  ```

- **デフォルトのGoバージョンを設定する:**

  ```sh
  golta default 1.21.5
  ```

- **プロジェクトにGoバージョンを固定（ピン留め）する:**
  Goltaは、プロジェクトの `go.mod` ファイルで指定されたバージョンを自動的に検出します。

  ```go
  // go.mod
  go 1.21
  ```

  このプロジェクト内で `go` コマンドを実行すると、Goltaは自動的に一致するGoバージョンを使用します。

- **インストールされているGoバージョンを一覧表示する:**
  ```sh
  golta list
  ```

## ❤️ 貢献

貢献は大歓迎です！プルリクエストを送信するか、Issueを作成してください。

## 📜 ライセンス

このプロジェクトはMITライセンスの下でライセンスされています。
