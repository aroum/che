Current = {
	_id = "current",
}

function Current:new(area, tab, active)
	local me = setmetatable({
		_area = area,
		_tab = tab,
		_folder = tab.current,
		_active = active ~= false,
	}, { __index = self })
	if active == false then
		me._id = "current_inactive"
	end
	return me
end

function Current:dim(element)
	if self._active then
		return element
	end
	return element:dim(false)
end

function Current:empty()
	local s
	if self._folder.files.filter then
		s = "No filter results"
	else
		local done, err = self._folder.stage()
		s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
	end

	return {
		self:dim(ui.Text(s):area(self._area):align(ui.Align.CENTER):wrap(ui.Wrap.YES)),
	}
end

function Current:reflow() return { self } end

function Current:redraw()
	local files = self._folder.window
	if #files == 0 then
		return self:empty()
	end

	local left, right = {}, {}
	for _, f in ipairs(files) do
		if f.is_upparent then
			local upparent_style = ui.Style():fg("blue")
			if not self._active then
				upparent_style = upparent_style:dim(true)
			end
			if f.is_hovered and self._active then
				upparent_style = upparent_style:patch(th.indicator.current)
			end
			local label = ui.Line { ui.Span("↑ .."):style(upparent_style) }
			left[#left + 1] = label
			right[#right + 1] = ui.Line {}
		else
			local entity = Entity:new(f, self._active)
			left[#left + 1], right[#right + 1] =
				self:dim(entity:redraw()), self:dim(Linemode:new(f):redraw())

			local max = math.max(0, self._area.w - right[#right]:width())
			left[#left]:truncate { max = max, ellipsis = entity:ellipsis(max) }
		end
	end

	return {
		ui.List(left):area(self._area),
		ui.Text(right):area(self._area):align(ui.Align.RIGHT),
	}
end

-- Mouse events
local _last_click_url = nil
local _last_click_time = 0
local DOUBLE_CLICK_MS = 400

function Current:click(event, up)
	if up or event.is_middle or event.is_right then
		return
	end

	local y = event.y - self._area.y + 1
	local file = self._folder.window[y]
	if not file then
		return
	end

	-- Switch focus to this pane if it's not active
	if not self._active then
		ya.emit("pane_switch", {})
	end

	if file.is_upparent then
		ya.emit("leave", {})
		return
	end

	if event.is_ctrl then
		-- Ctrl+click: toggle selection
		ya.emit("toggle", { file.url })
	else
		-- Move cursor to clicked file
		ya.emit("reveal", { file.url })

		-- Double-click detection
		local now = math.floor(ya.time() * 1000)
		local same_file = _last_click_url and tostring(_last_click_url) == tostring(file.url)
		if same_file and (now - _last_click_time) < DOUBLE_CLICK_MS then
			ya.emit("open", {})
			_last_click_url = nil
			_last_click_time = 0
		else
			_last_click_url = file.url
			_last_click_time = now
		end
	end
end

function Current:scroll(event, step) ya.emit("arrow", { step }) end

function Current:touch(event, step) end
