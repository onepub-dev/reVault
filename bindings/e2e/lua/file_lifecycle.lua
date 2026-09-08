-- Public file lifecycle and native archive lock conformance.
local revault = require('revault_api')
local runtime = revault.Revault.load()
local Lockbox = revault.Lockbox
local signer = runtime:generate_profile_signing_key_pair()
local key = string.rep('K', 32)
local function refuses(action)
  local ok, message = pcall(action)
  assert(not ok, 'operation should have been refused')
  assert(type(message) == 'string')
end
local readonly = os.getenv('REVAULT_READONLY_ARCHIVE')
if readonly then
  local box = Lockbox.open(readonly, {content_key=key})
  assert(box:get_file('/hello') == 'read only')
  box:close()
else
  local temporary = os.tmpname()
  os.remove(temporary)
  local path = temporary .. '.lbox'
  local function create(overwrite) return Lockbox.create(path, {content_key=key, signing_key=signer, overwrite=overwrite}) end
  local function open(write) return Lockbox.open(path, {content_key=key, signing_key=write and signer or nil}) end
  local payload = 'hello\0\255'
  local box = create()
  box:add_file('/hello', payload, false); box:commit(); box:commit(); box:close()
  local reader = open()
  box = open(); assert(box:get_file('/hello') == payload); box:close()
  refuses(function() reader:set_owner_signing_key(signer) end)
  refuses(function() reader:add_file('/bad', payload, false) end)
  refuses(function() open(true) end)
  refuses(function() create(true) end)
  reader:close()
  box = open(true); box:add_file('/hello', 'replacement', true); box:add_file('/added', payload, false); box:commit(); box:close()
  box = open(); assert(box:get_file('/hello') == 'replacement'); assert(box:get_file('/added') == payload); box:close()
  box = open(true); box:delete('/hello'); box:commit(); box:close()
  box = open(); assert(not box:exists('/hello')); assert(box:get_file('/added') == payload); box:close()
  refuses(function() create() end)
  box = create(true); box:add_file('/new', payload, false); box:commit(); box:close()
  box = open(); assert(not box:exists('/added')); assert(box:get_file('/new') == payload); box:close()
  box = Lockbox.create(path, {password='test password', signing_key=signer, overwrite=true}); box:add_file('/hello', payload, false); box:commit(); box:close()
  box = Lockbox.open(path, {password='test password'}); assert(box:get_file('/hello') == payload); box:close()
  os.remove(path)
end
signer:free()
print('PASS\tlua\tlockbox_file\t' .. (readonly and 1 or 16))
