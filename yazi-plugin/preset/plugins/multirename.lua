local M = {}

local state = ya.sync(function()
	local selected = {}
	for _, url in pairs(cx.active.selected) do
		selected[#selected + 1] = tostring(url)
	end
	if #selected == 0 and cx.active.current.hovered then
		selected[#selected + 1] = tostring(cx.active.current.hovered.url)
	end
	return selected
end)

local function find_gez()
	-- che sets this at startup so plugins always know the sibling binary path
	local path = os.getenv("CHE_CH_PATH") or os.getenv("GEZI_GEZ_PATH")
	if path and path ~= "" then
		return path
	end
	return "ch"
end

function M:entry()
	local selected = state()
	if #selected == 0 then
		return ya.notify { title = "Multi-Rename", content = "No files selected", timeout = 3, level = "warn" }
	end

	-- Escape visual selection
	ya.emit("escape", { visual = true })

	-- Clean old result file if exists
	local result_path = "/tmp/che_multirename_result.json"
	os.remove(result_path)

	-- Hide Yazi UI to let the child process use the terminal
	local permit = ui.hide()

	local args = { "multirename" }
	for _, path in ipairs(selected) do
		table.insert(args, path)
	end

	local gez_bin = find_gez()
	local child, err = Command(gez_bin)
		:arg(args)
		:stdin(Command.INHERIT)
		:stdout(Command.INHERIT)
		:stderr(Command.INHERIT)
		:spawn()

	if not child then
		permit:drop()
		return ya.notify { title = "Multi-Rename", content = "Failed to start rename tool: " .. tostring(err), timeout = 5, level = "error" }
	end

	local status, wait_err = child:wait()
	permit:drop()

	if not status then
		os.remove(result_path)
		return ya.notify { title = "Multi-Rename", content = "Error waiting for tool: " .. tostring(wait_err), timeout = 5, level = "error" }
	end

	if not status.success then
		os.remove(result_path)
		if status.code ~= 130 and status.code ~= 0 then
			return ya.notify { title = "Multi-Rename", content = "Tool exited with error", timeout = 5, level = "error" }
		end
		return
	end

	-- Read result file
	local rf = io.open(result_path, "r")
	if not rf then
		return
	end
	local result_str = rf:read("*all")
	rf:close()
	os.remove(result_path)

	local rename_map = ya.json_decode(result_str)
	if not rename_map then
		return ya.notify { title = "Multi-Rename", content = "Failed to parse rename results", timeout = 5, level = "error" }
	end

	-- Execute renaming
	local count = 0
	for _, mapping in ipairs(rename_map) do
		if mapping.old ~= mapping.new then
			if fs.rename(Url(mapping.old), Url(mapping.new)) then
				count = count + 1
			end
		end
	end

	if count > 0 then
		ya.notify {
			title = "Multi-Rename",
			content = string.format("Successfully renamed %d file(s)", count),
			timeout = 3,
			level = "info",
		}
	end
end

return M
