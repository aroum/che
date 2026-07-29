# CHANGELOG — `che` (`dual-yazi`)

This document tracks all new features, configuration options, utilities, and keybindings implemented since the baseline refactoring (`c8921e3c`).

---

## ⚙️ New Configuration Options (`yazi.toml`)

All new options can be configured in `~/.config/che/yazi.toml`.

```toml
[mgr]
# Automatic switch to glob mode for wildcard search patterns such as *.py or *.txt
# true  - if search query starts with * (e.g. *.py) and is not regex (.*),
#         fd automatically uses --glob (default)
# false - pass all search queries directly as regex (--regex)
smart_glob = true

# Display "↑ .." entry at the top of directory listings
# true  - display "↑ .." as the first entry in directory lists (default)
# false - hide virtual parent entry
show_upparent = true

# Virtual archive browsing
# true  - enter supported archives (.zip, .tar, .7z, etc.) as virtual directories (default)
# false - default opener / extraction behavior
archive_vfs = true

# Control cursor display in inactive dual-pane column
# true  - completely hide cursor selection and side indicators on inactive pane (default)
# false - display unified cursor on inactive pane dimmed along with the rest of the column
hide_inactive_cursor = true

[input]
# Toggle Vim input behavior (rename, file/folder creation, cd)
# false - single Esc press immediately closes input dialogs (default)
# true  - Esc switches to Vim Normal mode, second Esc closes the input
vim_mode = false

# Auto-highlight filename stem (excluding extension) during rename
# true  - for "file.txt", only "file" will be selected
# false - full string "file.txt" will be selected
rename_highlight_stem = true

cursor_blink = false

[confirm]
# Double Commander style overwrite confirmation dialog
# true  - full popup menu of Double Commander options with inline hotkeys:
#         [O]verwrite     Overwrite [A]ll    Overwrite Ol[d]er
#         [S]kip          Skip A[l]l         Auto [R]ename
#         Co[m]pare       [C]ancel
overwrite_dialog = true
```

### 🚩 Command Line Flags (CLI Flags)

| CLI Flag | Description |
| :--- | :--- |
| **`--single`** | Run `che` in **single-pane** mode (dual-pane mode is active by default). |
| **`--debug`** | Enable verbose debug mode with event logging. |

---

## 🚀 Key Features

### 1. Smart Auto-Switch to Glob Search (`smart_glob = true`)
* **Description:** 
  * When searching files with wildcard patterns (e.g., `*.py`, `*.txt`, `*.png`), search automatically recognizes the pattern and activates `--glob`.
  * Standard regular expressions (`.*\.py`, `test.*`, `[a-z]*`) continue to be evaluated as regex without breakage.

---

### 2. Fast Disk / Volume Picker with Single-Match Opening (`g m`)
* **Description:** 
  * **Single match:** Pressing a letter (e.g., **`s`** for drive `soft`), if exactly one drive matches, it **opens immediately** without needing to press Enter.
  * **Multiple matches:** If multiple drives start with **`s`**, repeated presses of **`s`** **cyclically navigate selection** between them, and pressing **`Enter`** confirms opening.

---

### 3. Dual-Pane Plugin State Isolation (`starship.yazi` & others)
* **Description:** 
  * Implemented per-CWD state isolation for Lua plugins in `Header`.
  * Third-party header plugins (such as `starship.yazi`) work **without any modifications to their source code** and render independent prompts for both left and right panes simultaneously.

---

### 4. Inline Letter Hotkeys in Batch Rename (`multirename`)
* **Hotkeys:** **`Alt + O`** (`[ [O]K ]`), **`Alt + C`** (`[ [C]ancel ]`)
* **Description:** 
  * Action buttons display inline hotkey formatting: **`[ [O]K ]`** and **`[ [C]ancel ]`**.
  * Hotkeys **`Alt + O`** and **`Alt + C`** trigger instant execution and dismissal.

---

### 5. Double Commander Style Overwrite Dialog (`overwrite_dialog = true`)
* **Configuration Flag:** `[confirm] overwrite_dialog = true`
* **Full list of options and inline hotkeys:**
  * **`o`** / **`y`** — **`[O]verwrite`** (Overwrite current file)
  * **`a`** — **`Overwrite [A]ll`** (Overwrite all matching files)
  * **`d`** — **`Overwrite Ol[d]er`** (Overwrite only older files)
  * **`s`** / **`n`** — **`[S]kip`** (Skip current file)
  * **`l`** — **`Skip A[l]l`** (Skip all)
  * **`r`** — **`Auto [R]ename`** (Auto-rename to `file_copy.ext`)
  * **`m`** — **`Co[m]pare`** (Compare file contents)
  * **`c`** / **`Esc`** — **`[C]ancel`** (Cancel operation)

---

### 6. Dual-Pane Mode by Default
* **Default Behavior:** Launching `che` opens two independent file management columns with separate navigation and history.
* **Binaries & Configuration:** Application binaries are named **`che`** and **`ch`**, with configuration stored in `~/.config/che/`.

---

### 7. Letter Jump Navigation Mode (`JUMP`)
* **Hotkey:** `{ on = "<C-j>", run = "jump_mode", desc = "Toggle letter jump mode" }`
* **Description:** 
  * Pressing `Ctrl + J` toggles **`JUMP`** mode (indicated by a magenta status bar badge).
  * Pressing any letter/digit moves the cursor to the first matching entry (case-insensitive).
  * Repeated keypresses cycle through all matching items.

---

### 8. Input Vim-Mode Configuration (`vim_mode`)
* **Configuration Flag:** `[input] vim_mode = false | true`
* **Description:** 
  * `vim_mode = false` (*default*): Single `Esc` press immediately closes input dialogs.
  * `vim_mode = true`: First `Esc` switches input to Vim Normal mode, second `Esc` closes the dialog.

---

### 9. Cross-Platform Disk Picker (`g m`)
* **Hotkey:** `{ on = [ "g", "m" ], run = "disks", desc = "Select disk/volume" }`

---

### 10. File Comments Support (`descript.ion`)
* **Hotkey:** `{ on = [ "c", "m" ], run = "comment", desc = "Set file comment" }`

---

### 11. System Clipboard Copy
* **Hotkey:** `{ on = "<C-Y>", run = "plugin system_copy", desc = "Copy files to OS clipboard" }`

---

### 12. Audio Previewer & Album Art Display
* **Description:** Automatic previewer for audio files (MP3, FLAC, M4A, WAV) displaying album cover art via `ffmpeg` / `exiftool`.

---

## ⌨️ Summary of New Hotkeys and Commands

| Hotkey | Command (`run = ...`) | Description |
| :--- | :--- | :--- |
| **`Ctrl + J`** | `jump_mode` | Toggle letter jump navigation mode (`JUMP`) |
| **`g` `m`** | `disks` | Open disk and volume picker popup |
| **`c` `m`** | `comment` | Add / edit file comment (`descript.ion`) |
| **`Ctrl + Shift + R`** / **`c` `r`** | `plugin multirename` | Open batch multirename TUI plugin |
| **`Ctrl + Shift + Y`** / **`c` `y`** | `plugin system_copy` | Copy selected files to OS clipboard |
| **`F5`** | `copy_to` | Copy selected files to opposite pane |
| **`F6`** | `move_to` | Move selected files to opposite pane |
| **`Tab`** / **`Shift + Tab`** | `pane_switch` | Switch focus between left and right panes |
