--- @sync peek

local M = {}

function M:peek(job)
	local folder = cx.active.preview.folder
	if not folder then
		return ya.preview_widget(job, ui.Line("Loading..."):area(job.area):align(ui.Align.CENTER))
	elseif folder.cwd ~= job.file.url then
		return
	end

	local limit = math.max(1, job.area.h - 2)
	local bound = math.max(0, #folder.files - limit)
	if job.skip > bound then
		return ya.emit("peek", { bound, only_if = job.file.url, upper_bound = true })
	end

	if #folder.files == 0 then
		local done, err = folder.stage()
		local s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
		return ya.preview_widget(job, ui.Text(s):area(job.area):align(ui.Align.CENTER):wrap(ui.Wrap.YES))
	end

	local files_count = 0
	local dirs_count = 0
	local total_size = 0
	for _, f in ipairs(folder.files) do
		if f.cha.is_dir then
			dirs_count = dirs_count + 1
		else
			files_count = files_count + 1
			total_size = total_size + f.cha.len
		end
	end

	local chunks = ui.Layout()
		:direction(ui.Layout.VERTICAL)
		:constraints({
			ui.Constraint.Length(1),
			ui.Constraint.Length(1),
			ui.Constraint.Fill(1),
		})
		:split(job.area)

	local left, right = {}, {}
	for _, f in ipairs(folder.window) do
		local entity = Entity:new(f)
		left[#left + 1], right[#right + 1] = entity:redraw(), Linemode:new(f):redraw()

		local max = math.max(0, chunks[3].w - right[#right]:width())
		left[#left]:truncate { max = max, ellipsis = entity:ellipsis(max) }
	end

	local info_str = string.format("Files: %d, Dirs: %d, Size: %s", files_count, dirs_count, ya.readable_size(total_size))
	local info_text = ui.Text(info_str):style(ui.Style():fg("cyan"))
	local sep_line = ui.Line(string.rep("─", job.area.w)):style(ui.Style():fg("gray"))

	ya.preview_widget(job, {
		info_text:area(chunks[1]),
		sep_line:area(chunks[2]),
		ui.List(left):area(chunks[3]),
		ui.Text(right):area(chunks[3]):align(ui.Align.RIGHT),
		table.unpack(Marker:new(chunks[3], folder):redraw()),
	})
end

function M:seek(job)
	local folder = cx.active.preview.folder
	if folder and folder.cwd == job.file.url then
		local limit = math.max(1, job.area.h - 2)
		local step = math.floor(job.units * limit / 10)
		local bound = math.max(0, #folder.files - limit)
		ya.emit("peek", {
			ya.clamp(0, cx.active.preview.skip + step, bound),
			only_if = job.file.url,
		})
	end
end

function M:spot(job)
	local i, url = 0, job.file.url
	for rows in self:spot_base(job) do
		i, rows[#rows + 1] = i + 1, ui.Row {}
		ya.spot_table(
			job,
			ui.Table(ya.list_merge(rows, require("file"):spot_base(job)))
				:area(ui.Pos { "center", w = 60, h = 20 })
				:row(i == 1 and 1 or nil)
				:col(1)
				:col_style(th.spot.tbl_col)
				:cell_style(th.spot.tbl_cell)
				:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
		)
	end
	if self.size then
		ya.emit("update_files", { op = fs.op("size", { url = url.parent, sizes = { [url.urn] = self.size } }) })
	end
end

function M:spot_base(job)
	local function yield(s)
		coroutine.yield {
			ui.Row({ "Folder" }):style(ui.Style():fg("green")),
			ui.Row { "  Size:", s },
		}
	end

	self.size = nil
	return ya.co(function()
		yield("0B (?)")

		local it, size, last = fs.calc_size(job.file.url), 0, 0
		if not it then
			return yield("Error")
		end

		while true do
			local next, now = it:recv(), ya.time()
			if not next then
				break
			elseif now >= last + 0.1 then
				last, size = now, size + next
				yield(ya.readable_size(size) .. " (?)")
			else
				size = size + next
			end
		end

		self.size = size
		yield(ya.readable_size(size))
	end)
end

return M
