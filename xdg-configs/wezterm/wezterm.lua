local wezterm = require("wezterm")
local act = wezterm.action
local config = wezterm.config_builder()

-- Theme
config.color_scheme = "tokyonight"

-- Font (Nerd Font fallback for status bar glyphs)
config.font = wezterm.font_with_fallback({
	"Maple Mono NF",
	"IosevkaTerm Nerd Font Mono",
})
config.font_size = 13
config.warn_about_missing_glyphs = false
config.line_height = 1.45
config.underline_position = "-0.25cell"
config.underline_thickness = "3px"

-- Window
config.window_decorations = "RESIZE|MACOS_FORCE_SQUARE_CORNERS"
config.window_frame = {
	border_left_width = 0,
	border_right_width = 0,
	border_top_height = 0,
	border_bottom_height = 0,
}
config.hide_tab_bar_if_only_one_tab = true
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = false
config.window_padding = { left = 0, right = 0, top = 0, bottom = 0 }
config.window_background_opacity = 1.0
config.inactive_pane_hsb = { brightness = 1.0 }
config.colors = { split = "#11111b" }
config.scrollback_lines = 10000000
config.enable_scroll_bar = false

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

-- Helpers
local tmux_prefix = "\x00" -- Ctrl+Space
local function csi(code)
	return "\x1b[" .. code
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
	window:perform_action(
		act.SplitPane({
			direction = "Left",
			size = { Cells = 34 },
			command = {
				args = { os.getenv("HOME") .. "/.config/tmux/scripts/tmux-session", "sidebar" },
				set_environment_variables = {
					PATH = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin",
				},
			},
		}),
		pane
	)
end

local function toggle_sidebar(window, pane)
	local info = find_sidebar(pane:tab())
	if info then
		-- Restore the tmux session list in the status bar. The sidebar process
		-- is about to be killed and won't run its own cleanup.
		local tmux_session = os.getenv("HOME") .. "/.config/tmux/scripts/tmux-session"
		os.execute(
			"PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin "
				.. "tmux set-option -gu @sidebar_open 2>/dev/null; "
				.. "PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin "
				.. tmux_session
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
		-- Sidebar focused — switch to the other pane
		for _, p in ipairs(tab:panes_with_info()) do
			if p.pane:get_user_vars().is_sidebar ~= "true" then
				p.pane:activate()
				return
			end
		end
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
	{ key = "w", mods = "SUPER", action = act.CloseCurrentPane({ confirm = false }) },
	{ key = "w", mods = "SUPER|SHIFT", action = act.CloseCurrentTab({ confirm = false }) },
	{ key = "q", mods = "SUPER", action = act.QuitApplication },
	{ key = "n", mods = "SUPER|ALT", action = act.SpawnWindow },
	{ key = "r", mods = "SUPER|SHIFT", action = act.ReloadConfiguration },
	{ key = "Enter", mods = "SHIFT", action = act.SendString("\n") },

	-- Session sidebar
	{ key = "e", mods = "SUPER|SHIFT", action = wezterm.action_callback(toggle_sidebar) },
	{ key = "o", mods = "SUPER", action = wezterm.action_callback(focus_sidebar) },

	-- {{{ tmux relay: prefix + key
	{ key = "1", mods = "SUPER", action = act.SendString(tmux_prefix .. "1") },
	{ key = "2", mods = "SUPER", action = act.SendString(tmux_prefix .. "2") },
	{ key = "3", mods = "SUPER", action = act.SendString(tmux_prefix .. "3") },
	{ key = "4", mods = "SUPER", action = act.SendString(tmux_prefix .. "4") },
	{ key = "5", mods = "SUPER", action = act.SendString(tmux_prefix .. "5") },
	{ key = "6", mods = "SUPER", action = act.SendString(tmux_prefix .. "6") },
	{ key = "7", mods = "SUPER", action = act.SendString(tmux_prefix .. "7") },
	{ key = "8", mods = "SUPER", action = act.SendString(tmux_prefix .. "8") },
	{ key = "9", mods = "SUPER", action = act.SendString(tmux_prefix .. "9") },
	-- tmux-fzf (super+shift+f -> prefix+F)
	{ key = "f", mods = "SUPER|SHIFT", action = act.SendString(tmux_prefix .. "F") },
	-- Ditch session (super+alt+x -> prefix+X)
	{ key = "x", mods = "SUPER|ALT", action = act.SendString(tmux_prefix .. "X") },
	-- }}}

	-- {{{ tmux relay: CSI sequences
	-- Ctrl+Tab → toggle last session
	{ key = "Tab", mods = "CTRL", action = act.SendString(csi("60~")) },
	-- Cmd+; → attention session
	{ key = ";", mods = "SUPER", action = act.SendString(csi("61~")) },
	-- Cmd+N → new project session
	{ key = "n", mods = "SUPER", action = act.SendString(csi("62~")) },
	-- Cmd+P → session chooser
	{ key = "p", mods = "SUPER", action = wezterm.action_callback(cmd_p_handler) },
	-- Cmd+Shift+N → session next
	{ key = "n", mods = "SUPER|SHIFT", action = act.SendString(csi("64~")) },
	-- Cmd+Shift+P → session prev
	{ key = "p", mods = "SUPER|SHIFT", action = act.SendString(csi("65~")) },
	-- Cmd+Shift+, → session move up
	{ key = ",", mods = "SUPER|SHIFT", action = act.SendString(csi("66~")) },
	-- Cmd+Shift+. → session move down
	{ key = ".", mods = "SUPER|SHIFT", action = act.SendString(csi("67~")) },
	-- Cmd+Ctrl+N → new worktree
	{ key = "n", mods = "SUPER|CTRL", action = act.SendString(csi("68~")) },
	-- Ctrl+Alt+[ → copy mode
	{ key = "[", mods = "CTRL|ALT", action = act.SendString(csi("69~")) },
	-- }}}

	-- {{{ vim relay: CSI sequences
	{ key = "j", mods = "SUPER", action = act.SendString(csi("90;1~")) },
	{ key = "s", mods = "SUPER", action = act.SendString(csi("90;2~")) },
	{ key = "c", mods = "SUPER|SHIFT", action = act.SendString(csi("90;3~")) },
	{ key = "c", mods = "SUPER|ALT|SHIFT", action = act.SendString(csi("90;4~")) },
	{ key = ".", mods = "SUPER", action = act.SendString(csi("90;6~")) },
	{ key = "e", mods = "SUPER", action = act.SendString(csi("90;7~")) },
	{ key = "b", mods = "SUPER", action = act.SendString(csi("90;8~")) },
	{ key = "i", mods = "SUPER", action = act.SendString(csi("90;9~")) },
	{ key = "l", mods = "SUPER", action = act.SendString(csi("90;10~")) },
	{ key = "i", mods = "SUPER|SHIFT", action = act.SendString(csi("90;11~")) },
	{ key = "k", mods = "SUPER", action = act.SendString(csi("90;12~")) },
	{ key = "v", mods = "SUPER|ALT", action = act.SendString(csi("90;13~")) },
	-- multicursor
	{ key = "d", mods = "SUPER", action = act.SendString(csi("90;14~")) },
	{ key = "d", mods = "SUPER|SHIFT", action = act.SendString(csi("90;15~")) },
	{ key = "u", mods = "SUPER", action = act.SendString(csi("90;16~")) },
	{ key = "u", mods = "SUPER|SHIFT", action = act.SendString(csi("90;17~")) },
	{ key = "k", mods = "SUPER|ALT", action = act.SendString(csi("90;18~")) },
	{ key = "j", mods = "SUPER|ALT", action = act.SendString(csi("90;19~")) },
	{ key = "k", mods = "SUPER|ALT|SHIFT", action = act.SendString(csi("90;20~")) },
	{ key = "j", mods = "SUPER|ALT|SHIFT", action = act.SendString(csi("90;21~")) },
	-- }}}
}

return config
