# Public file lifecycle and native archive lock conformance.
require 'tmpdir'
require_relative '../../ruby/revault_api'
ENV['LOCKBOX_LOCK_TIMEOUT_MS'] = '60'
api = Revault.load
signer = api.generate_profile_signing_key_pair
key = 'K' * 32
check = ->(condition) { raise 'file facade assertion failed' unless condition }
refuses = ->(&action) { begin; action.call; rescue Revault::RevaultError; next; end; raise 'operation should have been refused' }
if (archive = ENV['REVAULT_READONLY_ARCHIVE'])
  box = Revault::Lockbox.open(archive, content_key: key)
  begin; check.call(box.get_file('/hello') == 'read only'); ensure; box.close; end
else
  Dir.mktmpdir('revault-ruby-') do |root|
    archive = File.join(root, 'résumé.lbox')
    create = ->(overwrite = false) { Revault::Lockbox.create(archive, content_key: key, signing_key: signer, overwrite: overwrite) }
    open_box = ->(write = false) { Revault::Lockbox.open(archive, content_key: key, signing_key: write ? signer : nil) }
    payload = "hello\0\xff".b
    box = create.call
    begin; box.add_file('/hello', payload, false); box.commit; box.commit; ensure; box.close; end
    reader = open_box.call
    begin
      box = open_box.call
      begin; check.call(box.get_file('/hello') == payload); ensure; box.close; end
      refuses.call { reader.set_owner_signing_key(signer) }
      refuses.call { reader.add_file('/bad', payload, false) }
      refuses.call { open_box.call(true) }
      refuses.call { create.call(true) }
    ensure; reader.close; end
    box = open_box.call(true)
    begin; box.add_file('/hello', 'replacement', true); box.add_file('/added', payload, false); box.commit; ensure; box.close; end
    box = open_box.call
    begin; check.call(box.get_file('/hello') == 'replacement'); check.call(box.get_file('/added') == payload); ensure; box.close; end
    box = open_box.call(true)
    begin; box.delete('/hello'); box.commit; ensure; box.close; end
    box = open_box.call
    begin; check.call(!box.exists('/hello')); check.call(box.get_file('/added') == payload); ensure; box.close; end
    refuses.call { create.call }
    box = create.call(true)
    begin; box.add_file('/new', payload, false); box.commit; ensure; box.close; end
    box = open_box.call
    begin; check.call(!box.exists('/added')); check.call(box.get_file('/new') == payload); ensure; box.close; end
    check.call(Dir.children(root) == ['résumé.lbox'])
    box = Revault::Lockbox.create(archive, password: 'test password', signing_key: signer, overwrite: true)
    begin; box.add_file('/hello', payload, false); box.commit; ensure; box.close; end
    box = Revault::Lockbox.open(archive, password: 'test password')
    begin; check.call(box.get_file('/hello') == payload); ensure; box.close; end
  end
end
signer.free
puts "PASS\truby\tlockbox_file\t#{ENV['REVAULT_READONLY_ARCHIVE'] ? 1 : 18}"
