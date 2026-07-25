local M = {}

local function log(fmt, ...)
	local f = io.open("/tmp/che_audio_debug.log", "a")
	if f then
		f:write(string.format("[%s] " .. fmt .. "\n", os.date("%H:%M:%S"), ...))
		f:close()
	end
end

---@param lines string[]
---@param key string
---@param value nil|string|table
local function add_line(lines, key, value)
	if value == nil or value == "" or value == "None" or value == "Unknown" then
		return
	end

	if type(value) == "table" then
		value = table.concat(value, ", ")
	end

	table.insert(lines, string.format("%s: %s", key, value))
end

---@param items table
---@return string?
local function first(items)
	for _, item in pairs(items) do
		if item ~= nil and item ~= "" then
			return item
		end
	end
end

---@param file File
---@return string[]
local audio_exiftool = function(file)
	log("audio_exiftool starting for file: %s (path: %s)", tostring(file.url), tostring(file.path))
	local output, err = Command("exiftool"):arg({
		"-j",
		"-a",
		"-s",
		tostring(file.path),
	}):output()

	if not output then
		log("audio_exiftool Command output is nil, error: %s", tostring(err))
		return { "Failed to start `exiftool`, error: " .. tostring(err) }
	end

	log("audio_exiftool success status: %s, exit code: %s", tostring(output.status.success), tostring(output.status.code))
	log("audio_exiftool stdout len: %d, stderr: %s", #output.stdout, tostring(output.stderr))

	local json = ya.json_decode(output.stdout)
	if not json then
		log("audio_exiftool JSON decoding failed")
		return { "Failed to decode `exiftool` output: " .. tostring(output.stdout) }
	elseif type(json) ~= "table" then
		log("audio_exiftool JSON output is not a table")
		return { "Invalid `exiftool` output: " .. tostring(output.stdout) }
	end

	local tags = json[1] or {}
	local data = {}
	local artist = tags.Artist or "[unknown]"
	if type(artist) == "table" then
		artist = table.concat(artist, ", ")
	end
	local date = first { tags.Originaldate, tags.Date, tags.DateTimeOriginal }
	local cover = tags.PictureType or ""
	if tags.PictureWidth and tags.PictureHeight then
		cover = string.format("%s %sx%s", cover, tags.PictureWidth, tags.PictureHeight)
	end

	table.insert(data, artist .. " - " .. (tags.Title or "[untitled]"))
	add_line(data, "Album", tags.Album)
	add_line(data, "Genre", tags.Genre)
	add_line(data, "Date", date)
	add_line(data, "Cover", cover)

	local sr = first { tags.AudioSampleRate, tags.SampleRate }
	local bd = first { tags.AudioBitsPerSample, tags.BitsPerSample }
	if sr then
		sr = string.format("%.1f kHz", tonumber(sr) / 1000)
	end

	table.insert(data, "")
	table.insert(data, "# spec")
	add_line(data, "Duration", tags.Duration)
	add_line(data, "Format", first { tags.AudioFormat, tags.FileType })
	add_line(data, "Sample Rate", sr)
	add_line(data, "Bit Depth", (bd and (bd .. " bit")))
	add_line(data, "BitRate", first { tags.AvgBitrate, tags.AudioBitrate })
	add_line(data, "Channels", first { tags.AudioChannels, tags.ChannelMode, tags.Channels })

	log("audio_exiftool collected %d data lines", #data)
	return data
end

---@param job Job
function M:peek(job)
	log("peek called for file: %s", tostring(job.file.url))
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache then
		log("peek: no file cache slot available")
		return
	end

	log("peek: calling preload")
	self:preload(job)

	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))
	log("peek: showing image from cache: %s", tostring(cache))
	local img_area, err = ya.image_show(cache, job.area)
	if err then
		log("peek: ya.image_show error: %s", tostring(err))
	else
		log("peek: ya.image_show success, area: %s", img_area and tostring(img_area.h) or "nil")
	end

	local img_height = (img_area and img_area.h or 0)

	local info_lines = audio_exiftool(job.file)
	local ui_lines = {}
	for _, line in ipairs(info_lines) do
		table.insert(ui_lines, ui.Line(line))
	end

	log("peek: rendering text area y offset: %d", img_height)
	ya.preview_widget(job, {
		ui.Text(ui_lines):area(ui.Rect {
			x = job.area.x,
			y = job.area.y + img_height,
			w = job.area.w,
			h = job.area.h - img_height,
		}),
	})
	log("peek finished")
end

function M:seek() end

---@param job Job
---@return boolean, Error?
function M:preload(job)
	log("preload called for file: %s", tostring(job.file.url))
	local cache = ya.file_cache(job)
	if not cache then
		log("preload: no cache slot available")
		return true
	end

	local cha = fs.cha(cache)
	if cha and cha.len > 0 then
		log("preload: cache file already exists, size: %d", cha.len)
		return true
	end

	log("preload: invoking ffmpeg")
  -- stylua: ignore
  local output, err = Command('ffmpeg')
      :arg({
        '-hide_banner',
        '-loglevel', 'warning',
        '-i', tostring(job.file.path),
        '-frames:v', '1',
        '-an',
        string.format('%s.jpg', cache),
      })
      :stderr(Command.PIPED)
      :output()

	if not output then
		log("preload: ffmpeg failed to start, error: %s", tostring(err))
		return true, Err("Failed to start `ffmpeg`, error: %s", err)
	elseif not output.status.success then
		log("preload: ffmpeg failed, exit code: %s, stderr: %s", tostring(output.status.code), tostring(output.stderr))
		return true
	end

	log("preload: ffmpeg succeeded, renaming %s.jpg to %s", tostring(cache), tostring(cache))
	local ok, rename_err = fs.rename(Url(string.format("%s.jpg", cache)), cache)
	if not ok then
		log("preload: rename failed, error: %s", tostring(rename_err))
		return true, Err("Failed to rename: %s", rename_err)
	end

	log("preload: finished successfully")
	return true
end

return M
