DualPane = {
	_id = "dual_pane",
}

function DualPane:new(area)
	local me = setmetatable({ _area = area }, { __index = self })
	me:layout()
	me:build()
	return me
end

function DualPane:layout()
	self._chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(1, 2),
			ui.Constraint.Length(1),
			ui.Constraint.Ratio(1, 2),
		})
		:split(self._area)

	local left_pane = cx.tabs:pane(1)
	local right_pane = cx.tabs:pane(2)

	if not cx.tabs.preview_pane or cx.tabs.idx == 1 then
		self._left_chunks = ui.Layout()
			:direction(ui.Layout.VERTICAL)
			:constraints({
				ui.Constraint.Length(1),
				ui.Constraint.Length(Tabs.height(left_pane)),
				ui.Constraint.Fill(1),
				ui.Constraint.Length(1),
			})
			:split(self._chunks[1])
	end
	if not cx.tabs.preview_pane or cx.tabs.idx == 2 then
		self._right_chunks = ui.Layout()
			:direction(ui.Layout.VERTICAL)
			:constraints({
				ui.Constraint.Length(1),
				ui.Constraint.Length(Tabs.height(right_pane)),
				ui.Constraint.Fill(1),
				ui.Constraint.Length(1),
			})
			:split(self._chunks[3])
	end
end

function DualPane:build()
	self._base = {
		ui.Bar(ui.Edge.LEFT)
			:area(self._chunks[2])
			:symbol("│")
			:style(ui.Style():patch(th.mgr.border_style)),
	}

	if cx.tabs.preview_pane then
		local active_idx = cx.tabs.idx
		local ratio_no_preview = { parent = 0, current = 1, preview = 0, all = 1 }

		self._children = {}
		if active_idx == 1 then
			-- Left is active, Right is preview of Left
			table.insert(self._children, Header:new(self._left_chunks[1], cx.tabs[1]))
			table.insert(self._children, Tabs:new(self._left_chunks[2], cx.tabs:pane(1), 1))
			table.insert(self._children, Tab:new(self._left_chunks[3], cx.tabs[1], true, ratio_no_preview))
			table.insert(self._children, Status:new(self._left_chunks[4], cx.tabs[1]))

			table.insert(self._children, Preview:new(self._chunks[3], cx.tabs[1], true))
		else
			-- Right is active, Left is preview of Right
			table.insert(self._children, Preview:new(self._chunks[1], cx.tabs[2], true))

			table.insert(self._children, Header:new(self._right_chunks[1], cx.tabs[2]))
			table.insert(self._children, Tabs:new(self._right_chunks[2], cx.tabs:pane(2), 2))
			table.insert(self._children, Tab:new(self._right_chunks[3], cx.tabs[2], true, ratio_no_preview))
			table.insert(self._children, Status:new(self._right_chunks[4], cx.tabs[2]))
		end
	else
		local ratio = { parent = 0, current = 1, preview = 0, all = 1 }
		self._children = {
			Header:new(self._left_chunks[1], cx.tabs[1]),
			Tabs:new(self._left_chunks[2], cx.tabs:pane(1), 1),
			Tab:new(self._left_chunks[3], cx.tabs[1], cx.tabs.idx == 1, ratio),
			Status:new(self._left_chunks[4], cx.tabs[1]),
			Header:new(self._right_chunks[1], cx.tabs[2]),
			Tabs:new(self._right_chunks[2], cx.tabs:pane(2), 2),
			Tab:new(self._right_chunks[3], cx.tabs[2], cx.tabs.idx == 2, ratio),
			Status:new(self._right_chunks[4], cx.tabs[2]),
		}
	end
end

function DualPane:reflow()
	local components = { self }
	for _, child in ipairs(self._children) do
		components = ya.list_merge(components, child:reflow())
	end
	return components
end

function DualPane:redraw()
	local elements = self._base or {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

function DualPane:click(event, up) end

function DualPane:scroll(event, step) end

function DualPane:touch(event, step) end
