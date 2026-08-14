-- Get current session across both panes in dual-pane layout
local _get_current_session = ya.sync(function(state)
	local tabs = cx.tabs

	local session = {
		active_pane = tabs.idx,
		single_pane = tabs.single_pane,
		panes = {},
	}

	for p_idx = 1, #tabs do
		local pane = tabs:pane(p_idx)
		if pane then
			local p_data = {
				active_idx = pane.idx,
				tabs = {},
			}
			for t_idx = 1, #pane do
				local tab = pane[t_idx]
				if tab and tab.current and tab.current.cwd then
					p_data.tabs[t_idx] = {
						cwd = tostring(tab.current.cwd):gsub("\\", "/"),
						sort = {
							by = tab.pref.sort_by,
							sensitive = tab.pref.sort_sensitive,
							reverse = tab.pref.sort_reverse,
							dir_first = tab.pref.sort_dir_first,
							translit = tab.pref.sort_translit,
						},
						linemode = tab.pref.linemode,
						show_hidden = tab.pref.show_hidden and "show" or "hide",
					}
				end
			end
			session.panes[p_idx] = p_data
		end
	end

	return session
end)

-- Save session via DDS pub_to
local _save_session = ya.sync(function(state)
	if not state.enabled then
		return
	end
	local session = _get_current_session()
	ps.pub_to(0, state.event, session)
end)

-- Save session and quit
local _save_and_quit = ya.sync(function(state)
	if state.enabled then
		local session = _get_current_session()
		ps.pub_to(0, state.event, session)
	end
	ya.emit("quit", {})
end)

-- Restore session on startup across both panes
local _restore_session = ya.sync(function(state)
	if not state.enabled or state.restored then
		return
	end

	local session = state.session
	if not session then
		return
	end

	-- Backward-compatibility with legacy single-pane session payloads
	local panes = session.panes
	if not panes and session.tabs then
		panes = {
			[1] = {
				active_idx = session.active_idx or 1,
				tabs = session.tabs,
			},
		}
	end

	if not panes or #panes == 0 then
		state.restored = true
		return
	end

	-- Restore Pane 1 (left pane)
	if panes[1] and panes[1].tabs and #panes[1].tabs > 0 then
		ya.emit("pane_focus", { left = true })
		for idx, tab in ipairs(panes[1].tabs) do
			if idx == 1 then
				ya.emit("cd", { tab.cwd })
			else
				ya.emit("tab_create", { tab.cwd })
			end
			if tab.sort then
				ya.emit("sort", tab.sort)
			end
			if tab.linemode then
				ya.emit("linemode", { tab.linemode })
			end
			if tab.show_hidden then
				ya.emit("hidden", { tab.show_hidden })
			end
		end
		if panes[1].active_idx then
			ya.emit("tab_switch", { panes[1].active_idx - 1 })
		end
	end

	-- Restore Pane 2 (right pane)
	if panes[2] and panes[2].tabs and #panes[2].tabs > 0 then
		ya.emit("pane_focus", { left = false })
		for idx, tab in ipairs(panes[2].tabs) do
			if idx == 1 then
				ya.emit("cd", { tab.cwd })
			else
				ya.emit("tab_create", { tab.cwd })
			end
			if tab.sort then
				ya.emit("sort", tab.sort)
			end
			if tab.linemode then
				ya.emit("linemode", { tab.linemode })
			end
			if tab.show_hidden then
				ya.emit("hidden", { tab.show_hidden })
			end
		end
		if panes[2].active_idx then
			ya.emit("tab_switch", { panes[2].active_idx - 1 })
		end
	end

	-- Restore single_pane mode if specified
	if session.single_pane ~= nil and session.single_pane ~= cx.tabs.single_pane then
		ya.emit("pane_only", {})
	end

	-- Restore active pane focus
	if session.active_pane == 1 then
		ya.emit("pane_focus", { left = true })
	elseif session.active_pane == 2 then
		ya.emit("pane_focus", { left = false })
	end

	state.restored = true
end)

local function setup(state, opts)
	opts = opts or {}
	state.enabled = opts.enabled ~= false
	state.restored = false
	state.event = opts.event or "@autosession-event"

	if opts.sync_yanked then
		ps.sub_remote("@yank", function(opt) ya.emit("update_yanked", { opt = opt }) end)
	end

	if not state.enabled then
		return
	end

	local sub_callback = function(body)
		if not state.restored and state.enabled then
			state.session = body
			_restore_session()
		end
	end

	ps.sub_remote(state.event, sub_callback)
	if state.event ~= "@session" then
		ps.sub_remote("@session", sub_callback)
	end
end

return {
	setup = setup,
	entry = function(_, job)
		local action = job.args[1]
		if action == "save-and-quit" then
			_save_and_quit()
		elseif action == "save" then
			_save_session()
		elseif action == "restore" then
			_restore_session()
		end
	end,
}
