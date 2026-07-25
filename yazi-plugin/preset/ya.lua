function Err(s, ...) return Error.custom(string.format(s, ...)) end

function ya.clamp(min, x, max)
	if x < min then
		return min
	elseif x > max then
		return max
	else
		return x
	end
end

function ya.list_merge(a, b)
	for _, v in ipairs(b) do
		a[#a + 1] = v
	end
	return a
end

function ya.dict_merge(a, b)
	for k, v in pairs(b) do
		a[k] = v
	end
	return a
end

function ya.readable_size(size)
	local units = { "B", "K", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q" }
	local i = 1
	while size > 1024 and i < #units do
		size = size / 1024
		i = i + 1
	end
	local s = string.format("%.1f%s", size, units[i]):gsub("[.,]0", "", 1)
	return s
end

function ya.readable_path(path)
	local home = os.getenv("HOME") or os.getenv("USERPROFILE")
	if not home then
		return path
	elseif path:sub(1, #home) == home then
		return "~" .. path:sub(#home + 1)
	else
		return path
	end
end

function ya.child_at(pos, children)
	for i = #children, 1, -1 do
		if children[i]._area:contains(pos) then
			return children[i]
		end
	end
end

function ya.attach_state_metatable(t)
	if type(t) ~= "table" then
		return t
	end
	local mt = getmetatable(t) or {}
	local orig_index = mt.__index
	local orig_newindex = mt.__newindex

	mt.__newindex = function(tbl, k, v)
		if orig_newindex then
			orig_newindex(tbl, k, v)
		else
			rawset(tbl, k, v)
		end

		if k ~= "_cwd_cache" and k ~= "_id" then
			local cwd = rawget(tbl, "cwd")
			if cwd then
				local cwd_str = tostring(cwd)
				local cache = rawget(tbl, "_cwd_cache")
				if not cache then
					cache = {}
					rawset(tbl, "_cwd_cache", cache)
				end
				cache[cwd_str] = cache[cwd_str] or {}
				cache[cwd_str][k] = v
			end
		end
	end

	mt.__index = function(tbl, k)
		if Header and Header._current_rendering and k ~= "_cwd_cache" and k ~= "_id" then
			local rendering_cwd = tostring(Header._current_rendering._current.cwd)
			local cache = rawget(tbl, "_cwd_cache")
			if cache and cache[rendering_cwd] and cache[rendering_cwd][k] ~= nil then
				return cache[rendering_cwd][k]
			end
		end

		if type(orig_index) == "function" then
			return orig_index(tbl, k)
		elseif type(orig_index) == "table" then
			return orig_index[k]
		else
			return rawget(tbl, k)
		end
	end

	setmetatable(t, mt)
	return t
end
