# `che` — Dual-Pane Terminal File Manager

**`che`** is a fast dual-pane terminal file manager written in Rust, based on Yazi.

---

## 🌟 Key Features & Improvements

- **Dual-Pane Mode by Default:** Independent navigation, tab history, and operations between left and right columns.
- **Double Commander Overwrite Dialog:** Interactive overwrite popup featuring `[O]verwrite`, `Overwrite [A]ll`, `Overwrite Ol[d]er`, `[S]kip`, `Skip A[l]l`, `Auto [R]ename`, `Co[m]pare`, and `[C]ancel`.
- **Fast Disk & Volume Picker (`g m`):** Select mounted drives by letter. Opens single matches instantly and cycles through multiple matches.
- **Letter Jump Navigation (`Ctrl + J`):** Instant jumping to files matching typed letters.
- **Batch Multirename (`ch` / `Ctrl+Shift+R`):** TUI tool supporting masks, counter formatting, and case conversions.
- **Audio Preview & Album Art:** Displays embedded album covers for MP3, FLAC, M4A, and WAV files.
- **File Comments (`descript.ion`):** Read and edit file comments seamlessly.
- **System Clipboard Integration:** Copy files directly to native OS clipboard.

---

## ⚙️ Configuration & Paths

- Application Binary: `che`
- CLI Renamer Binary: `ch`
- Configuration Directory: `~/.config/che/` (`yazi.toml`, `keymap.toml`, `theme.toml`)

---

## ⌨️ Essential Hotkeys

- `Tab` / `Shift+Tab`: Switch active pane
- `F5`: Copy selected files to opposite pane
- `F6`: Move selected files to opposite pane
- `Ctrl + J`: Toggle Letter Jump mode
- `g` `m`: Open Disk / Volume Picker
- `c` `m`: Edit file comment (`descript.ion`)
- `Ctrl + Shift + R` / `c` `r`: Open batch multirename TUI
- `Ctrl + Shift + Y` / `c` `y`: Copy files to OS clipboard
