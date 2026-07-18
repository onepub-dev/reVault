# frozen_string_literal: true
require 'fileutils'
require 'tmpdir'
require 'revault_api'

API = Revault::Vault.new
def pass(symbol, assertions = 1) = puts("PASS\truby\t#{symbol}\t#{assertions}")
def check(value, message)
  raise(message) unless value
end
def artifact_root
  path = File.join(ENV.fetch('REVAULT_E2E_ARTIFACT_DIR', '/tmp/revault-e2e-artifacts'), 'ruby')
  FileUtils.mkdir_p(path, mode: 0o700); path
end
def fields
  Revault::Bindings::FormFieldList.new(values: [Revault::Bindings::FormField.new(
    id: 'username', label: 'Username', kind: 'text', required: true)]).to_proto
end
def moves(source, destination)
  Revault::Bindings::PathMoveList.new(values: [
    Revault::Bindings::PathMove.new(source:, destination:)
  ]).to_proto
end

def archive_lifecycle
  key = 'K' * 32
  box = API.lockbox_create(key); pass('lockbox_create')
  box.add_file('/hello.txt', 'hello from ruby conformance', false); pass('lockbox_add_file', 2)
  check(box.get_file('/hello.txt') == 'hello from ruby conformance', 'get file'); pass('lockbox_get_file', 3)
  box.add_file_with_permissions('/hello.txt', 'replacement payload', 0o640, true); pass('lockbox_add_file_with_permissions', 2)
  check(box.permissions('/hello.txt') == 0o640, 'permissions'); pass('lockbox_permissions')
  box.create_dir('/tree', true); pass('lockbox_create_dir', 2)
  check(box.is_dir('/tree'), 'directory'); pass('lockbox_is_dir')
  box.create_parent_dirs('/tree/a/b/file'); pass('lockbox_create_parent_dirs', 2)
  box.rename('/hello.txt', '/renamed.txt'); pass('lockbox_rename', 3)
  check(box.exists('/renamed.txt') && !box.exists('/hello.txt'), 'exists'); pass('lockbox_exists', 2)
  box.set_permissions('/renamed.txt', 0o600); pass('lockbox_set_permissions', 2)
  check(box.read_range('/renamed.txt', 0, 11) == 'replacement', 'range'); pass('lockbox_read_range', 3)
  box.set_variable('normal', 'value', false); pass('lockbox_set_variable')
  check(box.get_variable('normal') == 'value', 'variable'); pass('lockbox_get_variable', 3)
  box.move_variables(moves('normal', 'moved')); check(box.get_variable('moved') == 'value', 'moved variable')
  box.move_variables(moves('moved', 'normal')); pass('lockbox_move_variables', 3)
  box.set_variable('secret', 'hidden', true); box.variable_sensitivity('secret'); pass('lockbox_variable_sensitivity', 2)
  check(box.list_variables.values.length == 2, 'variables'); pass('lockbox_list_variables')
  box.delete_variable('normal'); pass('lockbox_delete_variable')
  box.add_symlink('/link', '/renamed.txt', false); pass('lockbox_add_symlink')
  check(box.get_symlink_target('/link') == '/renamed.txt', 'symlink'); pass('lockbox_get_symlink_target', 3)
  check(!box.list('/', true).entries.empty?, 'list'); box.stat('/renamed.txt'); pass('lockbox_list', 2); pass('lockbox_stat', 2)
  box.set_workload_profile('read-mostly'); box.set_worker_policy('single', 1); box.runtime_options
  pass('lockbox_set_workload_profile'); pass('lockbox_set_worker_policy'); pass('lockbox_runtime_options')
  box.commit; pass('lockbox_commit'); check(box.storage_len > 0, 'storage'); pass('lockbox_storage_len')
  archive = box.to_bytes; pass('lockbox_to_bytes', 2); pass('buffer_free')
  format = API.lockbox_format_version; check(format > 0 && API.lockbox_probe_format_version(archive) == format, 'format probe')
  pass('lockbox_format_version', 2); pass('lockbox_probe_format_version', 2)
  check(API.lockbox_probe_format_version('bad').zero? && !API.last_error_details.message.empty?, 'error details'); pass('buffer_last_error_details', 2)
  path = File.join(artifact_root, 'archive.lbox'); File.binwrite(path, archive)
  puts("ARTIFACT\truby\tarchive-created\t#{path}"); box.free
  opened = API.lockbox_open(archive, key); pass('lockbox_open', 2)
  check(opened.get_file('/renamed.txt') == 'replacement payload', 'opened')
  puts("ARTIFACT\truby\tarchive-opened\t#{path}")
  opened.delete('/renamed.txt'); pass('lockbox_delete', 2)
  opened.remove_dir('/tree', true); pass('lockbox_remove_dir', 2); opened.free; pass('lockbox_free', 2)
end

def key_lifecycle
  content = (0...32).to_a.pack('C*')
  contact = API.key_contact_generate; pass('key_contact_generate')
  private_key = contact.private; pass('key_contact_private', 2)
  copy = API.key_contact_from_private(private_key); pass('key_contact_from_private')
  public_bytes = contact.public; pass('key_contact_public', 2)
  public_key = API.key_contact_public_from_bytes(public_bytes); pass('key_contact_public_from_bytes')
  wrapped = public_key.encrypt(content); pass('key_contact_encrypt')
  check(copy.decrypt(wrapped) == content, 'decrypt'); pass('key_contact_decrypt', 3)
  check(!wrapped.public.empty? && !wrapped.ciphertext.empty? && !wrapped.encrypted.empty?, 'wrapped')
  pass('key_contact_wrapped_public', 2); pass('key_contact_wrapped_ciphertext', 2); pass('key_contact_wrapped_encrypted', 2)
  wrapped.free; pass('key_contact_wrapped_free')
  imported_private = API.vault_key_import_private(API.vault_key_export_private(contact, 'raw-hex'))
  pass('vault_key_export_private', 2); pass('vault_key_import_private')
  imported_public = API.vault_key_import_public(API.vault_key_export_public(public_key, 'lockbox-pem'))
  pass('vault_key_export_public', 2); pass('vault_key_import_public')
  fingerprint = API.vault_key_fingerprint(imported_public); pass('vault_key_fingerprint', 2)
  hex = API.vault_key_format_hex(fingerprint); check(API.vault_key_decode_hex(hex) == fingerprint, 'hex')
  pass('vault_key_format_hex', 2); pass('vault_key_decode_hex', 2)
  short = fingerprint[0, 12]; code = API.vault_key_format_crockford(short)
  API.vault_key_format_crockford_reading(code); check(API.vault_key_decode_crockford(code) == short, 'crockford')
  pass('vault_key_format_crockford', 2); pass('vault_key_format_crockford_reading', 2); pass('vault_key_decode_crockford', 2)
  imported_public.public_free; public_key.public_free; pass('key_contact_public_free', 2)
  imported_private.free; copy.free; contact.free; pass('key_contact_free', 3)
  plain = API.vault_key_hex_encode(content); check(API.vault_key_hex_decode(plain) == content, 'plain hex')
  pass('vault_key_hex_encode', 2); pass('vault_key_hex_decode', 2)
  signing = API.key_signing_generate; pass('key_signing_generate')
  signing_private = signing.private; signing_public = signing.public
  pass('key_signing_private', 2); pass('key_signing_public', 2)
  API.key_signing_from_private(signing_private).free; pass('key_signing_from_private')
  API.key_signing_public_from_bytes(signing_public).public_free
  pass('key_signing_public_from_bytes'); pass('key_signing_public_free', 2)
  signing.free; pass('key_signing_free', 3)
end

def advanced_archive
  key = 'A' * 32
  box = API.lockbox_create_with_options(key, 'bytes', 4 << 20, 'bulk-import', 'single', 1); pass('lockbox_create_with_options')
  box.add_file('/account.txt', 'account data', false)
  box.list_with_options('/', '*.txt', true, true, false, false, 20); pass('lockbox_list_with_options', 2)
  definition = box.define_form('account', 'Account', 'Account form', fields); pass('lockbox_define_form', 2)
  box.list_form_definitions; box.resolve_form('account'); box.list_form_revisions(definition.type_id)
  pass('lockbox_list_form_definitions'); pass('lockbox_resolve_form'); pass('lockbox_list_form_revisions')
  box.create_form_record('/account.form', 'account', 'Primary'); pass('lockbox_create_form_record')
  box.set_form_field('/account.form', 'username', 'alice', false); pass('lockbox_set_form_field')
  box.get_form_record('/account.form'); box.get_form_field('/account.form', 'username'); box.list_form_records
  pass('lockbox_get_form_record'); pass('lockbox_get_form_field'); pass('lockbox_list_form_records')
  box.move_form_records(moves('/account.form', '/moved.form')); box.get_form_record('/moved.form')
  box.move_form_records(moves('/moved.form', '/account.form')); pass('lockbox_move_form_records', 3)
  signing = API.key_signing_generate; contact = API.key_contact_generate
  public_key = API.key_contact_public_from_bytes(contact.public)
  box.set_owner_signing_key(signing); pass('lockbox_set_owner_signing_key')
  slot = box.add_password('archive password'); pass('lockbox_add_password')
  box.add_contact(public_key, 'recipient'); pass('lockbox_add_contact'); box.list_key_slots; pass('lockbox_list_key_slots')
  box.delete_key(slot); pass('lockbox_delete_key'); box.commit; box.owner_inspection; pass('lockbox_owner_inspection', 2)
  box.cache_stats; box.import_stats; box.reset_import_stats; box.page_inspection; box.recovery_report
  box.recovery_report_render(true, 100); box.stream_content(false); box.id
  %w[lockbox_cache_stats lockbox_import_stats lockbox_reset_import_stats lockbox_page_inspection lockbox_recovery_report].each { pass(_1) }
  pass('lockbox_recovery_report_render', 2); pass('lockbox_stream_content'); pass('lockbox_id', 2)
  archive = box.to_bytes; path = File.join(artifact_root, 'advanced.lbox'); File.binwrite(path, archive)
  API.lockbox_inspect_file(path); API.lockbox_recovery_scan_path(path, key); API.lockbox_recovery_scan(archive, key)
  pass('lockbox_inspect_file'); pass('lockbox_recovery_scan_path'); pass('lockbox_recovery_scan')
  API.lockbox_recovery_salvage(archive[0...-32], key, signing).free; pass('lockbox_recovery_salvage', 2)
  API.lockbox_open_with_options(archive, key, 'bytes', 4 << 20, 'bulk-import', 'single', 1).free; pass('lockbox_open_with_options', 2)
  password_box = API.lockbox_create_password('archive password'); password_box.add_file('/password.txt', 'password protected', false); password_box.commit
  password_archive = password_box.to_bytes; password_box.free; pass('lockbox_create_password')
  password_open = API.lockbox_open_password(password_archive, 'archive password'); password_open.get_file('/password.txt'); password_open.free; pass('lockbox_open_password', 2)
  contact_box = API.lockbox_create_contact(public_key); contact_box.add_file('/contact.txt', 'contact protected', false); contact_box.commit
  contact_archive = contact_box.to_bytes; contact_box.free; pass('lockbox_create_contact')
  contact_open = API.lockbox_open_contact(contact_archive, contact); contact_open.get_file('/contact.txt'); contact_open.free; pass('lockbox_open_contact', 2)
  signed = API.lockbox_create_with_signing_key(key, signing); signed.commit; signed.free; pass('lockbox_create_with_signing_key', 2)
  extract = Dir.mktmpdir('revault-ruby-extract-'); box.extract_file('/account.txt', File.join(extract, 'account.txt'), false); pass('lockbox_extract_file', 2)
  tree = File.join(extract, 'tree'); Dir.mkdir(tree); box.extract_directory(tree, 1 << 20, 4 << 20, 100, false, true, false); pass('lockbox_extract_directory', 2)
  box.delete_form_record('/account.form'); pass('lockbox_delete_form_record')
  box.free; public_key.public_free; contact.free; signing.free
end

def vault_lifecycle
  root = File.join(artifact_root, 'vault'); FileUtils.mkdir_p(root)
  password = 'vault password'; changed = 'new vault password'; id = (0xa0..0xaf).to_a.pack('C*')
  profile = API.key_contact_generate; contact = API.key_contact_generate
  contact_public = API.key_contact_public_from_bytes(contact.public)
  owner = API.key_signing_generate; owner_public = API.key_signing_public_from_bytes(owner.public)
  vault = API.vault_directory_replace(root, password); pass('vault_directory_replace')
  puts("ARTIFACT\truby\tvault-created\t#{root}")
  check(vault.root == root && vault.structure_version > 0, 'vault'); pass('vault_directory_root', 3); pass('vault_directory_structure_version')
  current = API.vault_structure_version_current
  check(current == vault.structure_version && API.vault_directory_probe_structure_version(root, password) == current, 'vault probe')
  pass('vault_structure_version_current', 2); pass('vault_directory_probe_structure_version', 2)
  vault.store_private_key('alice', profile); pass('vault_directory_store_private_key')
  vault.private_key_exists('alice'); pass('vault_directory_private_key_exists')
  vault.load_private_key('alice').free; vault.load_private_key_generation('alice', 1).free
  pass('vault_directory_load_private_key'); pass('vault_directory_load_private_key_generation')
  vault.store_profile_email('alice', 'alice@example.test'); vault.profile_email('alice')
  pass('vault_directory_store_profile_email'); pass('vault_directory_profile_email', 3)
  vault.list_profile_generations('alice'); vault.rotate_private_key('alice')
  pass('vault_directory_list_profile_generations'); pass('vault_directory_rotate_private_key')
  vault.load_owner_signing_key('alice').free; vault.load_owner_signing_key_generation('alice', 1).free
  pass('vault_directory_load_owner_signing_key'); pass('vault_directory_load_owner_signing_key_generation')
  vault.store_contact('bob', contact_public); vault.contact_exists('bob'); vault.load_contact('bob').public_free; vault.list_contacts
  pass('vault_directory_store_contact'); pass('vault_directory_contact_exists'); pass('vault_directory_load_contact'); pass('vault_directory_list_contacts')
  vault.store_contact_signing_key('bob', owner_public); vault.load_contact_signing_key('bob').public_free
  pass('vault_directory_store_contact_signing_key'); pass('vault_directory_load_contact_signing_key')
  vault.list_private_keys; vault.list_private_key_names; vault.list_contact_names
  pass('vault_directory_list_private_keys'); pass('vault_directory_list_private_key_names'); pass('vault_directory_list_contact_names')
  vault.store_backup(id, 'encrypted backup bytes'); vault.backup_count; check(vault.load_backup(id) == 'encrypted backup bytes', 'backup')
  pass('vault_directory_store_backup'); pass('vault_directory_backup_count'); pass('vault_directory_load_backup', 3)
  vault.remember_lockbox(id, '/tmp/example.lbox'); vault.list_known_lockboxes
  pass('vault_directory_remember_lockbox'); pass('vault_directory_list_known_lockboxes')
  vault.remember_access_slot_label(id, 7, 'primary'); vault.list_access_slot_labels(id); vault.find_access_slot_labels(id, 'primary')
  pass('vault_directory_remember_access_slot_label'); pass('vault_directory_list_access_slot_labels'); pass('vault_directory_find_access_slot_labels')
  vault.remember_password(id, password); vault.remembered_password(id)
  pass('vault_directory_remember_password'); pass('vault_directory_remembered_password', 3)
  vault_form = vault.define_form('login', 'Login', 'Login form', fields); vault.resolve_form('login'); vault.list_forms
  pass('vault_directory_define_form'); pass('vault_directory_resolve_form'); pass('vault_directory_list_forms')
  vault.list_form_revisions(vault_form.type_id); pass('vault_directory_list_form_revisions', 2)
  vault.seed_forms; pass('vault_directory_seed_forms'); vault.list_form_aliases; pass('vault_directory_list_form_aliases')
  vault.forget_access_slot_label(id, 7); vault.forget_lockbox('/tmp/example.lbox'); vault.delete_contact('bob')
  pass('vault_directory_forget_access_slot_label'); pass('vault_directory_forget_lockbox'); pass('vault_directory_delete_contact')
  vault.delete_private_key('alice'); vault.restore_private_key('alice', profile, owner, true)
  pass('vault_directory_delete_private_key', 2); pass('vault_directory_restore_private_key', 2)
  vault.free; pass('vault_directory_free')
  readonly = API.vault_read_only_open(root, password)
  readonly.list_profile_names; readonly.list_contact_names; readonly.list_form_aliases; readonly.list_known_lockboxes
  pass('vault_read_only_open'); pass('vault_read_only_list_profile_names', 2); pass('vault_read_only_list_contact_names')
  pass('vault_read_only_list_form_aliases', 2); pass('vault_read_only_list_known_lockboxes')
  readonly.free; pass('vault_read_only_free')
  API.vault_directory_change_password(root, password, changed); pass('vault_directory_change_password')
  API.vault_directory_open(root, changed).free; pass('vault_directory_open'); puts("ARTIFACT\truby\tvault-opened\t#{root}")
  API.vault_directory_open_or_create(root, changed).free; pass('vault_directory_open_or_create')
  owner_public.public_free; owner.free; contact_public.public_free; contact.free; profile.free
end

def default_vault
  FileUtils.mkdir_p(ENV.fetch('LOCKBOX_VAULT_DIR'))
  API.vault_directory_replace_default('default password').free; pass('vault_directory_replace_default')
  API.vault_read_only_open_default('default password').free; pass('vault_read_only_open_default')
  API.vault_default_directory; API.vault_default_path; pass('vault_default_directory', 3); pass('vault_default_path', 2)
  API.vault_directory_open_or_create_default('default password').free; pass('vault_directory_open_or_create_default')
  API.vault_directory_change_default_password('default password', 'changed default password'); pass('vault_directory_change_default_password')
  backup = File.join(artifact_root, 'default-vault.backup'); FileUtils.rm_f(backup)
  API.vault_backup_default(backup, false); API.vault_restore_default(backup, true)
  pass('vault_backup_default'); pass('vault_restore_default')
end

def platform_store
  platform = API.platform; platform.status; pass('vault_platform_status', 2)
  platform.set_scope('vault'); pass('vault_platform_set_scope')
  platform.disable; platform.disabled; pass('vault_platform_disable'); pass('vault_platform_disabled')
  platform.enable; pass('vault_platform_enable')
  platform.put_password('platform vault password'); check(platform.get_password == 'platform vault password', 'platform')
  pass('vault_platform_put_password'); pass('vault_platform_get_password', 3)
  platform.forget_password; pass('vault_platform_forget_password')
end

def agent_and_local
  [ENV.fetch('LOCKBOX_SESSION_AGENT_DIR'), ENV.fetch('LOCKBOX_VAULT_DIR')].each { FileUtils.mkdir_p(_1, mode: 0o700) }
  directory = API.vault_directory_replace_default('agent vault password'); profile = API.key_contact_generate
  directory.store_private_key('default', profile); profile.free; directory.free
  agent = API.agent; agent.forget_all; pass('vault_forget_all')
  child = Process.spawn(RbConfig.ruby, __FILE__, '--serve-agent',
                        out: File.join(Dir.tmpdir, 'ruby-agent.out'),
                        err: File.join(Dir.tmpdir, 'ruby-agent.err'))
  200.times { break if agent.is_running; sleep 0.05 }
  check(agent.is_running, 'agent'); pass('vault_agent_serve'); pass('vault_is_running')
  agent.start; pass('vault_agent_start')
  agent.verify_transport; pass('vault_agent_verify_transport')
  id = (0xc0..0xcf).to_a.pack('C*'); key = (0x20..0x3f).to_a.pack('C*')
  agent.put(id, key); check(agent.get(id) == key, 'agent key'); agent.list
  pass('vault_agent_put'); pass('vault_agent_get', 3); pass('vault_agent_list')
  agent.put_vault_unlock_key('vault-id', key, 120); agent.get_vault_unlock_key('vault-id'); pass('vault_agent_put_vault_unlock_key'); pass('vault_agent_get_vault_unlock_key', 3)
  owner = API.key_signing_generate; agent.put_owner_signing_key('vault-id', 'alice', owner, 120); agent.get_owner_signing_key('vault-id', 'alice').free
  pass('vault_agent_put_owner_signing_key'); pass('vault_agent_get_owner_signing_key')
  activity = agent.begin_activity('open'); pass('vault_agent_begin_activity'); agent.end_activity(activity); pass('vault_agent_end_activity')
  agent.sleep_support; pass('vault_agent_sleep_support'); API.vault_agent_log_path; API.vault_agent_log_destination
  pass('vault_agent_log_path', 2); pass('vault_agent_log_destination', 2)
  local = API.vault_local; pass('vault_local'); root = Dir.mktmpdir('revault-ruby-local-')
  password_path = File.join(root, 'password.lbox'); box = local.create_lockbox_password(password_path, 'local password')
  box.add_file('/data.txt', 'local vault data', false); box.commit; box.free; pass('vault_create_lockbox_password', 3)
  local.cache_lockbox_password(password_path, 'local password', 120); pass('vault_cache_lockbox_password')
  opened = local.open_lockbox_password(password_path, 'local password'); opened.get_file('/data.txt'); opened.free; pass('vault_open_lockbox_password', 3)
  local.close_lockbox(password_path); pass('vault_close_lockbox')
  content_path = File.join(root, 'content.lbox'); box = local.create_lockbox_content_key(content_path, key, owner)
  box.add_file('/data.txt', 'local vault data', false); box.commit; box.free; pass('vault_create_lockbox_content_key', 3)
  opened = local.open_lockbox_content_key(content_path, key, owner); opened.get_file('/data.txt'); opened.free; pass('vault_open_lockbox_content_key', 3)
  contact = API.key_contact_generate; public_key = API.key_contact_public_from_bytes(contact.public)
  box = local.create_lockbox_contact(File.join(root, 'contact.lbox'), public_key, 'recipient', owner)
  box.add_file('/data.txt', 'local vault data', false); box.commit; box.free; pass('vault_create_lockbox_contact', 3)
  public_key.public_free; contact.free; local.close_all; pass('vault_close_all'); local.free; pass('vault_free'); owner.free
  agent.forget_owner_signing_key('vault-id', 'alice'); agent.forget_vault_unlock_key('vault-id'); agent.forget(id)
  pass('vault_agent_forget_owner_signing_key'); pass('vault_agent_forget_vault_unlock_key'); pass('vault_agent_forget')
  agent.stop; pass('vault_agent_stop'); Process.wait(child); check($?.success?, 'agent child')
end

def interop(producer)
  root = ENV.fetch('REVAULT_E2E_ARTIFACT_DIR', '/tmp/revault-e2e-artifacts')
  box = API.lockbox_open(File.binread(File.join(root, producer, 'archive.lbox')), 'K' * 32)
  check(box.get_file('/renamed.txt') == 'replacement payload', 'foreign archive'); box.free
  vault = API.vault_directory_open(File.join(root, producer, 'vault'), 'new vault password')
  check(vault.structure_version > 0, 'foreign vault'); vault.free
  puts("INTEROP\truby\t#{producer}\tarchive\t3\nINTEROP\truby\t#{producer}\tvault\t2")
end

case ARGV
when ['--serve-agent'] then API.agent.serve
when ['--default'] then default_vault
when ['--platform'] then platform_store
when ['--agent'] then agent_and_local
else
  if ARGV.length == 2 && ARGV[0] == '--interop'
    interop(ARGV[1])
  else
    archive_lifecycle; key_lifecycle; advanced_archive; vault_lifecycle
    API.last_error; pass('buffer_last_error')
  end
end
