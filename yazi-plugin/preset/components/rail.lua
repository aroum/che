Rail = {
	_id = "rail",
}

function Rail:new(chunks, tab, active)
	local me = setmetatable({ _chunks = chunks, _tab = tab, _active = active ~= false }, { __index = self })
	me:build()
	return me
end

function Rail:style(style)
	local s = ui.Style():patch(style)
	if not self._active then
		s:patch(ui.Style():dim(false))
	end
	return s
end

function Rail:build()
	self._base = {
		ui.Bar(ui.Edge.RIGHT):area(self._chunks[1]):symbol(th.mgr.border_symbol):style(self:style(th.mgr.border_style)),
		ui.Bar(ui.Edge.LEFT):area(self._chunks[3]):symbol(th.mgr.border_symbol):style(self:style(th.mgr.border_style)),
	}
	self._children = {
		Marker:new(self._chunks[1], self._tab.parent, self._active),
		Marker:new(self._chunks[2], self._tab.current, self._active),
	}
end

function Rail:reflow() return {} end

function Rail:redraw()
	local elements = self._base or {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

-- Mouse events
function Rail:click(event, up) end

function Rail:scroll(event, step) end

function Rail:touch(event, step) end
