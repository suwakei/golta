<p align="center">
  <img src="https://raw.githubusercontent.com/suwakei/logo/main/Golta/GoltaLogo.png" alt="GoltaLogo" width="450">
</p>

<p align="center">
  <b>A fast, cross-platform Go version manager with Volta-style seamless switching.</b>
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
      		<th style="text-align:center">English</th>
      		<th style="text-align:center"><a href="README_ja.md">日本語</a></th>
    	</tr>
  	</thead>
</table>

Golta is a hassle-free way to manage your Go toolchains. It provides a fast, cross-platform solution for installing and switching between Go versions, inspired by the seamless experience of [Volta](https://volta.sh/).

## ✨ Features

- **Fast**: Built with Rust for optimal performance.
- **Cross-Platform**: Works on Windows, macOS, and Linux.
- **Seamless Switching**: Automatically switches Go versions based on your project's `go.mod` file.
- **Simple Installation**: Get started with a single command.
- **No Dependencies**: Distributed as a single binary.

## 🚀 Getting Started

### Installation

#### macOS / Linux

You can install Golta using the installer script:

```sh
curl -fsSL https://golta-website.vercel.app/install | bash
```

Or via Homebrew:

```bash
# TODO: Add to Homebrew
brew install suwakei/tap/golta
```

#### Windows

```sh
iwr -useb https://golta-website.vercel.app/install_win | iex
```

### Shell Setup

To complete the installation, you need to add Golta's home directory to your shell's `PATH`.

**Bash/Zsh:**

Add the following to your `~/.bashrc` or `~/.zshrc`:

```sh
export GOLTA_HOME="$HOME/.golta"
export PATH="$GOLTA_HOME/bin:$PATH"
```

**PowerShell:**

Add the following to your PowerShell profile (`$PROFILE`):

```powershell
$env:GOLTA_HOME = "$HOME\.golta"
$env:PATH = "$env:GOLTA_HOME\bin;$env:PATH"
```

## 📖 Usage

Here are some common commands to get you started:

- **Install a specific Go version:**

  ```sh
  golta install 1.21.5
  ```

- **Set the default Go version:**

  ```sh
  golta default 1.21.5
  ```

- **Pin a Go version to a project:**
  Golta automatically detects the version specified in your project's `go.mod` file.

  ```go
  // go.mod
  go 1.21
  ```

  When you run `go` commands inside this project, Golta will automatically use the matching Go version.

- **List installed Go versions:**
  ```sh
  golta list
  ```

## ❤️ Contributing

Contributions are welcome! Please feel free to submit a pull request or create an issue.

## 📜 License

This project is licensed under the MIT License.
