local wezterm = require("wezterm")
local act = wezterm.action
local config = wezterm.config_builder()

-- Theme
config.color_scheme = "tokyonight"

-- Font: non-NF Maple as primary so cell metrics are text-sized. Font Awesome 7
-- Brands picks up brand icons (Maple non-NF has no PUA overlap with FA brands),
-- then Maple Mono NF covers the rest of the Nerd Font glyph set.
-- · ✢ * ✶ ✻ ✽
config.font = wezterm.font_with_fallback({
	"Maple Mono",
	"Font Awesome 7 Brands",
	"Maple Mono NF",
})
config.font_size = 11
config.warn_about_missing_glyphs = false
config.line_height = 1.45
config.underline_position = "-0.25cell"
config.underline_thickness = "3px"
config.enable_kitty_keyboard = true

-- Window [info]
config.window_decorations = "RESIZE|MACOS_FORCE_SQUARE_CORNERS|MACOS_FORCE_DISABLE_SHADOW"
config.window_frame = {
	border_left_width = 0,
	border_right_width = 0,
	border_top_height = 0,
	border_bottom_height = 0,

	-- tab bar
	font = wezterm.font({ family = "Maple Mono NF", weight = "Thin" }),
	font_size = 2.0,
	inactive_titlebar_bg = "#000000",
	active_titlebar_bg = "#000000",
}
config.use_fancy_tab_bar = true
config.hide_tab_bar_if_only_one_tab = false
config.show_new_tab_button_in_tab_bar = false
config.show_close_tab_button_in_tabs = false
config.show_tab_index_in_tab_bar = false
config.tab_bar_at_bottom = false
config.window_padding = { left = 0, right = 0, top = 0, bottom = 0 }
config.window_background_opacity = 1.0
config.colors = { split = "#1A1B26" }
config.scrollback_lines = 10000000
config.enable_scroll_bar = false
config.macos_fullscreen_extend_behind_notch = true
config.native_macos_fullscreen_mode = false

-- Mouse
config.hide_mouse_cursor_when_typing = true

-- macOS
config.send_composed_key_when_left_alt_is_pressed = false
config.send_composed_key_when_right_alt_is_pressed = true

-- Bell
config.audible_bell = "Disabled"

-- Misc
config.check_for_updates = false
config.automatically_reload_config = true
config.default_cursor_style = "BlinkingBar"

-- Disable default keybindings, build from scratch
config.disable_default_key_bindings = true

-- Shared paths
local PATH = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
local mux_bin = os.getenv("HOME") .. "/.config/tmux/scripts/mux"

-- Helpers
local tmux_prefix = "\x00" -- Ctrl+Space
local function csi(code)
	return "\x1b[" .. code
end

-- Detect MacBook built-in display (14"/16" 2021+ have a notch). The active
-- screen's name comes from wezterm.gui.screens(); names vary by machine/OS, so
-- match on common substrings.
local function is_notched_screen()
	local ok, screens = pcall(wezterm.gui.screens)
	if not ok or not screens or not screens.active then
		return false
	end
	local name = (screens.active.name or ""):lower()
	return name:find("built%-in") ~= nil or name:find("builtin") ~= nil or name:find("liquid retina") ~= nil
end

-- Push notched state to tmux and refresh the status bar. Triggered when the
-- window enters/leaves fullscreen or moves between screens. Uses
-- background_child_process (non-blocking, non-tty) to avoid stalling the GUI
-- thread and to detach from the hardened-runtime parent cleanly.
local function update_notched(window)
	local dims = window:get_dimensions()
	local notched = dims.is_full_screen and is_notched_screen()
	local val = notched and "1" or "0"
	local overrides = window:get_config_overrides() or {}
	local desired_split = notched and "#000000" or "#1A1B26"
	local desired_hide = not notched
	local dirty = false
	if not overrides.colors or overrides.colors.split ~= desired_split then
		overrides.colors = { split = desired_split }
		dirty = true
	end
	if overrides.hide_tab_bar_if_only_one_tab ~= desired_hide then
		overrides.hide_tab_bar_if_only_one_tab = desired_hide
		dirty = true
	end
	if dirty then
		window:set_config_overrides(overrides)
	end
	wezterm.background_child_process({
		"/bin/sh",
		"-c",
		"export PATH="
			.. PATH
			.. "; "
			.. "tmux set-option -g @notched "
			.. val
			.. " >/dev/null 2>&1; "
			.. mux_bin
			.. " update '' "
			.. "\"$(tmux display -p '#{client_width}' 2>/dev/null)\" "
			.. ">/dev/null 2>&1",
	})
end

wezterm.on("gui-startup", function(cmd)
	local _, _, window = wezterm.mux.spawn_window(cmd or {})
	local gui = window:gui_window()
	gui:maximize()
	gui:toggle_fullscreen()
end)
wezterm.on("window-resized", function(window)
	update_notched(window)
end)
wezterm.on("window-config-reloaded", function(window)
	update_notched(window)
end)

local function sidebar_enabled()
	local f = io.popen("PATH=" .. PATH .. " tmux show-option -gv @sidebar_enabled 2>/dev/null")
	if not f then
		return true
	end
	local out = f:read("*a") or ""
	f:close()
	return out:gsub("%s+$", "") ~= "0"
end

local function find_sidebar(tab)
	for _, info in ipairs(tab:panes_with_info()) do
		if info.pane:get_user_vars().is_sidebar == "true" then
			return info
		end
	end
	return nil
end

local function open_sidebar(window, pane)
	if not sidebar_enabled() then
		return
	end
	window:perform_action(
		act.SplitPane({
			direction = "Left",
			size = { Cells = 45 },
			command = {
				args = { mux_bin, "sidebar" },
				set_environment_variables = {
					PATH = PATH,
				},
			},
		}),
		pane
	)
end

local function toggle_sidebar(window, pane)
	local info = find_sidebar(pane:tab())
	if info then
		os.execute(
			"PATH="
				.. PATH
				.. " "
				.. "tmux set-option -gu @sidebar_open 2>/dev/null; "
				.. "PATH="
				.. PATH
				.. " "
				.. mux_bin
				.. " update >/dev/null 2>&1 &"
		)
		info.pane:activate()
		window:perform_action(act.CloseCurrentPane({ confirm = false }), info.pane)
	else
		open_sidebar(window, pane)
	end
end

local function cmd_p_handler(window, pane)
	local info = find_sidebar(pane:tab())
	if info then
		info.pane:activate()
		window:perform_action(act.SendString("/"), info.pane)
	else
		window:perform_action(act.SendString(csi("63~")), pane)
	end
end

local function focus_sidebar(window, pane)
	local tab = pane:tab()
	local info = find_sidebar(tab)
	if not info then
		open_sidebar(window, pane)
		return
	end
	if info.is_active then
		window:perform_action(act.SendString("\x0f"), pane)
	else
		info.pane:activate()
	end
end

config.keys = {
	-- Clipboard
	{ key = "c", mods = "SUPER", action = act.CopyTo("Clipboard") },
	{ key = "v", mods = "SUPER", action = act.PasteFrom("Clipboard") },

	-- Font size
	{ key = "-", mods = "SUPER", action = act.DecreaseFontSize },
	{ key = "=", mods = "SUPER", action = act.IncreaseFontSize },
	{ key = "+", mods = "SUPER", action = act.IncreaseFontSize },
	{ key = "0", mods = "SUPER", action = act.ResetFontSize },

	-- Window management
	{ key = "f", mods = "CTRL|ALT|SUPER", action = act.ToggleFullScreen },
	{ key = "w", mods = "SUPER", action = act.CloseCurrentPane({ confirm = false }) },
	{ key = "w", mods = "SUPER|SHIFT", action = act.CloseCurrentTab({ confirm = false }) },
	{ key = "q", mods = "SUPER", action = act.QuitApplication },
	{ key = "n", mods = "SUPER|ALT", action = act.SpawnWindow },
	{ key = "r", mods = "SUPER|SHIFT", action = act.ReloadConfiguration },
	{ key = "Enter", mods = "SHIFT", action = act.SendString("\n") },

	-- Session sidebar
	{ key = "e", mods = "SUPER|SHIFT", action = wezterm.action_callback(toggle_sidebar) },
	{ key = "o", mods = "SUPER", action = wezterm.action_callback(focus_sidebar) },

	-- Cmd+P: sidebar-aware session chooser (conditional — not table-driven)
	{ key = "p", mods = "SUPER", action = wezterm.action_callback(cmd_p_handler) },
}

-- {{{ tmux relay: prefix + key
-- Cmd+1..9 select tmux windows; Cmd+Shift+F opens tmux-fzf; Cmd+Alt+X ditches session
local prefix_relay = {
	{ key = "f", mods = "SUPER|SHIFT", suffix = "F" },
	{ key = "x", mods = "SUPER|ALT", suffix = "X" },
}
for i = 1, 9 do
	table.insert(prefix_relay, { key = tostring(i), mods = "SUPER", suffix = tostring(i) })
end
for _, r in ipairs(prefix_relay) do
	table.insert(config.keys, { key = r.key, mods = r.mods, action = act.SendString(tmux_prefix .. r.suffix) })
end
-- }}}

-- {{{ tmux relay: CSI user-key sequences
local csi_relay = {
	{ key = "Tab", mods = "CTRL", csi = "60~" },
	{ key = ";", mods = "SUPER", csi = "61~" },
	{ key = "n", mods = "SUPER", csi = "62~" },
	{ key = "n", mods = "SUPER|SHIFT", csi = "64~" },
	{ key = "p", mods = "SUPER|SHIFT", csi = "65~" },
	{ key = "mapped:<", mods = "SUPER|SHIFT", csi = "66~" },
	{ key = "mapped:>", mods = "SUPER|SHIFT", csi = "67~" },
	{ key = "n", mods = "SUPER|CTRL", csi = "68~" },
	{ key = "[", mods = "CTRL|ALT", csi = "69~" },
}
for _, r in ipairs(csi_relay) do
	table.insert(config.keys, { key = r.key, mods = r.mods, action = act.SendString(csi(r.csi)) })
end
-- }}}

-- {{{ vim relay: CSI sequences
local vim_relay = {
	{ key = "j", mods = "SUPER", csi = "90;1~" },
	{ key = "s", mods = "SUPER", csi = "90;2~" },
	{ key = "c", mods = "SUPER|SHIFT", csi = "90;3~" },
	{ key = "c", mods = "SUPER|ALT|SHIFT", csi = "90;4~" },
	{ key = ".", mods = "SUPER", csi = "90;6~" },
	{ key = "e", mods = "SUPER", csi = "90;7~" },
	{ key = "b", mods = "SUPER", csi = "90;8~" },
	{ key = "i", mods = "SUPER", csi = "90;9~" },
	{ key = "l", mods = "SUPER", csi = "90;10~" },
	{ key = "i", mods = "SUPER|SHIFT", csi = "90;11~" },
	{ key = "k", mods = "SUPER", csi = "90;12~" },
	{ key = "v", mods = "SUPER|ALT", csi = "90;13~" },
	{ key = "d", mods = "SUPER", csi = "90;14~" },
	{ key = "d", mods = "SUPER|SHIFT", csi = "90;15~" },
	{ key = "u", mods = "SUPER", csi = "90;16~" },
	{ key = "u", mods = "SUPER|SHIFT", csi = "90;17~" },
	{ key = "k", mods = "SUPER|ALT", csi = "90;18~" },
	{ key = "j", mods = "SUPER|ALT", csi = "90;19~" },
	{ key = "k", mods = "SUPER|ALT|SHIFT", csi = "90;20~" },
	{ key = "j", mods = "SUPER|ALT|SHIFT", csi = "90;21~" },
}
for _, r in ipairs(vim_relay) do
	table.insert(config.keys, { key = r.key, mods = r.mods, action = act.SendString(csi(r.csi)) })
end
-- }}}

return config
