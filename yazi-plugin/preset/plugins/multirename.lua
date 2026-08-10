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

local function spawn_ch(bin, args)
	local ok, child = pcall(
		function()
			return Command(bin):arg(args):stdin(Command.INHERIT):stdout(Command.INHERIT):stderr(Command.INHERIT):spawn()
		end
	)
	if ok and child then
		return child
	end
	return nil
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

	local candidates = {}
	local env_path = os.getenv("CHE_CH_PATH") or os.getenv("GEZI_GEZ_PATH")
	if env_path and env_path ~= "" then
		table.insert(candidates, env_path)
	end
	table.insert(candidates, "ch")
	table.insert(candidates, "/opt/homebrew/bin/ch")
	table.insert(candidates, "/usr/local/bin/ch")
	local home = os.getenv("HOME")
	if home then
		table.insert(candidates, home .. "/.cargo/bin/ch")
	end

	local child = nil
	for _, bin in ipairs(candidates) do
		child = spawn_ch(bin, args)
		if child then
			break
		end
	end

	if not child then
		permit:drop()
		return ya.notify {
			title = "Multi-Rename",
			content = "Failed to start rename tool ('ch' binary not found in PATH or Homebrew)",
			timeout = 5,
			level = "error",
		}
	end

	local status, wait_err = child:wait()
	permit:drop()

	if not status then
		os.remove(result_path)
		return ya.notify {
			title = "Multi-Rename",
			content = "Error waiting for tool: " .. tostring(wait_err),
			timeout = 5,
			level = "error",
		}
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
