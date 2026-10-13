--- AR-lua: Lua router
---
--- ## 1. Contract
---
--- Routes requests from Lua.
local function route()
  -- Implements §FS-entry.1.
  -- See §FS-gone-lua.
  local one = "data §api/FS-str-lua"
  local many = [[first line
data §api/FS-multi-lua
]]
  return one, many
end
return route
