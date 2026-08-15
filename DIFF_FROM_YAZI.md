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

### 6. Binaries & Versioning Scheme
- **Binary Names**: Main TUI executable is named **`che`** (`yazi-fm`), and CLI helper tool is named **`ch`** (`yazi-cli`).
- **Calendar Versioning (CalVer)**: Adheres strictly to CalVer scheme (`YY.M.D`, e.g., `26.8.15`).

---

## ⚙️ Configuration Reference (`~/.config/che/`)

```toml
# ~/.config/che/yazi.toml
[mgr]
linemode            = "adaptive"
dual_pane_min_width = 80
```

```toml
# ~/.config/che/keymap.toml
[[mgr.prepend_keymap]]
on   = [ "c", "r" ]
run  = "plugin multirename"
desc = "Multi-Rename (Double Commander style TUI)"

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
