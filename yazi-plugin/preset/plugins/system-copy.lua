local M = {}

local function log(fmt, ...)
  local f = io.open("/tmp/che_system_copy_debug.log", "a")
  if f then
    f:write(string.format("[%s] " .. fmt .. "\n", os.date("%H:%M:%S"), ...))
    f:close()
  end
end

-- Access cx only inside ya.sync because cx is not available in the async thread
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

function M:entry()
	log("system_copy async entry point called")

	local selected = state()
	log("Selected files count: %d", #selected)
	for i, u in ipairs(selected) do
		log("File %d: %s", i, u)
	end

	if #selected == 0 then
		return ya.notify { title = "System Copy", content = "No files selected", timeout = 3, level = "warn" }
	end

	local os_family = ya.target_family()
	local os_name = ya.target_os()
	log("OS family: %s, OS name: %s", os_family, os_name)
	local success, err

	if os_family == "windows" then
		local paths_list = {}
		for _, url in ipairs(selected) do
			local p = url:gsub("^file://", "")
			table.insert(paths_list, string.format('"%s"', p))
		end
		local script = string.format(
			'Add-Type -AssemblyName System.Windows.Forms; $c = New-Object System.Collections.Specialized.StringCollection; %s; [System.Windows.Forms.Clipboard]::SetFileDropList($c)',
			table.concat(paths_list, "; "):gsub('"', '\\"')
		)
		log("Executing Windows script: %s", script)
		local output
		output, err = Command("powershell")
			:arg({ "-NoProfile", "-NonInteractive", "-Command", script })
			:output()
		success = output and output.status.success
		log("Powershell exit status: %s, err: %s", tostring(success), tostring(err))
	elseif os_name == "macos" then
		local paths = {}
		for _, url in ipairs(selected) do
			local p = url:gsub("^file://", "")
			p = p:gsub("%%(%x%x)", function(h) return string.char(tonumber(h, 16)) end)
			table.insert(paths, string.format('"%s"', p))
		end
		local paths_list = table.concat(paths, ", ")
		
		local osascript_command = string.format([[
use framework "Foundation"
use framework "AppKit"
use scripting additions

set pb to current application's NSPasteboard's generalPasteboard()
pb's clearContents()
pb's setPropertyList:{%s} forType:(current application's NSFilenamesPboardType)
]], paths_list)
		
		log("Executing AppleScript Objective-C via stdin:\n%s", osascript_command)
		
		local child
		child, err = Command("osascript")
			:stdin(Command.PIPED)
			:stdout(Command.PIPED)
			:stderr(Command.PIPED)
			:spawn()
		
		if child then
			child:write_all(osascript_command)
			child:flush()
			local output = child:wait_with_output()
			success = output and output.status.success
			if output then
				log("osascript exit status: %s, code: %s, stderr: %s", tostring(success), tostring(output.status.code), tostring(output.stderr))
			else
				log("osascript wait_with_output returned nil")
			end
		else
			log("osascript failed to spawn, error: %s", tostring(err))
		end
	else
		-- Linux
		local wayland = os.getenv("WAYLAND_DISPLAY") ~= nil
		log("Linux Wayland mode: %s", tostring(wayland))
		local uris = {}
		for _, url in ipairs(selected) do
			if not url:find("^file://") then
				table.insert(uris, "file://" .. url)
			else
				table.insert(uris, url)
			end
		end
		local input_data = table.concat(uris, "\n") .. "\n"
		log("URI list input data:\n%s", input_data)

		if wayland then
			local child
			child, err = Command("wl-copy")
				:arg({ "-t", "text/uri-list" })
				:stdin(Command.PIPED)
				:spawn()
			if child then
				child:write_all(input_data)
				child:flush()
				local output = child:wait_with_output()
				success = output and output.status.success
				log("wl-copy status: %s", tostring(success))
			else
				log("wl-copy spawn failed, error: %s", tostring(err))
			end
		else
			local child
			child, err = Command("xclip")
				:arg({ "-i", "-selection", "clipboard", "-t", "text/uri-list" })
				:stdin(Command.PIPED)
				:spawn()
			if child then
				child:write_all(input_data)
				child:flush()
				local output = child:wait_with_output()
				success = output and output.status.success
				log("xclip status: %s", tostring(success))
			else
				log("xclip spawn failed, error: %s", tostring(err))
			end
		end
	end

	log("system_copy finish status: %s", tostring(success))
	if success then
		ya.notify {
			title = "System Copy",
			content = string.format("Copied %d file(s) to system clipboard", #selected),
			timeout = 3,
			level = "info",
		}
	else
		ya.notify {
			title = "System Copy",
			content = "Failed to copy files: " .. tostring(err or "unknown error"),
			timeout = 5,
			level = "error",
		}
	end
end

return M
