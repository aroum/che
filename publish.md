# Release & Distribution Guide for `che`

Instructions for publishing **`che`** binaries and packages across Homebrew, Winget, Snapcraft, and Nix.

---

## 🍺 1. Homebrew (Brew Tap for macOS & Linux)

Allows macOS (Apple Silicon & Intel) and Linux users to install pre-built binaries:

```bash
brew tap aroum/che
brew install che
```

### Formula File (`Formula/che.rb`)

Create `Formula/che.rb` in your `aroum/homebrew-che` repository:

```ruby
class Che < Formula
  desc "Dual-pane terminal file manager written in Rust"
  homepage "https://github.com/aroum/che"
  version "0.1.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/aroum/che/releases/download/v0.1.0/che-v0.1.0-aarch64-apple-darwin.zip"
    sha256 "<SHA256_AARCH64_MAC>"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/aroum/che/releases/download/v0.1.0/che-v0.1.0-x86_64-apple-darwin.zip"
    sha256 "<SHA256_X86_64_MAC>"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/aroum/che/releases/download/v0.1.0/che-v0.1.0-x86_64-unknown-linux-gnu.zip"
    sha256 "<SHA256_X86_64_LINUX>"
  end

  def install
    bin.install "che"
    bin.install "ch"
  end

  test do
    system "#{bin}/che", "--version"
  end
end
```

---

## 🪟 2. Windows Package Manager (Winget)

Installable via `winget install aroum.che`.  Automated releases are published to `microsoft/winget-pkgs` using `vedantmgoyal9/winget-releaser`.

---

## 🐧 3. Snapcraft (Snap Store for Linux)

Installable via `snap install che-fm`.

---

## ❄️ 4. Nix (Flakes & Cachix)

Run directly using Nix Flakes:

```bash
nix run github:aroum/che
```
