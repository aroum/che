-- ==============================================================================
-- che-bookmarks.yazi — Fast directory bookmarks and hops for che file manager
-- Inspired by bunny.yazi (Copyright (c) 2024 Stel Clementine, MIT License)
-- ==============================================================================

local function fail(s, ...)
	ya.notify { title = "che-bookmarks", content = string.format(s, ...), timeout = 4, level = "error" }
end

local function info(s, ...)
	ya.notify { title = "che-bookmarks", content = string.format(s, ...), timeout = 2, level = "info" }
end

local function warn(s, ...)
	ya.notify { title = "che-bookmarks", content = string.format(s, ...), timeout = 3, level = "warn" }
end

local get_state = ya.sync(function(state, attr) return state[attr] end)

local set_state = ya.sync(function(state, attr, value) state[attr] = value end)

local get_cwd = ya.sync(function(_) return tostring(cx.active.current.cwd) end)

local get_current_tab_idx = ya.sync(function(_)
	local pane_idx = cx.tabs.idx
	local pane = cx.tabs:pane(pane_idx)
	return pane and pane.idx or 1
end)

local get_tabs_as_paths = ya.sync(function(_)
	local pane_idx = cx.tabs.idx
	local pane = cx.tabs:pane(pane_idx)
	if not pane then
		return {}
	end

	local active_tab_idx = pane.idx
	local result = {}
	for idx = 1, #pane do
		if idx ~= active_tab_idx and pane[idx] then
			result[idx] = tostring(pane[idx].current.cwd)
		end
	end
	return result
end)

local function path_to_desc(path, strategy)
	if strategy == "filename" then
		if path == "/" then
			return "/"
		end
		local url_name = Url(path):name()
		return url_name and tostring(url_name) or path
	end

	local home = os.getenv("HOME")
	if home and home ~= "" then
		local start_pos, end_pos = string.find(path, home, 1, true)
		if start_pos == 1 then
			return "~" .. path:sub(end_pos + 1)
		end
	end
	return tostring(path)
end

local function normalize_path(path)
	if string.sub(path, 1, 1) == "~" then
		local home = os.getenv("HOME") or ""
		return home .. path:sub(2)
	end
	return path
end

local function key_to_cands(key)
	if type(key) == "table" then
		return key
	elseif type(key) == "string" then
		if utf8.len(key) == 1 then
			return key
		end
		-- Split multi-character string into table of characters for which-key sequence
		local chars = {}
		for _, code in utf8.codes(key) do
			table.insert(chars, utf8.char(code))
		end
		return chars
	end
	return tostring(key)
end

local function key_to_string(key)
	if type(key) == "table" then
		return table.concat(key, "")
	end
	return tostring(key)
end

local function sort_hops(hops)
	local function convert_key(key)
		local t = type(key)
		if t == "table" then
			return table.concat(key, "")
		elseif t == "string" and string.lower(key) ~= key then
			return "z" .. key
		end
		return tostring(key)
	end

	table.sort(hops, function(x, y) return convert_key(x.key) < convert_key(y.key) end)
	return hops
end

local function get_persist_file_path(config)
	if config.persist_file and config.persist_file ~= "" then
		return normalize_path(config.persist_file)
	end
	local home = os.getenv("HOME") or os.getenv("USERPROFILE")
	if home and home ~= "" then
		local sep = package.config:sub(1, 1) or "/"
		return home .. sep .. ".config" .. sep .. "che" .. sep .. "bookmarks.json"
	end
	local tmp = os.getenv("TMPDIR") or os.getenv("TEMP") or os.getenv("TMP") or "/tmp"
	return tmp .. "/che_bookmarks.json"
end

local function load_persisted_hops(config)
	if not config.persist then
		return {}
	end
	local file_path = get_persist_file_path(config)
	local f = io.open(file_path, "r")
	if not f then
		return {}
	end
	local content = f:read("*a")
	f:close()
	if not content or content == "" then
		return {}
	end

	local ok, data = pcall(ya.json_decode, content)
	if ok and type(data) == "table" then
		local loaded = {}
		for _, item in ipairs(data) do
			if item.key and item.path then
				table.insert(loaded, {
					key = item.key,
					path = normalize_path(item.path),
					desc = item.desc,
					custom = true,
				})
			end
		end
		return loaded
	end
	return {}
end

local function save_persisted_hops(hops, config)
	if not config.persist then
		return
	end
	local to_save = {}
	for _, h in ipairs(hops) do
		if h.custom then
			table.insert(to_save, {
				key = h.key,
				path = h.path,
				desc = h.desc,
			})
		end
	end

	local file_path = get_persist_file_path(config)
	local encoded = ya.json_encode(to_save)
	if encoded then
		-- ensure directory exists
		local dir = file_path:match("(.+)/[^/]+$")
		if dir then
			os.execute(string.format('mkdir -p "%s"', dir))
		end
		local f = io.open(file_path, "w")
		if f then
			f:write(encoded)
			f:close()
		end
	end
end

local create_special_hops = function(config)
	local hops = {}
	local desc_strategy = config.desc_strategy
	if config.ephemeral then
		table.insert(hops, { key = "<Enter>", desc = "Create / add bookmark", path = "__MARK__" })
		table.insert(hops, { key = "<Delete>", desc = "Delete bookmark", path = "__DELETE__" })
	end
	table.insert(hops, { key = "<Space>", desc = "Fuzzy search bookmarks", path = "__FUZZY__" })

	local tabhist = get_state("tabhist") or {}
	local tab = get_current_tab_idx()
	if tabhist[tab] and tabhist[tab][2] then
		local prev_dir = tabhist[tab][2]
		table.insert(hops, {
			key = "<Backspace>",
			path = prev_dir,
			desc = path_to_desc(prev_dir, desc_strategy),
		})
	end

	if config.tabs then
		for idx, tab_path in pairs(get_tabs_as_paths()) do
			table.insert(hops, {
				key = tostring(idx),
				path = tab_path,
				desc = string.format("Tab %d: %s", idx, path_to_desc(tab_path, desc_strategy)),
			})
		end
	end
	return hops
end

local select_fuzzy = function(hops, config)
	local permit = ui.hide()
	local child, spawn_err =
		Command(config.fuzzy_cmd):stdin(Command.PIPED):stdout(Command.PIPED):stderr(Command.INHERIT):spawn()
	if not child then
		fail("Command `%s` failed. Is it installed?", config.fuzzy_cmd)
		return
	end

	local fuzzy_entries = {}
	for _, hop in pairs(hops) do
		local existing = fuzzy_entries[hop.path]
		if not existing or existing == "" then
			local key_str = key_to_string(hop.key)
			local desc = hop.desc or path_to_desc(hop.path, config.desc_strategy)
			if desc == hop.path then
				desc = ""
			end
			fuzzy_entries[hop.path] = string.format("[%s] %s", key_str, desc)
		end
	end

	local input_lines = {}
	for entry_path, entry_desc in pairs(fuzzy_entries) do
		local line = string.format("%-30s\t%s", entry_desc, entry_path)
		table.insert(input_lines, line)
	end

	child:write_all(table.concat(input_lines, "\n"))
	child:flush()
	local output, _ = child:wait_with_output()
	permit:drop()

	if not output or not output.status.success then
		return
	end

	local desc, path = string.match(output.stdout, "^(.-) *\t(.-)\n$")
	if not path or path == "" then
		return
	end
	if not desc or desc == "" then
		desc = path_to_desc(path, config.desc_strategy)
	end
	return { desc = desc, path = path }
end

local cd = function(selected_hop, config)
	local _, dir_list_err = fs.read_dir(Url(selected_hop.path), { limit = 1, resolve = true })
	if dir_list_err then
		fail("Cannot access directory %s", path_to_desc(selected_hop.path))
		return
	end
	ya.emit("cd", { selected_hop.path })
	if config.notify then
		info("Hopped to %s", selected_hop.desc or path_to_desc(selected_hop.path, config.desc_strategy))
	end
end

local create_mark = function(hops, config)
	-- Prompt 1: Bookmark key/prefix (supports single or multi-character like "a", "doc", "work")
	local key_input, key_event = ya.input {
		title = "Bookmark key (e.g. a, doc, proj):",
		pos = { "top-center", y = 2, w = 50 },
	}
	if key_event ~= 1 or not key_input or key_input:match("^%s*$") then
		return
	end
	local key_str = key_input:match("^%s*(.-)%s*$")

	-- Prompt 2: Optional custom description
	local desc_input, desc_event = ya.input {
		title = "Bookmark description (leave empty to show path):",
		pos = { "top-center", y = 2, w = 50 },
	}
	local custom_desc = nil
	if desc_event == 1 and desc_input and not desc_input:match("^%s*$") then
		custom_desc = desc_input:match("^%s*(.-)%s*$")
	end

	local cwd = get_cwd()
	local final_desc = custom_desc or path_to_desc(cwd, config.desc_strategy)

	-- Check if key is already present, update or append
	local found = false
	for i, h in ipairs(hops) do
		if key_to_string(h.key) == key_str then
			h.path = cwd
			h.desc = final_desc
			h.custom = true
			found = true
			break
		end
	end

	if not found then
		table.insert(hops, {
			key = key_str,
			path = cwd,
			desc = final_desc,
			custom = true,
		})
	end

	set_state("hops", sort_hops(hops))
	save_persisted_hops(hops, config)

	info("Bookmark '%s' saved -> %s", key_str, final_desc)
end

local delete_mark = function(hops, config)
	local custom_hops = {}
	for _, hop in pairs(hops) do
		if hop.custom then
			table.insert(custom_hops, hop)
		end
	end

	if #custom_hops == 0 then
		info("No custom bookmarks to delete")
		return
	end

	local cands = {}
	for idx, hop in ipairs(custom_hops) do
		local key_str = key_to_string(hop.key)
		local cand_on = tostring(idx)
		if idx > 9 then
			cand_on = string.char(97 + idx - 10)
		end
		table.insert(cands, {
			on = cand_on,
			desc = string.format("Delete [%s] %s (%s)", key_str, hop.desc, path_to_desc(hop.path, config.desc_strategy)),
			hop_key = key_str,
		})
	end

	info("Select bookmark to delete")
	local hops_idx = ya.which { cands = cands }
	if not hops_idx then
		return
	end
	local selected = cands[hops_idx]

	for i, h in ipairs(hops) do
		if h.custom and key_to_string(h.key) == selected.hop_key then
			table.remove(hops, i)
			break
		end
	end

	set_state("hops", sort_hops(hops))
	save_persisted_hops(hops, config)

	info("Deleted bookmark '%s'", selected.hop_key)
end

local delete_all_marks = function(hops, config)
	local remaining = {}
	for _, h in ipairs(hops) do
		if not h.custom then
			table.insert(remaining, h)
		end
	end
	set_state("hops", sort_hops(remaining))
	save_persisted_hops(remaining, config)
	info("Cleared all custom bookmarks")
end

local attempt_hop = function(hops, config)
	local cands = {}
	for _, hop in pairs(create_special_hops(config)) do
		table.insert(cands, { desc = hop.desc, on = hop.key, path = hop.path })
	end
	for _, hop in pairs(hops) do
		table.insert(cands, {
			desc = hop.desc or path_to_desc(hop.path, config.desc_strategy),
			on = key_to_cands(hop.key),
			path = hop.path,
		})
	end

	local hops_idx = ya.which { cands = cands }
	if not hops_idx then
		return
	end
	local selected_hop = cands[hops_idx]

	if selected_hop.path == "__MARK__" then
		create_mark(hops, config)
		return
	elseif selected_hop.path == "__DELETE__" then
		delete_mark(hops, config)
		return
	elseif selected_hop.path == "__FUZZY__" then
		local fuzzy_hop = select_fuzzy(hops, config)
		if not fuzzy_hop then
			return
		end
		selected_hop = fuzzy_hop
	end

	cd(selected_hop, config)
end

local function init()
	local options = get_state("options") or {}
	local desc_strategy = options.desc_strategy or "path"
	local config = {
		enabled = options.enabled ~= false,
		desc_strategy = desc_strategy,
		fuzzy_cmd = options.fuzzy_cmd or "fzf",
		notify = options.notify or false,
		ephemeral = options.ephemeral ~= false,
		tabs = options.tabs ~= false,
		persist = options.persist ~= false,
		persist_file = options.persist_file,
	}
	set_state("config", config)

	local hops = {}
	if type(options.hops) == "table" then
		for _, hop in pairs(options.hops) do
			local p = normalize_path(hop.path)
			table.insert(hops, {
				key = hop.key,
				path = p,
				desc = hop.desc or path_to_desc(p, desc_strategy),
				custom = false,
			})
		end
	end

	-- Load persisted bookmarks
	local persisted = load_persisted_hops(config)
	for _, p in ipairs(persisted) do
		-- if key doesn't conflict with static hops, insert it
		local found = false
		for _, h in ipairs(hops) do
			if key_to_string(h.key) == key_to_string(p.key) then
				found = true
				break
			end
		end
		if not found then
			table.insert(hops, p)
		end
	end

	set_state("hops", sort_hops(hops))
	set_state("init", true)
end

return {
	setup = function(state, options)
		state.options = options or {}
		ps.sub("cd", function(body)
			local tab = body.tab
			local cwd = tostring(cx.active.current.cwd)
			local tabhist = state.tabhist or {}
			if not tabhist[tab] then
				tabhist[tab] = { cwd }
			else
				tabhist[tab] = { cwd, tabhist[tab][1] }
			end
			state.tabhist = tabhist
		end)
	end,
	entry = function(self, job)
		if not get_state("init") then
			init()
		end

		local config = get_state("config") or { enabled = true }
		if not config.enabled then
			warn("che-bookmarks is disabled in configuration")
			return
		end

		local hops = get_state("hops") or {}
		local action = job.args[1]

		if action == "fuzzy" then
			local fuzzy_hop = select_fuzzy(hops, config)
			if fuzzy_hop then
				cd(fuzzy_hop, config)
			end
		elseif action == "add" or action == "mark" or action == "save" or action == "create" then
			create_mark(hops, config)
		elseif action == "delete" or action == "remove" then
			delete_mark(hops, config)
		elseif action == "delete_all" or action == "clear" then
			delete_all_marks(hops, config)
		else
			attempt_hop(hops, config)
		end
	end,
}
