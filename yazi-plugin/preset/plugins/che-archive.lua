local M = {}

local state = ya.sync(function()
	local selected = {}
	for _, url in pairs(cx.active.selected) do
		selected[#selected + 1] = tostring(url)
	end
	if #selected == 0 and cx.active.current.hovered then
		selected[#selected + 1] = tostring(cx.active.current.hovered.url)
	end

	local current_dir = tostring(cx.active.current.cwd)
	local opposite_dir = nil
	local other_idx = cx.active_tab == 1 and 2 or 1
	if cx.tabs[other_idx] and cx.tabs[other_idx].current then
		opposite_dir = tostring(cx.tabs[other_idx].current.cwd)
	end

	return selected, current_dir, opposite_dir
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

local function is_cmd_available(cmd)
	local stat_cmd = ya.target_family() == "windows" and string.format("where %s > nul 2>&1", cmd)
		or string.format("command -v %s >/dev/null 2>&1", cmd)
	return os.execute(stat_cmd)
end

function M:entry(job)
	local selected, current_dir, opposite_dir = state()
	if #selected == 0 then
		return ya.notify { title = "Archive", content = "No files selected", timeout = 3, level = "warn" }
	end

	-- Escape visual selection
	ya.emit("escape", { visual = true })

	local result_path = "/tmp/che_archive_result.json"
	os.remove(result_path)

	-- Hide UI for TUI dialog
	local permit = ui.hide()

	local args = { "archive" }
	if job and job.args and job.args[1] then
		table.insert(args, "--mode")
		table.insert(args, job.args[1])
	end
	if current_dir then
		table.insert(args, "--output-dir")
		table.insert(args, current_dir)
	end
	if opposite_dir then
		table.insert(args, "--opposite-dir")
		table.insert(args, opposite_dir)
	end
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
			title = "Archive",
			content = "Failed to start archive tool ('ch' binary not found in PATH)",
			timeout = 5,
			level = "error",
		}
	end

	local status, wait_err = child:wait()
	permit:drop()

	if not status or not status.success then
		os.remove(result_path)
		return
	end

	local rf = io.open(result_path, "r")
	if not rf then
		return
	end
	local result_str = rf:read("*all")
	rf:close()
	os.remove(result_path)

	local plan = ya.json_decode(result_str)
	if not plan then
		return
	end

	if plan.op == "pack" then
		self:execute_pack(plan)
	elseif plan.op == "extract" then
		self:execute_extract(plan)
	end
end

function M:execute_pack(plan)
	local ext = plan.format
	local archive_path = plan.archive_path
	local files = plan.files

	ya.notify {
		title = "Archive",
		content = string.format("Archiving %d item(s) to %s...", #files, ya.quote(archive_path)),
		timeout = 4,
		level = "info",
	}

	local cmd = nil
	local args = {}

	if
		ext == "7z" or (ext == "zip" and (is_cmd_available("7z") or is_cmd_available("7zz") or is_cmd_available("7za")))
	then
		cmd = is_cmd_available("7z") and "7z" or (is_cmd_available("7zz") and "7zz" or "7za")
		table.insert(args, "a")
		if ext == "zip" then
			table.insert(args, "-tzip")
		end
		table.insert(args, "-mx=" .. tostring(plan.level))
		if plan.solid and ext == "7z" then
			table.insert(args, "-ms=on")
		end
		if plan.password and plan.password ~= "" then
			table.insert(args, "-p" .. plan.password)
			if plan.encrypt_header and ext == "7z" then
				table.insert(args, "-mhe=on")
			end
		end
		table.insert(args, archive_path)
		for _, f in ipairs(files) do
			table.insert(args, f)
		end
	elseif ext == "zip" and is_cmd_available("zip") then
		cmd = "zip"
		table.insert(args, "-r")
		table.insert(args, "-" .. tostring(plan.level))
		if plan.password and plan.password ~= "" then
			table.insert(args, "-P")
			table.insert(args, plan.password)
		end
		table.insert(args, archive_path)
		for _, f in ipairs(files) do
			table.insert(args, f)
		end
	else
		-- Tar-based formats
		cmd = "tar"
		table.insert(args, "-caf")
		table.insert(args, archive_path)
		for _, f in ipairs(files) do
			table.insert(args, f)
		end
	end

	local child, spawn_err = Command(cmd):arg(args):spawn()
	if not child then
		return ya.notify {
			title = "Archive",
			content = "Failed to run archiver command: " .. tostring(spawn_err),
			timeout = 5,
			level = "error",
		}
	end

	local status, wait_err = child:wait()
	if not status or not status.success then
		return ya.notify {
			title = "Archive",
			content = "Archiving failed with error: "
				.. tostring(wait_err or "exit code " .. tostring(status and status.code)),
			timeout = 5,
			level = "error",
		}
	end

	-- Delete sources if requested
	if plan.delete_source then
		for _, f in ipairs(files) do
			fs.remove("dir_all", Url(f))
		end
	end

	ya.notify {
		title = "Archive",
		content = string.format("Successfully created %s", ya.quote(archive_path)),
		timeout = 4,
		level = "info",
	}
end

function M:execute_extract(plan)
	local archive_path = plan.archive_path
	local target_dir = plan.target_dir

	-- Ensure target directory exists
	fs.create("dir_all", Url(target_dir))

	ya.notify {
		title = "Archive",
		content = string.format("Extracting %s to %s...", ya.quote(archive_path), ya.quote(target_dir)),
		timeout = 4,
		level = "info",
	}

	local cmd = nil
	local args = {}

	if is_cmd_available("7z") or is_cmd_available("7zz") or is_cmd_available("7za") then
		cmd = is_cmd_available("7z") and "7z" or (is_cmd_available("7zz") and "7zz" or "7za")
		table.insert(args, "x")
		table.insert(args, "-o" .. target_dir)
		if plan.overwrite then
			table.insert(args, "-aoa")
		else
			table.insert(args, "-aos")
		end
		if plan.password and plan.password ~= "" then
			table.insert(args, "-p" .. plan.password)
		else
			table.insert(args, "-p-")
		end
		table.insert(args, "-y")
		table.insert(args, archive_path)
	elseif is_cmd_available("unzip") and archive_path:lower():match("%.zip$") then
		cmd = "unzip"
		if plan.overwrite then
			table.insert(args, "-o")
		else
			table.insert(args, "-n")
		end
		if plan.password and plan.password ~= "" then
			table.insert(args, "-P")
			table.insert(args, plan.password)
		end
		table.insert(args, "-d")
		table.insert(args, target_dir)
		table.insert(args, archive_path)
	else
		cmd = "tar"
		table.insert(args, "-xf")
		table.insert(args, archive_path)
		table.insert(args, "-C")
		table.insert(args, target_dir)
	end

	local child, spawn_err = Command(cmd):arg(args):spawn()
	if not child then
		return ya.notify {
			title = "Archive",
			content = "Failed to run extraction command: " .. tostring(spawn_err),
			timeout = 5,
			level = "error",
		}
	end

	local status, wait_err = child:wait()
	if not status or not status.success then
		-- If password wasn't provided, prompt user for password and retry
		if not plan.password or plan.password == "" then
			local pwd, event = ya.input {
				title = "Enter archive password:",
				obscure = true,
				pos = { "top-center", y = 3, w = 40 },
			}
			if event == 1 and pwd and pwd ~= "" then
				plan.password = pwd
				return self:execute_extract(plan)
			end
		end

		return ya.notify {
			title = "Archive",
			content = "Extraction failed with error: "
				.. tostring(wait_err or "exit code " .. tostring(status and status.code)),
			timeout = 5,
			level = "error",
		}
	end

	-- Delete archive if requested
	if plan.delete_source then
		fs.remove("file", Url(archive_path))
	end

	ya.notify {
		title = "Archive",
		content = string.format("Successfully extracted to %s", ya.quote(target_dir)),
		timeout = 4,
		level = "info",
	}
end

return M
