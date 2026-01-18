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

Golta
A fast, seamless Go version manager powered by Rust. 🦀

Inspired by Volta, Golta allows you to switch Go versions automatically based on your project directory.

## ✨ Features

- **Fast**: Built with Rust for optimal performance.
- **Cross-Platform**: Works on Windows, macOS, and Linux.
- **Seamless Switching**: Automatically switches Go versions based on .golta.json or your project's go.mod file.
- **Simple Installation**: Get started with a single command.
- **No Dependencies**: Distributed as a single binary.

## 🚀 Getting Started

### Installation

#### macOS / Linux

You can install Golta using the installer script:

```bash
curl -fsSL https://golta-website.vercel.app/install | bash
```

#### Windows

```powershell
iwr -useb https://golta-website.vercel.app/install_win | iex
```

[!TIP] After installation, restart your terminal to start using golta.

## 📖 Usage

### Installing Go

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

For more detailed guides, troubleshooting,visit our documentation: [golta.dev](https://golta-website.vercel.app/)

## ❤️ Contributing

Contributions are welcome! Please feel free to submit a pull request or create an issue.

## 📜 License

This project is licensed under the MIT License.
