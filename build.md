# Building `che` from Source

Instructions for building **`che`** on macOS, Linux, and Windows.

---

## 🛠 Prerequisites

- **Rust 1.78+** (with Cargo)
- **Git**
- *(Optional)* `ffmpeg` and `exiftool` for audio cover preview support.

---

## 📦 Compilation Steps

### 1. Clone Repository

```bash
git clone https://github.com/aroum/che.git
cd che
```

### 2. Debug Build (Fast Development Build)

```bash
cargo build
```

The resulting binaries will be located at:
- `./target/debug/che`
- `./target/debug/ch`

### 3. Release Build (Optimized Production Build)

```bash
cargo build --release
```

The resulting binaries will be located at:
- `./target/release/che`
- `./target/release/ch`

---

## 🚀 Installation

To install binaries into Cargo bin directory (`~/.cargo/bin`):

```bash
cargo install --path yazi-fm
cargo install --path yazi-cli
```
