# Differences from Upstream Yazi (`DIFF_FROM_YAZI.md`)

This document tracks all custom features, architectural enhancements, UI/UX improvements, and configuration options implemented in **che** (`dual-yazi`) relative to upstream Yazi (`sxyazi/yazi`).

---

## 🚀 Key Custom Features & Enhancements

### 1. Dual-Pane Side-by-Side Layout (`dual_pane`)
- **Independent Tab Panes**: Support for dual-pane file management with independent navigation, selection, and directory state per pane.
- **Adaptive Narrow Window Fallback (`dual_pane_min_width`)**: Automatic fallback to single-pane layout when the terminal width drops below the configured threshold (`dual_pane_min_width`, default: `80` columns).
- **Narrow Mode Full-Screen Preview (`Ctrl+Q`)**: In narrow/single-pane mode, pressing `Ctrl+Q` seamlessly replaces the file list pane with a full-screen preview pane.
- **Pane Navigation & Switch (`Tab` / `Shift+Tab`)**: Instant focus switching between left and right panes using standard Commander-style hotkeys.
- **Cross-Pane Operations (`F5` / `F6`)**: Dedicated `copy_to` (`F5`) and `move_to` (`F6`) actions to copy or move selected files directly to the opposite pane.

### 2. Double Commander Style Batch Multi-Rename TUI (`multirename`)
- **Interactive Multi-Rename TUI**: Integrated Double Commander style batch renaming TUI plugin (`multirename`).
- **Binary Discovery & Fallback**: Automatic discovery of `ch` / `che` binaries with environment variable `CHE_CH_PATH` and system PATH resolution.
- **Regex & Mask Support**: Built-in mask pattern parsing (`[N]`, `[E]`, `[N1-5]`), case conversions (`Upper`, `Lower`, `Title`), and regex search/replace.
- **Custom Keymap Integration**: Configurable binding (`c r` / `Ctrl+Shift+R`) in `~/.config/che/keymap.toml` while preserving standard text-editor rename behavior by default.

### 3. Native Custom Statusline & Linemodes
- **`cheline` Statusline Component**: Powerline-style custom statusline plugin with modular indicators (selected count, yanked status, git branch, task progress).
- **`commander` Linemode**: Two-column layout displaying file size and formatted modification timestamp (`%d.%m.%y %H:%M`) with green highlight for recently modified files (< 48 hours).
- **`adaptive` Responsive Linemode**: Intelligent linemode component that dynamically adjusts detail based on column width (`none` on narrow columns `< 32`, `size` on medium `< 46`, and full `commander` on wide columns).
- **Component Geometry Awareness**: `Linemode:new(file, active, area)` receives panel geometry `self._area` so Lua linemodes can query exact column width (`self._area.w`).

### 4. Cyrillic Keyboard & Input Enhancements
- **Layout-Aware Key Mapping**: Preserves modifier state (`Shift`, `Ctrl`, `Alt`) for non-ASCII / Cyrillic keyboard layouts (`Ctrl+Shift+к` ➔ `<C-S-r>`).
- **Input Autocomplete Reactive Triggering**: Typing ordinary characters in input fields (`type_str`) immediately tags input snapshot and triggers `flush_value()` so completion popups appear instantly on initial keypress without requiring backspace.

### 5. Bundled Dual-Pane Session Persistence (`che-session`)
- **Upstream Origin & Dual-Pane Extension**: Based on [barbanevosa/autosession.yazi](https://github.com/barbanevosa/autosession.yazi) and extended for `che`'s dual-pane architecture.
- **Dual-Pane Independent State**: Saves and restores open tabs and working directories for both Left (Pane 1) and Right (Pane 2) panes independently.
- **Per-Tab View Preferences**: Automatically persists sorting options (`by`, `reverse`, `dir_first`, `sensitive`, `translit`), linemodes (`commander`, `adaptive`, etc.), and hidden file visibility per tab.
- **Active Tab & Pane Focus**: Restores cursor/active tab index in each pane, active pane selection, and `single_pane` mode.
- **Built-in Bundled Plugin & Toggle Flag**: Delivered out-of-the-box as `che-session` preset plugin without requiring external package downloads. Can be enabled or disabled via `require("che-session"):setup({ enabled = true })` (or `enabled = false`).
- **Commands & Keymaps**: Supports `save-and-quit`, `save`, and `restore` actions via `plugin che-session -- save-and-quit`.

### 6. Double Commander Style Archive Manager & Password Protection (`che-archive`)
- **Interactive Archive Creation TUI**: Double Commander style archive creation interface supporting format selection (`zip`, `tar`, `7z`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`), compression levels (`0` Store to `9` Ultra), compression methods (`LZMA2`, `Deflate`, `ZSTD`, etc.), solid block compression, and password encryption.
- **Header Encryption for 7z**: Option to encrypt archive file lists and metadata (`-mhe=on`).
- **Cross-Pane Packing & Extraction**: `che-archive` plugin bindings to pack files into the active or opposite pane (`c a`, `c A`) and extract archives into the active or opposite pane (`c x`, `c X`, `c e`).
- **Interactive Password Prompt**: Native masked password input dialog (`InputCfg::password`) when opening password-protected archives, with session-level password caching in `ARCHIVE_PASSWORDS` and `-p-` stdin null redirection to prevent CLI tool hangs.

### 7. Bundled Fast Directory Bookmarks & Hops (`che-bookmarks`)
- **Interactive Preset Plugin**: Built-in bookmarks plugin supporting fast directory jumping, interactive bookmark creation with custom keys/prefixes and descriptions (`<Enter>`), and inline deletion (`<Delete>`).
- **Fuzzy Search & Tab Jumps**: Built-in fuzzy finder integration (`fzf`) and dynamic tab jumping (`1..9`) for open tabs in the active pane.
- **Persistence & Config Toggle**: Automatically persists custom bookmarks in `~/.config/che/bookmarks.json` and supports configuration via `require("che-bookmarks"):setup({ enabled = true, hops = { ... } })`.

### 8. Binaries & Versioning Scheme
- **Binary Names**: Main TUI executable is named **`che`** (`yazi-fm`), and CLI helper tool is named **`ch`** (`yazi-cli`).
- **Calendar Versioning (CalVer)**: Adheres strictly to CalVer scheme (`YY.M.D`, e.g., `26.8.18`).

---

## ⚙️ Configuration Reference (`~/.config/che/`)

```toml
# ~/.config/che/yazi.toml
[mgr]
linemode            = "adaptive"
dual_pane_min_width = 80
```

```toml
[[mgr.prepend_keymap]]
on   = [ "c", "r" ]
run  = "plugin multirename"
desc = "Multi-Rename (Double Commander style TUI)"

# ── Archive Pack & Extract (che-archive) ──────────────────────────────────────
[[mgr.prepend_keymap]]
on   = [ "c", "a" ]
run  = "plugin che-archive -- pack-opposite"
desc = "Pack files to archive (opposite pane)"

[[mgr.prepend_keymap]]
on   = [ "c", "A" ]
run  = "plugin che-archive -- pack"
desc = "Pack files to archive (current pane)"

[[mgr.prepend_keymap]]
on   = [ "c", "x" ]
run  = "plugin che-archive -- extract-opposite"
desc = "Extract archive to opposite pane"

[[mgr.prepend_keymap]]
on   = [ "c", "X" ]
run  = "plugin che-archive -- extract"
desc = "Extract archive to current pane"

[[mgr.prepend_keymap]]
on   = [ "c", "e" ]
run  = "plugin che-archive -- extract-to-folder"
desc = "Extract archive to subdirectory"

# ── Fast Bookmarks (che-bookmarks) ───────────────────────────────────────────
[[mgr.prepend_keymap]]
on   = ";"
run  = "plugin che-bookmarks"
desc = "Hop to bookmark (che-bookmarks)"

[[mgr.prepend_keymap]]
on   = "'"
run  = "plugin che-bookmarks -- fuzzy"
desc = "Fuzzy search bookmarks (che-bookmarks)"

# ── Custom Linemodes ─────────────────────────────────────────────────────────
[[mgr.prepend_keymap]]
on   = [ "m", "c" ]
run  = "linemode commander"
desc = "Linemode: commander"

[[mgr.prepend_keymap]]
on   = [ "m", "a" ]
run  = "linemode adaptive"
desc = "Linemode: adaptive"
```

```lua
-- ~/.config/che/init.lua

-- ── che-session: Dual-pane session persistence ──────────────────────────────
require("che-session"):setup({
  enabled = true, -- Set to false to disable auto-restore
})

-- ── che-bookmarks: Fast directory hops & bookmarks ──────────────────────────
require("che-bookmarks"):setup({
  enabled = true,        -- Set to false to deactivate plugin
  persist = true,        -- Automatically persist custom bookmarks to bookmarks.json
  desc_strategy = "path",-- "path" or "filename"
  ephemeral = true,      -- Allow interactive <Enter> creation and <Delete> removal
  tabs = true,           -- Include dynamic tab hops for the active pane (1..9)
  fuzzy_cmd = "fzf",     -- Fuzzy finder command
  hops = {
    { key = "/", path = "/" },
    { key = "t", path = "/tmp" },
    { key = "~", path = "~", desc = "Home" },
    { key = "d", path = "~/Desktop", desc = "Desktop" },
    { key = "D", path = "~/Documents", desc = "Documents" },
    { key = "c", path = "~/.config", desc = "Config files" },
    { key = { "l", "s" }, path = "~/.local/share", desc = "Local share" },
  },
})

-- ── Custom Linemodes ────────────────────────────────────────────────────────
function Linemode:commander()
  local size = self._file:size()
  local size_str = size and ya.readable_size(size) or "  -"
  local mtime = self._file.cha.mtime
  local mtime_str = mtime and os.date("%d.%m.%y %H:%M", math.floor(mtime)) or "----"
  local result = string.format("%8s  %s", size_str, mtime_str)

  if mtime and os.time() - math.floor(mtime) <= 172800 then
    return ui.Line({ ui.Span(result):fg("green") })
  end
  return result
end

function Linemode:adaptive()
  local width = self._area and self._area.w or 80
  if width < 32 then
    return ""
  end

  local size = self._file:size()
  local size_str = size and ya.readable_size(size) or "  -"
  if width < 46 then
    return size_str
  end

  local mtime = self._file.cha.mtime
  local mtime_str = mtime and os.date("%d.%m.%y %H:%M", math.floor(mtime)) or "----"
  local result = string.format("%8s  %s", size_str, mtime_str)

  if mtime and os.time() - math.floor(mtime) <= 172800 then
    return ui.Line({ ui.Span(result):fg("green") })
  end
  return result
end

Linemode.size_and_mtime = Linemode.commander
```
