"""Owned, class-oriented reVault API.

Use :class:`Vault` as the entry point and close owned handles with a context
manager. Secret access is callback-scoped. The repository README contains the
security model and examples: https://github.com/onepub-dev/reVault#readme
"""
from __future__ import annotations
import ctypes
from pathlib import Path

from . import _Buffer, _error, load
from ._domain import decode, encode_form_fields, encode_path_moves

def _take(lib, result):
    if not result.ptr: raise RuntimeError(_error(lib))
    try: return ctypes.string_at(result.ptr, result.len)
    finally: lib.buffer_free(result)

def _with_secret(owner, getter, callback):
    handle = ctypes.c_void_p()
    if not getter(ctypes.byref(handle)): raise RuntimeError(_error(owner._lib))
    if not handle.value: return None
    try:
        length = ctypes.c_size_t()
        if not owner._lib.secret_len(handle, ctypes.byref(length)):
            raise RuntimeError(_error(owner._lib))
        native = (ctypes.c_uint8 * max(1, length.value))()
        if not owner._lib.secret_copy(handle, native, length.value):
            raise RuntimeError(_error(owner._lib))
        secret = bytearray(native[:length.value])
        try: return callback(secret)
        finally:
            secret[:] = b'\0' * len(secret)
            ctypes.memset(native, 0, max(1, length.value))
    finally: owner._lib.secret_free(handle)

def _call(owner, symbol, values):
    lib = owner._lib
    route = _ROUTES[symbol]
    if route[0] and route[0][0] == 'handle' and hasattr(owner, '_handle'):
        values = (owner, *values)
    native_args, keepalive = [], []
    for kind, value in zip(route[0], values):
        if kind in ('text', 'bytes'):
            raw = value.encode() if kind == 'text' else bytes(value)
            buffer = ctypes.create_string_buffer(raw, max(1, len(raw)))
            keepalive.append(buffer)
            native_args.extend((buffer, len(raw)))
        elif kind == 'handle':
            native_args.append(value._handle)
        else: native_args.append(value)
    result = getattr(lib, symbol)(*native_args)
    result_kind = route[1]
    if result_kind.startswith('handle:'):
        if not result: raise RuntimeError(_error(lib))
        return globals()[result_kind[7:]](owner._root, result)
    if result_kind == 'bool':
        if not result: raise RuntimeError(_error(lib))
        return None
    if result_kind == 'predicate': return bool(result)
    if result_kind == 'void':
        if route[2] and hasattr(owner, '_handle'): owner._handle = None
        return None
    if result_kind == 'value': return result
    raw = _take(lib, result)
    if result_kind == 'utf8': return raw.decode()
    if result_kind == 'binary': return raw
    return decode(result_kind[8:], raw)

class _OwnedHandle:
    def __init__(self, root, handle): self._root, self._lib, self._handle = root, getattr(root, '_lib', root), ctypes.c_void_p(handle)
    def __enter__(self): return self
    def __exit__(self, *_):
        close = getattr(self, 'free', None)
        if close and self._handle: close()

class Vault:
    """Primary API used to open lockboxes and access reVault's local services.

    Create one when the application starts, then use it to manage keys, metadata,
    the session agent, and operating-system credential storage.
    """
    def __init__(self, path: str | Path | None = None):
        """Load and validate the bundled native library, or the library at path."""
        self._root = self; self._lib = load(path); self.agent = Agent(self, None); self.platform = Platform(self, None)
    @property
    def last_error(self):
        """Return the diagnostic from the most recent native call on this thread."""
        return _error(self._lib)
    def last_error_details(self):
        """Return structured details for the most recent native failure."""
        return _call(self, 'buffer_last_error_details', ())

class Lockbox(_OwnedHandle):
    """An open encrypted archive containing files, variables, secrets, and forms.

    Obtain one from ``Vault`` or ``LocalVault``; commit pending mutations and
    release it when the application finishes using its decrypted contents.
    """
    pass

class ContactKeyPair(_OwnedHandle):
    """A profile's contact-encryption identity, including its private key.

    Distribute the public half and retain this object to decrypt content keys
    addressed to the profile.
    """
    pass

class ContactPublicKey(_OwnedHandle):
    """A recipient's shareable encryption identity.

    Use it when granting that recipient lockbox access; it contains no private
    key material.
    """
    pass

class WrappedContactKey(_OwnedHandle):
    """A content key encrypted for one contact.

    Store or transfer it with an access record; only the matching
    ``ContactKeyPair`` can recover the content key.
    """
    pass

class SigningKeyPair(_OwnedHandle):
    """A lockbox owner's signing identity, including its private key.

    Supply it when creating or committing a mutable lockbox so readers can
    authenticate revisions.
    """
    pass

class SigningPublicKey(_OwnedHandle):
    """The shareable half of a lockbox owner's signing identity.

    Readers use it to verify owner-authorized revisions; it cannot create a
    signature.
    """
    pass

class VaultDirectory(_OwnedHandle):
    """A writable, password-protected metadata store for one installation.

    It keeps profile keys, contacts, forms, backups, and remembered lockbox
    paths; lockbox file contents remain separate.
    """
    pass

class ReadOnlyVaultDirectory(_OwnedHandle):
    """A restricted metadata view for listing profiles, contacts, and lockboxes.

    Use it for discovery screens and diagnostics that must not load owner
    signing keys or change local metadata.
    """
    pass

class Agent(_OwnedHandle):
    """Client for the local session service that caches secrets for a limited time.

    Use it to start or inspect the agent and temporarily retain vault unlock or
    owner signing keys across application operations.
    """
    pass

class AgentActivity(_OwnedHandle):
    """A lifetime token for an operation that currently needs cached secrets.

    Keep it alive during the operation and release it afterward so the agent can
    expire secrets when no other activity needs them.
    """
    pass

class Platform(_OwnedHandle):
    """Access to the operating system credential store used for vault passwords.

    Select an application scope, then enable, save, retrieve, or forget the
    password associated with that scope.
    """
    pass

class LocalVault(_OwnedHandle):
    """A session that manages lockbox files used by a local application.

    Use it to create or open lockboxes by host path, cache short-lived
    passwords, and commit and close the files opened during the session.
    """
    pass

def _Vault_lockbox_format_version(self):
    """Returns the version."""
    return _call(self, 'lockbox_format_version', ())
Vault.lockbox_format_version = _Vault_lockbox_format_version

def _Vault_lockbox_probe_format_version(self, bytes):
    """Returns the version."""
    return _call(self, 'lockbox_probe_format_version', (bytes,))
Vault.lockbox_probe_format_version = _Vault_lockbox_probe_format_version

def _Vault_lockbox_create(self, key):
    """Creates a new lockbox."""
    return _call(self, 'lockbox_create', (key,))
Vault.lockbox_create = _Vault_lockbox_create

def _Vault_lockbox_create_with_options(self, key, cache_mode, cache_bytes, workload, worker, jobs):
    """Create a lockbox with explicit cache capacity, workload, worker policy, and job count."""
    return _call(self, 'lockbox_create_with_options', (key, cache_mode, cache_bytes, workload, worker, jobs))
Vault.lockbox_create_with_options = _Vault_lockbox_create_with_options

def _Vault_lockbox_create_password(self, password):
    """Returns the password."""
    return _call(self, 'lockbox_create_password', (password,))
Vault.lockbox_create_password = _Vault_lockbox_create_password

def _Vault_lockbox_create_contact(self, contact):
    """Returns the contact."""
    return _call(self, 'lockbox_create_contact', (contact,))
Vault.lockbox_create_contact = _Vault_lockbox_create_contact

def _Vault_lockbox_create_with_signing_key(self, content_key, signing_key):
    """Returns the key."""
    return _call(self, 'lockbox_create_with_signing_key', (content_key, signing_key))
Vault.lockbox_create_with_signing_key = _Vault_lockbox_create_with_signing_key

def _Vault_lockbox_open(self, archive, key):
    """Opens an existing lockbox."""
    return _call(self, 'lockbox_open', (archive, key))
Vault.lockbox_open = _Vault_lockbox_open

def _Vault_lockbox_open_with_options(self, archive, key, cache_mode, cache_bytes, workload, worker, jobs):
    """Open a lockbox with explicit cache capacity, workload, worker policy, and job count."""
    return _call(self, 'lockbox_open_with_options', (archive, key, cache_mode, cache_bytes, workload, worker, jobs))
Vault.lockbox_open_with_options = _Vault_lockbox_open_with_options

def _Vault_lockbox_open_password(self, archive, password):
    """Returns the password."""
    return _call(self, 'lockbox_open_password', (archive, password))
Vault.lockbox_open_password = _Vault_lockbox_open_password

def _Vault_lockbox_open_contact(self, archive, contact):
    """Returns the contact."""
    return _call(self, 'lockbox_open_contact', (archive, contact))
Vault.lockbox_open_contact = _Vault_lockbox_open_contact

def _Vault_lockbox_inspect_file(self, path):
    """Returns the file."""
    return _call(self, 'lockbox_inspect_file', (path,))
Vault.lockbox_inspect_file = _Vault_lockbox_inspect_file

def _Vault_lockbox_recovery_scan_path(self, path, key):
    """Returns the path."""
    return _call(self, 'lockbox_recovery_scan_path', (path, key))
Vault.lockbox_recovery_scan_path = _Vault_lockbox_recovery_scan_path

def _Vault_lockbox_recovery_scan(self, bytes, key):
    """Scans scan."""
    return _call(self, 'lockbox_recovery_scan', (bytes, key))
Vault.lockbox_recovery_scan = _Vault_lockbox_recovery_scan

def _Vault_lockbox_recovery_salvage(self, bytes, key, signing_key):
    """Salvages salvage."""
    return _call(self, 'lockbox_recovery_salvage', (bytes, key, signing_key))
Vault.lockbox_recovery_salvage = _Vault_lockbox_recovery_salvage

def _Vault_key_contact_generate(self):
    """Generates generate."""
    return _call(self, 'key_contact_generate', ())
Vault.key_contact_generate = _Vault_key_contact_generate

def _Vault_key_contact_from_private(self, bytes):
    """Returns the private."""
    return _call(self, 'key_contact_from_private', (bytes,))
Vault.key_contact_from_private = _Vault_key_contact_from_private

def _Vault_key_contact_public_from_bytes(self, bytes):
    """Returns the bytes."""
    return _call(self, 'key_contact_public_from_bytes', (bytes,))
Vault.key_contact_public_from_bytes = _Vault_key_contact_public_from_bytes

def _Vault_key_signing_generate(self):
    """Generates generate."""
    return _call(self, 'key_signing_generate', ())
Vault.key_signing_generate = _Vault_key_signing_generate

def _Vault_key_signing_from_private(self, bytes):
    """Returns the private."""
    return _call(self, 'key_signing_from_private', (bytes,))
Vault.key_signing_from_private = _Vault_key_signing_from_private

def _Vault_key_signing_public_from_bytes(self, bytes):
    """Returns the bytes."""
    return _call(self, 'key_signing_public_from_bytes', (bytes,))
Vault.key_signing_public_from_bytes = _Vault_key_signing_public_from_bytes

def _Vault_vault_key_export_private(self, key, format):
    """Returns the private."""
    return _call(self, 'vault_key_export_private', (key, format))
Vault.vault_key_export_private = _Vault_vault_key_export_private

def _Vault_vault_key_export_public(self, key, format):
    """Returns the public."""
    return _call(self, 'vault_key_export_public', (key, format))
Vault.vault_key_export_public = _Vault_vault_key_export_public

def _Vault_vault_key_import_private(self, bytes):
    """Returns the private."""
    return _call(self, 'vault_key_import_private', (bytes,))
Vault.vault_key_import_private = _Vault_vault_key_import_private

def _Vault_vault_key_import_public(self, bytes):
    """Returns the public."""
    return _call(self, 'vault_key_import_public', (bytes,))
Vault.vault_key_import_public = _Vault_vault_key_import_public

def _Vault_vault_key_fingerprint(self, key):
    """Returns the stable fingerprint of this key."""
    return _call(self, 'vault_key_fingerprint', (key,))
Vault.vault_key_fingerprint = _Vault_vault_key_fingerprint

def _Vault_vault_key_format_hex(self, bytes):
    """Returns the hex."""
    return _call(self, 'vault_key_format_hex', (bytes,))
Vault.vault_key_format_hex = _Vault_vault_key_format_hex

def _Vault_vault_key_decode_hex(self, text):
    """Returns the hex."""
    return _call(self, 'vault_key_decode_hex', (text,))
Vault.vault_key_decode_hex = _Vault_vault_key_decode_hex

def _Vault_vault_key_format_crockford(self, bytes):
    """Returns the crockford."""
    return _call(self, 'vault_key_format_crockford', (bytes,))
Vault.vault_key_format_crockford = _Vault_vault_key_format_crockford

def _Vault_vault_key_format_crockford_reading(self, code):
    """Returns the reading."""
    return _call(self, 'vault_key_format_crockford_reading', (code,))
Vault.vault_key_format_crockford_reading = _Vault_vault_key_format_crockford_reading

def _Vault_vault_key_decode_crockford(self, code):
    """Returns the crockford."""
    return _call(self, 'vault_key_decode_crockford', (code,))
Vault.vault_key_decode_crockford = _Vault_vault_key_decode_crockford

def _Vault_vault_key_hex_encode(self, bytes):
    """Encodes encode."""
    return _call(self, 'vault_key_hex_encode', (bytes,))
Vault.vault_key_hex_encode = _Vault_vault_key_hex_encode

def _Vault_vault_key_hex_decode(self, text):
    """Decodes decode."""
    return _call(self, 'vault_key_hex_decode', (text,))
Vault.vault_key_hex_decode = _Vault_vault_key_hex_decode

def _Vault_vault_directory_open(self, root, password):
    """Opens an existing lockbox."""
    return _call(self, 'vault_directory_open', (root, password))
Vault.vault_directory_open = _Vault_vault_directory_open

def _Vault_vault_structure_version_current(self):
    """Returns the current."""
    return _call(self, 'vault_structure_version_current', ())
Vault.vault_structure_version_current = _Vault_vault_structure_version_current

def _Vault_vault_directory_probe_structure_version(self, root, password):
    """Returns the version."""
    return _call(self, 'vault_directory_probe_structure_version', (root, password))
Vault.vault_directory_probe_structure_version = _Vault_vault_directory_probe_structure_version

def _Vault_vault_directory_open_or_create_default(self, password):
    """Returns the default."""
    return _call(self, 'vault_directory_open_or_create_default', (password,))
Vault.vault_directory_open_or_create_default = _Vault_vault_directory_open_or_create_default

def _Vault_vault_directory_replace_default(self, password):
    """Returns the default."""
    return _call(self, 'vault_directory_replace_default', (password,))
Vault.vault_directory_replace_default = _Vault_vault_directory_replace_default

def _Vault_vault_directory_change_password(self, root, old_password, new_password):
    """Returns the password."""
    return _call(self, 'vault_directory_change_password', (root, old_password, new_password))
Vault.vault_directory_change_password = _Vault_vault_directory_change_password

def _Vault_vault_directory_change_default_password(self, old_password, new_password):
    """Returns the password."""
    return _call(self, 'vault_directory_change_default_password', (old_password, new_password))
Vault.vault_directory_change_default_password = _Vault_vault_directory_change_default_password

def _Vault_vault_directory_replace(self, root, password):
    """Updates replace."""
    return _call(self, 'vault_directory_replace', (root, password))
Vault.vault_directory_replace = _Vault_vault_directory_replace

def _Vault_vault_directory_open_or_create(self, root, password):
    """Creates a new lockbox."""
    return _call(self, 'vault_directory_open_or_create', (root, password))
Vault.vault_directory_open_or_create = _Vault_vault_directory_open_or_create

def _Vault_vault_backup_default(self, path, overwrite):
    """Returns the default."""
    return _call(self, 'vault_backup_default', (path, overwrite))
Vault.vault_backup_default = _Vault_vault_backup_default

def _Vault_vault_restore_default(self, path, overwrite):
    """Returns the default."""
    return _call(self, 'vault_restore_default', (path, overwrite))
Vault.vault_restore_default = _Vault_vault_restore_default

def _Vault_vault_read_only_open(self, root, password):
    """Opens an existing lockbox."""
    return _call(self, 'vault_read_only_open', (root, password))
Vault.vault_read_only_open = _Vault_vault_read_only_open

def _Vault_vault_read_only_open_default(self, password):
    """Returns the default."""
    return _call(self, 'vault_read_only_open_default', (password,))
Vault.vault_read_only_open_default = _Vault_vault_read_only_open_default

def _Vault_vault_default_directory(self):
    """Returns the directory."""
    return _call(self, 'vault_default_directory', ())
Vault.vault_default_directory = _Vault_vault_default_directory

def _Vault_vault_default_path(self):
    """Returns the path."""
    return _call(self, 'vault_default_path', ())
Vault.vault_default_path = _Vault_vault_default_path

def _Vault_vault_agent_log_path(self):
    """Returns the path."""
    return _call(self, 'vault_agent_log_path', ())
Vault.vault_agent_log_path = _Vault_vault_agent_log_path

def _Vault_vault_agent_log_destination(self):
    """Returns the destination."""
    return _call(self, 'vault_agent_log_destination', ())
Vault.vault_agent_log_destination = _Vault_vault_agent_log_destination

def _Vault_vault_local(self):
    """Returns the local."""
    return _call(self, 'vault_local', ())
Vault.vault_local = _Vault_vault_local

def _Lockbox_add_file(self, path, data, replace):
    """Returns the file."""
    return _call(self, 'lockbox_add_file', (path, data, replace))
Lockbox.add_file = _Lockbox_add_file

def _Lockbox_add_file_with_permissions(self, path, data, permissions, replace):
    """Returns the permissions."""
    return _call(self, 'lockbox_add_file_with_permissions', (path, data, permissions, replace))
Lockbox.add_file_with_permissions = _Lockbox_add_file_with_permissions

def _Lockbox_get_file(self, path):
    """Returns the file."""
    return _call(self, 'lockbox_get_file', (path,))
Lockbox.get_file = _Lockbox_get_file

def _Lockbox_extract_file(self, source, destination, replace):
    """Returns the file."""
    return _call(self, 'lockbox_extract_file', (source, destination, replace))
Lockbox.extract_file = _Lockbox_extract_file

def _Lockbox_extract_directory(self, destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite):
    """Returns the directory."""
    return _call(self, 'lockbox_extract_directory', (destination, max_file_bytes, max_total_bytes, max_files, restore_symlinks, restore_permissions, overwrite))
Lockbox.extract_directory = _Lockbox_extract_directory

def _Lockbox_stream_content(self, physical):
    """Returns the content."""
    return _call(self, 'lockbox_stream_content', (physical,))
Lockbox.stream_content = _Lockbox_stream_content

def _Lockbox_cache_stats(self):
    """Returns the stats."""
    return _call(self, 'lockbox_cache_stats', ())
Lockbox.cache_stats = _Lockbox_cache_stats

def _Lockbox_import_stats(self):
    """Returns the stats."""
    return _call(self, 'lockbox_import_stats', ())
Lockbox.import_stats = _Lockbox_import_stats

def _Lockbox_reset_import_stats(self):
    """Returns the stats."""
    return _call(self, 'lockbox_reset_import_stats', ())
Lockbox.reset_import_stats = _Lockbox_reset_import_stats

def _Lockbox_page_inspection(self):
    """Returns the inspection."""
    return _call(self, 'lockbox_page_inspection', ())
Lockbox.page_inspection = _Lockbox_page_inspection

def _Lockbox_recovery_report(self):
    """Returns the report."""
    return _call(self, 'lockbox_recovery_report', ())
Lockbox.recovery_report = _Lockbox_recovery_report

def _Lockbox_recovery_report_render(self, verbose, max_entries):
    """Returns the render."""
    return _call(self, 'lockbox_recovery_report_render', (verbose, max_entries))
Lockbox.recovery_report_render = _Lockbox_recovery_report_render

def _Lockbox_storage_len(self):
    """Returns the len."""
    return _call(self, 'lockbox_storage_len', ())
Lockbox.storage_len = _Lockbox_storage_len

def _Lockbox_set_workload_profile(self, profile):
    """Returns the profile."""
    return _call(self, 'lockbox_set_workload_profile', (profile,))
Lockbox.set_workload_profile = _Lockbox_set_workload_profile

def _Lockbox_set_worker_policy(self, mode, jobs):
    """Returns the policy."""
    return _call(self, 'lockbox_set_worker_policy', (mode, jobs))
Lockbox.set_worker_policy = _Lockbox_set_worker_policy

def _Lockbox_runtime_options(self):
    """Returns the options."""
    return _call(self, 'lockbox_runtime_options', ())
Lockbox.runtime_options = _Lockbox_runtime_options

def _Lockbox_commit(self):
    """Authenticates and publishes the staged changes."""
    return _call(self, 'lockbox_commit', ())
Lockbox.commit = _Lockbox_commit

def _Lockbox_create_dir(self, path, create_parents):
    """Returns the dir."""
    return _call(self, 'lockbox_create_dir', (path, create_parents))
Lockbox.create_dir = _Lockbox_create_dir

def _Lockbox_delete(self, path):
    """Removes delete."""
    return _call(self, 'lockbox_delete', (path,))
Lockbox.delete = _Lockbox_delete

def _Lockbox_remove_dir(self, path, recursive):
    """Returns the dir."""
    return _call(self, 'lockbox_remove_dir', (path, recursive))
Lockbox.remove_dir = _Lockbox_remove_dir

def _Lockbox_create_parent_dirs(self, path):
    """Returns the dirs."""
    return _call(self, 'lockbox_create_parent_dirs', (path,))
Lockbox.create_parent_dirs = _Lockbox_create_parent_dirs

def _Lockbox_rename(self, source, destination):
    """Updates rename."""
    return _call(self, 'lockbox_rename', (source, destination))
Lockbox.rename = _Lockbox_rename

def _Lockbox_list(self, path, recursive):
    """Lists list."""
    return _call(self, 'lockbox_list', (path, recursive))
Lockbox.list = _Lockbox_list

def _Lockbox_list_with_options(self, path, glob, recursive, include_files, include_symlinks, include_directories, limit):
    """Returns the options."""
    return _call(self, 'lockbox_list_with_options', (path, glob, recursive, include_files, include_symlinks, include_directories, limit))
Lockbox.list_with_options = _Lockbox_list_with_options

def _Lockbox_stat(self, path):
    """Returns metadata for the selected lockbox entry."""
    return _call(self, 'lockbox_stat', (path,))
Lockbox.stat = _Lockbox_stat

def _Lockbox_set_variable(self, name, value):
    """Returns the variable."""
    return _call(self, 'lockbox_set_variable', (name, value))
Lockbox.set_variable = _Lockbox_set_variable

def _Lockbox_set_secret_variable(self, name, value):
    """Returns the variable."""
    """Store a secret variable from mutable bytes."""
    raw, secret = name.encode(), bytearray(value)
    native = (ctypes.c_uint8 * len(secret)).from_buffer(secret) if secret else None
    try:
        if not self._lib.lockbox_set_secret_variable(self._handle, raw, len(raw), native, len(secret)):
            raise RuntimeError(_error(self._lib))
    finally: secret[:] = b'\0' * len(secret)
Lockbox.set_secret_variable = _Lockbox_set_secret_variable

def _Lockbox_get_variable(self, name):
    """Returns the variable."""
    return _call(self, 'lockbox_get_variable', (name,))
Lockbox.get_variable = _Lockbox_get_variable

def _Lockbox_with_secret_variable(self, name, callback):
    """Returns the variable."""
    """Call ``callback`` with temporary secret bytes, then wipe the transfer copy."""
    raw = name.encode()
    return _with_secret(self, lambda output: self._lib.lockbox_get_secret_variable(self._handle, raw, len(raw), output), callback)
Lockbox.with_secret_variable = _Lockbox_with_secret_variable

def _Lockbox_delete_variable(self, name):
    """Returns the variable."""
    return _call(self, 'lockbox_delete_variable', (name,))
Lockbox.delete_variable = _Lockbox_delete_variable

def _Lockbox_move_variables(self, moves):
    """Returns the variables."""
    return _call(self, 'lockbox_move_variables', (encode_path_moves(moves),))
Lockbox.move_variables = _Lockbox_move_variables

def _Lockbox_list_variables(self):
    """Returns the variables."""
    return _call(self, 'lockbox_list_variables', ())
Lockbox.list_variables = _Lockbox_list_variables

def _Lockbox_variable_sensitivity(self, name):
    """Returns the sensitivity."""
    return _call(self, 'lockbox_variable_sensitivity', (name,))
Lockbox.variable_sensitivity = _Lockbox_variable_sensitivity

def _Lockbox_add_symlink(self, path, target, replace):
    """Returns the symlink."""
    return _call(self, 'lockbox_add_symlink', (path, target, replace))
Lockbox.add_symlink = _Lockbox_add_symlink

def _Lockbox_get_symlink_target(self, path):
    """Returns the target."""
    return _call(self, 'lockbox_get_symlink_target', (path,))
Lockbox.get_symlink_target = _Lockbox_get_symlink_target

def _Lockbox_id(self):
    """Returns the id."""
    return _call(self, 'lockbox_id', ())
Lockbox.id = _Lockbox_id

def _Lockbox_exists(self, path):
    """Reports whether exists."""
    return _call(self, 'lockbox_exists', (path,))
Lockbox.exists = _Lockbox_exists

def _Lockbox_is_dir(self, path):
    """Returns the dir."""
    return _call(self, 'lockbox_is_dir', (path,))
Lockbox.is_dir = _Lockbox_is_dir

def _Lockbox_permissions(self, path):
    """Returns the permissions."""
    return _call(self, 'lockbox_permissions', (path,))
Lockbox.permissions = _Lockbox_permissions

def _Lockbox_set_permissions(self, path, permissions):
    """Returns the permissions."""
    return _call(self, 'lockbox_set_permissions', (path, permissions))
Lockbox.set_permissions = _Lockbox_set_permissions

def _Lockbox_read_range(self, path, offset, len):
    """Returns the range."""
    return _call(self, 'lockbox_read_range', (path, offset, len))
Lockbox.read_range = _Lockbox_read_range

def _Lockbox_add_password(self, password):
    """Returns the password."""
    return _call(self, 'lockbox_add_password', (password,))
Lockbox.add_password = _Lockbox_add_password

def _Lockbox_add_contact(self, contact, name):
    """Returns the contact."""
    return _call(self, 'lockbox_add_contact', (contact, name))
Lockbox.add_contact = _Lockbox_add_contact

def _Lockbox_delete_key(self, id):
    """Returns the key."""
    return _call(self, 'lockbox_delete_key', (id,))
Lockbox.delete_key = _Lockbox_delete_key

def _Lockbox_list_key_slots(self):
    """Returns the slots."""
    return _call(self, 'lockbox_list_key_slots', ())
Lockbox.list_key_slots = _Lockbox_list_key_slots

def _Lockbox_set_owner_signing_key(self, key):
    """Returns the key."""
    return _call(self, 'lockbox_set_owner_signing_key', (key,))
Lockbox.set_owner_signing_key = _Lockbox_set_owner_signing_key

def _Lockbox_owner_inspection(self):
    """Returns the inspection."""
    return _call(self, 'lockbox_owner_inspection', ())
Lockbox.owner_inspection = _Lockbox_owner_inspection

def _Lockbox_define_form(self, alias, name, description, fields):
    """Returns the form."""
    return _call(self, 'lockbox_define_form', (alias, name, description, encode_form_fields(fields)))
Lockbox.define_form = _Lockbox_define_form

def _Lockbox_list_form_definitions(self):
    """Returns the definitions."""
    return _call(self, 'lockbox_list_form_definitions', ())
Lockbox.list_form_definitions = _Lockbox_list_form_definitions

def _Lockbox_resolve_form(self, reference):
    """Returns the form."""
    return _call(self, 'lockbox_resolve_form', (reference,))
Lockbox.resolve_form = _Lockbox_resolve_form

def _Lockbox_list_form_revisions(self, type_id):
    """Returns the revisions."""
    return _call(self, 'lockbox_list_form_revisions', (type_id,))
Lockbox.list_form_revisions = _Lockbox_list_form_revisions

def _Lockbox_create_form_record(self, path, type_reference, name):
    """Returns the record."""
    return _call(self, 'lockbox_create_form_record', (path, type_reference, name))
Lockbox.create_form_record = _Lockbox_create_form_record

def _Lockbox_set_form_field(self, path, field, value):
    """Returns the field."""
    return _call(self, 'lockbox_set_form_field', (path, field, value))
Lockbox.set_form_field = _Lockbox_set_form_field

def _Lockbox_set_secret_form_field(self, path, field, value):
    """Returns the field."""
    """Store a secret form field from mutable bytes."""
    path_raw, field_raw, secret = path.encode(), field.encode(), bytearray(value)
    native = (ctypes.c_uint8 * len(secret)).from_buffer(secret) if secret else None
    try:
        if not self._lib.lockbox_set_secret_form_field(self._handle, path_raw, len(path_raw), field_raw, len(field_raw), native, len(secret)):
            raise RuntimeError(_error(self._lib))
    finally: secret[:] = b'\0' * len(secret)
Lockbox.set_secret_form_field = _Lockbox_set_secret_form_field

def _Lockbox_list_form_records(self):
    """Returns the records."""
    return _call(self, 'lockbox_list_form_records', ())
Lockbox.list_form_records = _Lockbox_list_form_records

def _Lockbox_get_form_record(self, path):
    """Returns the record."""
    return _call(self, 'lockbox_get_form_record', (path,))
Lockbox.get_form_record = _Lockbox_get_form_record

def _Lockbox_delete_form_record(self, path):
    """Returns the record."""
    return _call(self, 'lockbox_delete_form_record', (path,))
Lockbox.delete_form_record = _Lockbox_delete_form_record

def _Lockbox_move_form_records(self, moves):
    """Returns the records."""
    return _call(self, 'lockbox_move_form_records', (encode_path_moves(moves),))
Lockbox.move_form_records = _Lockbox_move_form_records

def _Lockbox_get_form_field(self, path, field):
    """Returns the field."""
    return _call(self, 'lockbox_get_form_field', (path, field))
Lockbox.get_form_field = _Lockbox_get_form_field

def _Lockbox_with_secret_form_field(self, path, field, callback):
    """Returns the field."""
    """Call ``callback`` with temporary field bytes, then wipe the transfer copy."""
    path_raw, field_raw = path.encode(), field.encode()
    return _with_secret(self, lambda output: self._lib.lockbox_get_secret_form_field(self._handle, path_raw, len(path_raw), field_raw, len(field_raw), output), callback)
Lockbox.with_secret_form_field = _Lockbox_with_secret_form_field

def _Lockbox_to_bytes(self):
    """Returns the bytes."""
    return _call(self, 'lockbox_to_bytes', ())
Lockbox.to_bytes = _Lockbox_to_bytes

def _Lockbox_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'lockbox_free', ())
Lockbox.free = _Lockbox_free

def _ContactKeyPair_public(self):
    """Returns the public."""
    return _call(self, 'key_contact_public', ())
ContactKeyPair.public = _ContactKeyPair_public

def _ContactKeyPair_private(self):
    """Returns the private."""
    return _call(self, 'key_contact_private', ())
ContactKeyPair.private = _ContactKeyPair_private

def _ContactKeyPair_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'key_contact_free', ())
ContactKeyPair.free = _ContactKeyPair_free

def _ContactKeyPair_decrypt(self, wrapped):
    """Decrypts a wrapped content key for this contact."""
    return _call(self, 'key_contact_decrypt', (wrapped,))
ContactKeyPair.decrypt = _ContactKeyPair_decrypt

def _ContactPublicKey_public_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'key_contact_public_free', ())
ContactPublicKey.public_free = _ContactPublicKey_public_free

def _ContactPublicKey_encrypt(self, content_key):
    """Encrypts a content key for the selected contact."""
    return _call(self, 'key_contact_encrypt', (content_key,))
ContactPublicKey.encrypt = _ContactPublicKey_encrypt

def _WrappedContactKey_public(self):
    """Returns the public."""
    return _call(self, 'key_contact_wrapped_public', ())
WrappedContactKey.public = _WrappedContactKey_public

def _WrappedContactKey_ciphertext(self):
    """Returns the ciphertext."""
    return _call(self, 'key_contact_wrapped_ciphertext', ())
WrappedContactKey.ciphertext = _WrappedContactKey_ciphertext

def _WrappedContactKey_encrypted(self):
    """Returns the encrypted."""
    return _call(self, 'key_contact_wrapped_encrypted', ())
WrappedContactKey.encrypted = _WrappedContactKey_encrypted

def _WrappedContactKey_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'key_contact_wrapped_free', ())
WrappedContactKey.free = _WrappedContactKey_free

def _SigningKeyPair_public(self):
    """Returns the public."""
    return _call(self, 'key_signing_public', ())
SigningKeyPair.public = _SigningKeyPair_public

def _SigningKeyPair_private(self):
    """Returns the private."""
    return _call(self, 'key_signing_private', ())
SigningKeyPair.private = _SigningKeyPair_private

def _SigningKeyPair_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'key_signing_free', ())
SigningKeyPair.free = _SigningKeyPair_free

def _SigningPublicKey_public_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'key_signing_public_free', ())
SigningPublicKey.public_free = _SigningPublicKey_public_free

def _VaultDirectory_root(self):
    """Returns the root."""
    return _call(self, 'vault_directory_root', ())
VaultDirectory.root = _VaultDirectory_root

def _VaultDirectory_structure_version(self):
    """Returns the version."""
    return _call(self, 'vault_directory_structure_version', ())
VaultDirectory.structure_version = _VaultDirectory_structure_version

def _VaultDirectory_list_private_keys(self):
    """Returns the keys."""
    return _call(self, 'vault_directory_list_private_keys', ())
VaultDirectory.list_private_keys = _VaultDirectory_list_private_keys

def _VaultDirectory_list_private_key_names(self):
    """Returns the names."""
    return _call(self, 'vault_directory_list_private_key_names', ())
VaultDirectory.list_private_key_names = _VaultDirectory_list_private_key_names

def _VaultDirectory_list_contact_names(self):
    """Returns the names."""
    return _call(self, 'vault_directory_list_contact_names', ())
VaultDirectory.list_contact_names = _VaultDirectory_list_contact_names

def _VaultDirectory_list_form_aliases(self):
    """Returns the aliases."""
    return _call(self, 'vault_directory_list_form_aliases', ())
VaultDirectory.list_form_aliases = _VaultDirectory_list_form_aliases

def _VaultDirectory_private_key_exists(self, name):
    """Reports whether exists."""
    return _call(self, 'vault_directory_private_key_exists', (name,))
VaultDirectory.private_key_exists = _VaultDirectory_private_key_exists

def _VaultDirectory_delete_private_key(self, name):
    """Returns the key."""
    return _call(self, 'vault_directory_delete_private_key', (name,))
VaultDirectory.delete_private_key = _VaultDirectory_delete_private_key

def _VaultDirectory_store_private_key(self, name, key):
    """Returns the key."""
    return _call(self, 'vault_directory_store_private_key', (name, key))
VaultDirectory.store_private_key = _VaultDirectory_store_private_key

def _VaultDirectory_load_private_key(self, name):
    """Returns the key."""
    return _call(self, 'vault_directory_load_private_key', (name,))
VaultDirectory.load_private_key = _VaultDirectory_load_private_key

def _VaultDirectory_load_private_key_generation(self, name, index):
    """Returns the generation."""
    return _call(self, 'vault_directory_load_private_key_generation', (name, index))
VaultDirectory.load_private_key_generation = _VaultDirectory_load_private_key_generation

def _VaultDirectory_store_contact(self, name, key):
    """Returns the contact."""
    return _call(self, 'vault_directory_store_contact', (name, key))
VaultDirectory.store_contact = _VaultDirectory_store_contact

def _VaultDirectory_load_contact(self, name):
    """Returns the contact."""
    return _call(self, 'vault_directory_load_contact', (name,))
VaultDirectory.load_contact = _VaultDirectory_load_contact

def _VaultDirectory_contact_exists(self, name):
    """Reports whether exists."""
    return _call(self, 'vault_directory_contact_exists', (name,))
VaultDirectory.contact_exists = _VaultDirectory_contact_exists

def _VaultDirectory_delete_contact(self, name):
    """Returns the contact."""
    return _call(self, 'vault_directory_delete_contact', (name,))
VaultDirectory.delete_contact = _VaultDirectory_delete_contact

def _VaultDirectory_list_contacts(self):
    """Returns the contacts."""
    return _call(self, 'vault_directory_list_contacts', ())
VaultDirectory.list_contacts = _VaultDirectory_list_contacts

def _VaultDirectory_store_profile_email(self, name, email):
    """Returns the email."""
    return _call(self, 'vault_directory_store_profile_email', (name, email))
VaultDirectory.store_profile_email = _VaultDirectory_store_profile_email

def _VaultDirectory_profile_email(self, name):
    """Returns the email."""
    return _call(self, 'vault_directory_profile_email', (name,))
VaultDirectory.profile_email = _VaultDirectory_profile_email

def _VaultDirectory_store_backup(self, id, bytes):
    """Returns the backup."""
    return _call(self, 'vault_directory_store_backup', (id, bytes))
VaultDirectory.store_backup = _VaultDirectory_store_backup

def _VaultDirectory_load_backup(self, id):
    """Returns the backup."""
    return _call(self, 'vault_directory_load_backup', (id,))
VaultDirectory.load_backup = _VaultDirectory_load_backup

def _VaultDirectory_backup_count(self):
    """Returns the count."""
    return _call(self, 'vault_directory_backup_count', ())
VaultDirectory.backup_count = _VaultDirectory_backup_count

def _VaultDirectory_restore_private_key(self, name, key, signing_key, overwrite):
    """Returns the key."""
    return _call(self, 'vault_directory_restore_private_key', (name, key, signing_key, overwrite))
VaultDirectory.restore_private_key = _VaultDirectory_restore_private_key

def _VaultDirectory_load_owner_signing_key(self, name):
    """Returns the key."""
    return _call(self, 'vault_directory_load_owner_signing_key', (name,))
VaultDirectory.load_owner_signing_key = _VaultDirectory_load_owner_signing_key

def _VaultDirectory_load_owner_signing_key_generation(self, name, index):
    """Returns the generation."""
    return _call(self, 'vault_directory_load_owner_signing_key_generation', (name, index))
VaultDirectory.load_owner_signing_key_generation = _VaultDirectory_load_owner_signing_key_generation

def _VaultDirectory_store_contact_signing_key(self, name, key):
    """Returns the key."""
    return _call(self, 'vault_directory_store_contact_signing_key', (name, key))
VaultDirectory.store_contact_signing_key = _VaultDirectory_store_contact_signing_key

def _VaultDirectory_load_contact_signing_key(self, name):
    """Returns the key."""
    return _call(self, 'vault_directory_load_contact_signing_key', (name,))
VaultDirectory.load_contact_signing_key = _VaultDirectory_load_contact_signing_key

def _VaultDirectory_list_profile_generations(self, name):
    """Returns the generations."""
    return _call(self, 'vault_directory_list_profile_generations', (name,))
VaultDirectory.list_profile_generations = _VaultDirectory_list_profile_generations

def _VaultDirectory_rotate_private_key(self, name):
    """Returns the key."""
    return _call(self, 'vault_directory_rotate_private_key', (name,))
VaultDirectory.rotate_private_key = _VaultDirectory_rotate_private_key

def _VaultDirectory_remember_lockbox(self, id, path):
    """Returns the lockbox."""
    return _call(self, 'vault_directory_remember_lockbox', (id, path))
VaultDirectory.remember_lockbox = _VaultDirectory_remember_lockbox

def _VaultDirectory_list_known_lockboxes(self):
    """Returns the lockboxes."""
    return _call(self, 'vault_directory_list_known_lockboxes', ())
VaultDirectory.list_known_lockboxes = _VaultDirectory_list_known_lockboxes

def _VaultDirectory_forget_lockbox(self, path):
    """Returns the lockbox."""
    return _call(self, 'vault_directory_forget_lockbox', (path,))
VaultDirectory.forget_lockbox = _VaultDirectory_forget_lockbox

def _VaultDirectory_remember_access_slot_label(self, id, slot_id, name):
    """Returns the label."""
    return _call(self, 'vault_directory_remember_access_slot_label', (id, slot_id, name))
VaultDirectory.remember_access_slot_label = _VaultDirectory_remember_access_slot_label

def _VaultDirectory_list_access_slot_labels(self, id):
    """Returns the labels."""
    return _call(self, 'vault_directory_list_access_slot_labels', (id,))
VaultDirectory.list_access_slot_labels = _VaultDirectory_list_access_slot_labels

def _VaultDirectory_find_access_slot_labels(self, id, name):
    """Returns the labels."""
    return _call(self, 'vault_directory_find_access_slot_labels', (id, name))
VaultDirectory.find_access_slot_labels = _VaultDirectory_find_access_slot_labels

def _VaultDirectory_forget_access_slot_label(self, id, slot_id):
    """Returns the label."""
    return _call(self, 'vault_directory_forget_access_slot_label', (id, slot_id))
VaultDirectory.forget_access_slot_label = _VaultDirectory_forget_access_slot_label

def _VaultDirectory_define_form(self, alias, name, description, fields):
    """Returns the form."""
    return _call(self, 'vault_directory_define_form', (alias, name, description, encode_form_fields(fields)))
VaultDirectory.define_form = _VaultDirectory_define_form

def _VaultDirectory_resolve_form(self, reference):
    """Returns the form."""
    return _call(self, 'vault_directory_resolve_form', (reference,))
VaultDirectory.resolve_form = _VaultDirectory_resolve_form

def _VaultDirectory_list_forms(self):
    """Returns the forms."""
    return _call(self, 'vault_directory_list_forms', ())
VaultDirectory.list_forms = _VaultDirectory_list_forms

def _VaultDirectory_list_form_revisions(self, type_id):
    """Returns the revisions."""
    return _call(self, 'vault_directory_list_form_revisions', (type_id,))
VaultDirectory.list_form_revisions = _VaultDirectory_list_form_revisions

def _VaultDirectory_seed_forms(self):
    """Returns the forms."""
    return _call(self, 'vault_directory_seed_forms', ())
VaultDirectory.seed_forms = _VaultDirectory_seed_forms

def _VaultDirectory_remember_password(self, id, password):
    """Returns the password."""
    return _call(self, 'vault_directory_remember_password', (id, password))
VaultDirectory.remember_password = _VaultDirectory_remember_password

def _VaultDirectory_remembered_password(self, id):
    """Returns the password."""
    return _call(self, 'vault_directory_remembered_password', (id,))
VaultDirectory.remembered_password = _VaultDirectory_remembered_password

def _VaultDirectory_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'vault_directory_free', ())
VaultDirectory.free = _VaultDirectory_free

def _ReadOnlyVaultDirectory_list_profile_names(self):
    """Returns the names."""
    return _call(self, 'vault_read_only_list_profile_names', ())
ReadOnlyVaultDirectory.list_profile_names = _ReadOnlyVaultDirectory_list_profile_names

def _ReadOnlyVaultDirectory_list_contact_names(self):
    """Returns the names."""
    return _call(self, 'vault_read_only_list_contact_names', ())
ReadOnlyVaultDirectory.list_contact_names = _ReadOnlyVaultDirectory_list_contact_names

def _ReadOnlyVaultDirectory_list_form_aliases(self):
    """Returns the aliases."""
    return _call(self, 'vault_read_only_list_form_aliases', ())
ReadOnlyVaultDirectory.list_form_aliases = _ReadOnlyVaultDirectory_list_form_aliases

def _ReadOnlyVaultDirectory_list_known_lockboxes(self):
    """Returns the lockboxes."""
    return _call(self, 'vault_read_only_list_known_lockboxes', ())
ReadOnlyVaultDirectory.list_known_lockboxes = _ReadOnlyVaultDirectory_list_known_lockboxes

def _ReadOnlyVaultDirectory_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'vault_read_only_free', ())
ReadOnlyVaultDirectory.free = _ReadOnlyVaultDirectory_free

def _Agent_is_running(self):
    """Returns the running."""
    return _call(self, 'vault_is_running', ())
Agent.is_running = _Agent_is_running

def _Agent_forget_all(self):
    """Returns the all."""
    return _call(self, 'vault_forget_all', ())
Agent.forget_all = _Agent_forget_all

def _Agent_serve(self):
    """Returns the serve."""
    return _call(self, 'vault_agent_serve', ())
Agent.serve = _Agent_serve

def _Agent_verify_transport(self):
    """Returns the transport."""
    return _call(self, 'vault_agent_verify_transport', ())
Agent.verify_transport = _Agent_verify_transport

def _Agent_get(self, id):
    """Returns get."""
    return _call(self, 'vault_agent_get', (id,))
Agent.get = _Agent_get

def _Agent_put(self, id, key):
    """Stores put."""
    return _call(self, 'vault_agent_put', (id, key))
Agent.put = _Agent_put

def _Agent_forget(self, id):
    """Removes forget."""
    return _call(self, 'vault_agent_forget', (id,))
Agent.forget = _Agent_forget

def _Agent_stop(self):
    """Stops stop."""
    return _call(self, 'vault_agent_stop', ())
Agent.stop = _Agent_stop

def _Agent_start(self):
    """Starts start."""
    return _call(self, 'vault_agent_start', ())
Agent.start = _Agent_start

def _Agent_list(self):
    """Lists list."""
    return _call(self, 'vault_agent_list', ())
Agent.list = _Agent_list

def _Agent_sleep_support(self):
    """Returns the support."""
    return _call(self, 'vault_agent_sleep_support', ())
Agent.sleep_support = _Agent_sleep_support

def _Agent_get_vault_unlock_key(self, vault_id):
    """Returns the key."""
    return _call(self, 'vault_agent_get_vault_unlock_key', (vault_id,))
Agent.get_vault_unlock_key = _Agent_get_vault_unlock_key

def _Agent_put_vault_unlock_key(self, vault_id, key, ttl_seconds):
    """Returns the key."""
    return _call(self, 'vault_agent_put_vault_unlock_key', (vault_id, key, ttl_seconds))
Agent.put_vault_unlock_key = _Agent_put_vault_unlock_key

def _Agent_forget_vault_unlock_key(self, vault_id):
    """Returns the key."""
    return _call(self, 'vault_agent_forget_vault_unlock_key', (vault_id,))
Agent.forget_vault_unlock_key = _Agent_forget_vault_unlock_key

def _Agent_get_owner_signing_key(self, vault_id, profile):
    """Returns the key."""
    return _call(self, 'vault_agent_get_owner_signing_key', (vault_id, profile))
Agent.get_owner_signing_key = _Agent_get_owner_signing_key

def _Agent_put_owner_signing_key(self, vault_id, profile, key, ttl_seconds):
    """Returns the key."""
    return _call(self, 'vault_agent_put_owner_signing_key', (vault_id, profile, key, ttl_seconds))
Agent.put_owner_signing_key = _Agent_put_owner_signing_key

def _Agent_forget_owner_signing_key(self, vault_id, profile):
    """Returns the key."""
    return _call(self, 'vault_agent_forget_owner_signing_key', (vault_id, profile))
Agent.forget_owner_signing_key = _Agent_forget_owner_signing_key

def _Agent_begin_activity(self, kind):
    """Returns the activity."""
    return _call(self, 'vault_agent_begin_activity', (kind,))
Agent.begin_activity = _Agent_begin_activity

def _Agent_end_activity(self, handle):
    """Returns the activity."""
    return _call(self, 'vault_agent_end_activity', (handle,))
Agent.end_activity = _Agent_end_activity

def _Platform_status(self):
    """Returns the status."""
    return _call(self, 'vault_platform_status', ())
Platform.status = _Platform_status

def _Platform_set_scope(self, scope):
    """Returns the scope."""
    return _call(self, 'vault_platform_set_scope', (scope,))
Platform.set_scope = _Platform_set_scope

def _Platform_forget_password(self):
    """Returns the password."""
    return _call(self, 'vault_platform_forget_password', ())
Platform.forget_password = _Platform_forget_password

def _Platform_put_password(self, password):
    """Returns the password."""
    return _call(self, 'vault_platform_put_password', (password,))
Platform.put_password = _Platform_put_password

def _Platform_enable(self):
    """Returns the enable."""
    return _call(self, 'vault_platform_enable', ())
Platform.enable = _Platform_enable

def _Platform_disable(self):
    """Returns the disable."""
    return _call(self, 'vault_platform_disable', ())
Platform.disable = _Platform_disable

def _Platform_disabled(self):
    """Returns the disabled."""
    return _call(self, 'vault_platform_disabled', ())
Platform.disabled = _Platform_disabled

def _Platform_get_password(self):
    """Returns the password."""
    return _call(self, 'vault_platform_get_password', ())
Platform.get_password = _Platform_get_password

def _LocalVault_create_lockbox_password(self, path, password):
    """Returns the password."""
    return _call(self, 'vault_create_lockbox_password', (path, password))
LocalVault.create_lockbox_password = _LocalVault_create_lockbox_password

def _LocalVault_open_lockbox_password(self, path, password):
    """Returns the password."""
    return _call(self, 'vault_open_lockbox_password', (path, password))
LocalVault.open_lockbox_password = _LocalVault_open_lockbox_password

def _LocalVault_create_lockbox_content_key(self, path, content_key, signing_key):
    """Returns the key."""
    return _call(self, 'vault_create_lockbox_content_key', (path, content_key, signing_key))
LocalVault.create_lockbox_content_key = _LocalVault_create_lockbox_content_key

def _LocalVault_create_lockbox_contact(self, path, contact, name, signing_key):
    """Returns the contact."""
    return _call(self, 'vault_create_lockbox_contact', (path, contact, name, signing_key))
LocalVault.create_lockbox_contact = _LocalVault_create_lockbox_contact

def _LocalVault_open_lockbox_content_key(self, path, content_key, signing_key):
    """Returns the key."""
    return _call(self, 'vault_open_lockbox_content_key', (path, content_key, signing_key))
LocalVault.open_lockbox_content_key = _LocalVault_open_lockbox_content_key

def _LocalVault_cache_lockbox_password(self, path, password, ttl_seconds):
    """Returns the password."""
    return _call(self, 'vault_cache_lockbox_password', (path, password, ttl_seconds))
LocalVault.cache_lockbox_password = _LocalVault_cache_lockbox_password

def _LocalVault_close_lockbox(self, path):
    """Returns the lockbox."""
    return _call(self, 'vault_close_lockbox', (path,))
LocalVault.close_lockbox = _LocalVault_close_lockbox

def _LocalVault_close_all(self):
    """Returns the all."""
    return _call(self, 'vault_close_all', ())
LocalVault.close_all = _LocalVault_close_all

def _LocalVault_free(self):
    """Releases the native resources held by this object."""
    return _call(self, 'vault_free', ())
LocalVault.free = _LocalVault_free

Revault = Vault

_ROUTES = {
    'buffer_last_error_details': ((), 'message:ErrorDetails', False),
    'lockbox_format_version': ((), 'value', False),
    'lockbox_probe_format_version': (('bytes',), 'value', False),
    'lockbox_create': (('bytes',), 'handle:Lockbox', False),
    'lockbox_create_with_options': (('bytes', 'text', 'value', 'text', 'text', 'value'), 'handle:Lockbox', False),
    'lockbox_create_password': (('bytes',), 'handle:Lockbox', False),
    'lockbox_create_contact': (('handle',), 'handle:Lockbox', False),
    'lockbox_create_with_signing_key': (('bytes', 'handle'), 'handle:Lockbox', False),
    'lockbox_open': (('bytes', 'bytes'), 'handle:Lockbox', False),
    'lockbox_open_with_options': (('bytes', 'bytes', 'text', 'value', 'text', 'text', 'value'), 'handle:Lockbox', False),
    'lockbox_open_password': (('bytes', 'bytes'), 'handle:Lockbox', False),
    'lockbox_open_contact': (('bytes', 'handle'), 'handle:Lockbox', False),
    'lockbox_add_file': (('handle', 'text', 'bytes', 'value'), 'bool', False),
    'lockbox_add_file_with_permissions': (('handle', 'text', 'bytes', 'value', 'value'), 'bool', False),
    'lockbox_get_file': (('handle', 'text'), 'bytes', False),
    'lockbox_extract_file': (('handle', 'text', 'text', 'value'), 'bool', False),
    'lockbox_extract_directory': (('handle', 'text', 'value', 'value', 'value', 'value', 'value', 'value'), 'bool', False),
    'lockbox_stream_content': (('handle', 'value'), 'message:StreamChunkList', False),
    'lockbox_cache_stats': (('handle',), 'message:CacheStats', False),
    'lockbox_import_stats': (('handle',), 'message:ImportStats', False),
    'lockbox_reset_import_stats': (('handle',), 'bool', False),
    'lockbox_inspect_file': (('text',), 'message:FileInspection', False),
    'lockbox_page_inspection': (('handle',), 'message:PageInspectionList', False),
    'lockbox_recovery_report': (('handle',), 'message:RecoveryReport', False),
    'lockbox_recovery_report_render': (('handle', 'value', 'value'), 'utf8', False),
    'lockbox_recovery_scan_path': (('text', 'bytes'), 'message:RecoveryReport', False),
    'lockbox_storage_len': (('handle',), 'value', False),
    'lockbox_set_workload_profile': (('handle', 'text'), 'bool', False),
    'lockbox_set_worker_policy': (('handle', 'text', 'value'), 'bool', False),
    'lockbox_runtime_options': (('handle',), 'message:RuntimeOptions', False),
    'lockbox_commit': (('handle',), 'bool', False),
    'lockbox_create_dir': (('handle', 'text', 'value'), 'bool', False),
    'lockbox_delete': (('handle', 'text'), 'bool', False),
    'lockbox_remove_dir': (('handle', 'text', 'value'), 'bool', False),
    'lockbox_create_parent_dirs': (('handle', 'text'), 'bool', False),
    'lockbox_rename': (('handle', 'text', 'text'), 'bool', False),
    'lockbox_list': (('handle', 'text', 'value'), 'message:LockboxEntryList', False),
    'lockbox_list_with_options': (('handle', 'text', 'text', 'value', 'value', 'value', 'value', 'value'), 'message:LockboxEntryList', False),
    'lockbox_stat': (('handle', 'text'), 'message:OptionalLockboxEntry', False),
    'lockbox_set_variable': (('handle', 'text', 'text'), 'bool', False),
    'lockbox_set_secret_variable': (('handle', 'text', 'bytes'), 'bool', False),
    'lockbox_get_variable': (('handle', 'text'), 'message:OptionalString', False),
    'lockbox_get_secret_variable': (('handle', 'text'), 'secret', False),
    'lockbox_delete_variable': (('handle', 'text'), 'bool', False),
    'lockbox_move_variables': (('handle', 'bytes'), 'bool', False),
    'lockbox_list_variables': (('handle',), 'message:VariableList', False),
    'lockbox_variable_sensitivity': (('handle', 'text'), 'message:OptionalString', False),
    'lockbox_add_symlink': (('handle', 'text', 'text', 'value'), 'bool', False),
    'lockbox_get_symlink_target': (('handle', 'text'), 'utf8', False),
    'lockbox_id': (('handle',), 'bytes', False),
    'lockbox_exists': (('handle', 'text'), 'predicate', False),
    'lockbox_is_dir': (('handle', 'text'), 'predicate', False),
    'lockbox_permissions': (('handle', 'text'), 'value', False),
    'lockbox_set_permissions': (('handle', 'text', 'value'), 'bool', False),
    'lockbox_read_range': (('handle', 'text', 'value', 'value'), 'bytes', False),
    'lockbox_recovery_scan': (('bytes', 'bytes'), 'message:RecoveryReport', False),
    'lockbox_recovery_salvage': (('bytes', 'bytes', 'handle'), 'handle:Lockbox', False),
    'lockbox_add_password': (('handle', 'bytes'), 'value', False),
    'lockbox_add_contact': (('handle', 'handle', 'text'), 'value', False),
    'lockbox_delete_key': (('handle', 'value'), 'bool', False),
    'lockbox_list_key_slots': (('handle',), 'message:KeySlotList', False),
    'lockbox_set_owner_signing_key': (('handle', 'handle'), 'bool', False),
    'lockbox_owner_inspection': (('handle',), 'message:OwnerInspection', False),
    'lockbox_define_form': (('handle', 'text', 'text', 'text', 'bytes'), 'message:FormDefinition', False),
    'lockbox_list_form_definitions': (('handle',), 'message:FormDefinitionList', False),
    'lockbox_resolve_form': (('handle', 'text'), 'message:FormDefinition', False),
    'lockbox_list_form_revisions': (('handle', 'text'), 'message:FormDefinitionList', False),
    'lockbox_create_form_record': (('handle', 'text', 'text', 'text'), 'message:FormRecord', False),
    'lockbox_set_form_field': (('handle', 'text', 'text', 'text'), 'bool', False),
    'lockbox_set_secret_form_field': (('handle', 'text', 'text', 'bytes'), 'bool', False),
    'lockbox_list_form_records': (('handle',), 'message:FormRecordList', False),
    'lockbox_get_form_record': (('handle', 'text'), 'message:OptionalFormRecord', False),
    'lockbox_delete_form_record': (('handle', 'text'), 'bool', False),
    'lockbox_move_form_records': (('handle', 'bytes'), 'bool', False),
    'lockbox_get_form_field': (('handle', 'text', 'text'), 'message:OptionalFormValue', False),
    'lockbox_get_secret_form_field': (('handle', 'text', 'text'), 'secret', False),
    'secret_len': (('handle', 'value'), 'bool', False),
    'secret_copy': (('handle', 'bytes'), 'bool', False),
    'secret_free': (('handle',), 'void', False),
    'lockbox_to_bytes': (('handle',), 'bytes', False),
    'lockbox_free': (('handle',), 'void', True),
    'vault_is_running': ((), 'predicate', False),
    'vault_forget_all': ((), 'bool', False),
    'key_contact_generate': ((), 'handle:ContactKeyPair', False),
    'key_contact_from_private': (('bytes',), 'handle:ContactKeyPair', False),
    'key_contact_public': (('handle',), 'bytes', False),
    'key_contact_private': (('handle',), 'bytes', False),
    'key_contact_public_from_bytes': (('bytes',), 'handle:ContactPublicKey', False),
    'key_contact_public_free': (('handle',), 'void', True),
    'key_contact_free': (('handle',), 'void', True),
    'key_contact_encrypt': (('handle', 'bytes'), 'handle:WrappedContactKey', False),
    'key_contact_decrypt': (('handle', 'handle'), 'bytes', False),
    'key_contact_wrapped_public': (('handle',), 'bytes', False),
    'key_contact_wrapped_ciphertext': (('handle',), 'bytes', False),
    'key_contact_wrapped_encrypted': (('handle',), 'bytes', False),
    'key_contact_wrapped_free': (('handle',), 'void', True),
    'key_signing_generate': ((), 'handle:SigningKeyPair', False),
    'key_signing_from_private': (('bytes',), 'handle:SigningKeyPair', False),
    'key_signing_public': (('handle',), 'bytes', False),
    'key_signing_private': (('handle',), 'bytes', False),
    'key_signing_public_from_bytes': (('bytes',), 'handle:SigningPublicKey', False),
    'key_signing_public_free': (('handle',), 'void', True),
    'key_signing_free': (('handle',), 'void', True),
    'vault_key_export_private': (('handle', 'text'), 'bytes', False),
    'vault_key_export_public': (('handle', 'text'), 'bytes', False),
    'vault_key_import_private': (('bytes',), 'handle:ContactKeyPair', False),
    'vault_key_import_public': (('bytes',), 'handle:ContactPublicKey', False),
    'vault_key_fingerprint': (('handle',), 'bytes', False),
    'vault_key_format_hex': (('bytes',), 'utf8', False),
    'vault_key_decode_hex': (('text',), 'bytes', False),
    'vault_key_format_crockford': (('bytes',), 'utf8', False),
    'vault_key_format_crockford_reading': (('text',), 'utf8', False),
    'vault_key_decode_crockford': (('text',), 'bytes', False),
    'vault_key_hex_encode': (('bytes',), 'utf8', False),
    'vault_key_hex_decode': (('text',), 'bytes', False),
    'vault_directory_open': (('text', 'bytes'), 'handle:VaultDirectory', False),
    'vault_structure_version_current': ((), 'value', False),
    'vault_directory_probe_structure_version': (('text', 'bytes'), 'value', False),
    'vault_directory_open_or_create_default': (('bytes',), 'handle:VaultDirectory', False),
    'vault_directory_replace_default': (('bytes',), 'handle:VaultDirectory', False),
    'vault_directory_change_password': (('text', 'bytes', 'bytes'), 'bool', False),
    'vault_directory_change_default_password': (('bytes', 'bytes'), 'bool', False),
    'vault_directory_replace': (('text', 'bytes'), 'handle:VaultDirectory', False),
    'vault_directory_open_or_create': (('text', 'bytes'), 'handle:VaultDirectory', False),
    'vault_directory_root': (('handle',), 'utf8', False),
    'vault_directory_structure_version': (('handle',), 'value', False),
    'vault_directory_list_private_keys': (('handle',), 'message:StringList', False),
    'vault_directory_list_private_key_names': (('handle',), 'message:StringList', False),
    'vault_directory_list_contact_names': (('handle',), 'message:StringList', False),
    'vault_directory_list_form_aliases': (('handle',), 'message:StringList', False),
    'vault_directory_private_key_exists': (('handle', 'text'), 'predicate', False),
    'vault_directory_delete_private_key': (('handle', 'text'), 'bool', False),
    'vault_directory_store_private_key': (('handle', 'text', 'handle'), 'bool', False),
    'vault_directory_load_private_key': (('handle', 'text'), 'handle:ContactKeyPair', False),
    'vault_directory_load_private_key_generation': (('handle', 'text', 'value'), 'handle:ContactKeyPair', False),
    'vault_directory_store_contact': (('handle', 'text', 'handle'), 'bool', False),
    'vault_directory_load_contact': (('handle', 'text'), 'handle:ContactPublicKey', False),
    'vault_directory_contact_exists': (('handle', 'text'), 'predicate', False),
    'vault_directory_delete_contact': (('handle', 'text'), 'bool', False),
    'vault_directory_list_contacts': (('handle',), 'message:ContactList', False),
    'vault_directory_store_profile_email': (('handle', 'text', 'text'), 'bool', False),
    'vault_directory_profile_email': (('handle', 'text'), 'message:OptionalString', False),
    'vault_directory_store_backup': (('handle', 'bytes', 'bytes'), 'bool', False),
    'vault_directory_load_backup': (('handle', 'bytes'), 'bytes', False),
    'vault_directory_backup_count': (('handle',), 'value', False),
    'vault_directory_restore_private_key': (('handle', 'text', 'handle', 'handle', 'value'), 'bool', False),
    'vault_directory_load_owner_signing_key': (('handle', 'text'), 'handle:SigningKeyPair', False),
    'vault_directory_load_owner_signing_key_generation': (('handle', 'text', 'value'), 'handle:SigningKeyPair', False),
    'vault_directory_store_contact_signing_key': (('handle', 'text', 'handle'), 'bool', False),
    'vault_directory_load_contact_signing_key': (('handle', 'text'), 'handle:SigningPublicKey', False),
    'vault_directory_list_profile_generations': (('handle', 'text'), 'message:ProfileHistory', False),
    'vault_directory_rotate_private_key': (('handle', 'text'), 'message:ProfileHistory', False),
    'vault_directory_remember_lockbox': (('handle', 'bytes', 'text'), 'bool', False),
    'vault_directory_list_known_lockboxes': (('handle',), 'message:KnownLockboxList', False),
    'vault_directory_forget_lockbox': (('handle', 'text'), 'bool', False),
    'vault_directory_remember_access_slot_label': (('handle', 'bytes', 'value', 'text'), 'bool', False),
    'vault_directory_list_access_slot_labels': (('handle', 'bytes'), 'message:AccessSlotLabelList', False),
    'vault_directory_find_access_slot_labels': (('handle', 'bytes', 'text'), 'message:AccessSlotLabelList', False),
    'vault_directory_forget_access_slot_label': (('handle', 'bytes', 'value'), 'bool', False),
    'vault_directory_define_form': (('handle', 'text', 'text', 'text', 'bytes'), 'message:FormDefinition', False),
    'vault_directory_resolve_form': (('handle', 'text'), 'message:FormDefinition', False),
    'vault_directory_list_forms': (('handle',), 'message:FormDefinitionList', False),
    'vault_directory_list_form_revisions': (('handle', 'text'), 'message:FormDefinitionList', False),
    'vault_directory_seed_forms': (('handle',), 'value', False),
    'vault_directory_remember_password': (('handle', 'bytes', 'bytes'), 'bool', False),
    'vault_directory_remembered_password': (('handle', 'bytes'), 'bytes', False),
    'vault_backup_default': (('text', 'value'), 'message:VaultBackupManifest', False),
    'vault_restore_default': (('text', 'value'), 'message:VaultBackupManifest', False),
    'vault_directory_free': (('handle',), 'void', True),
    'vault_read_only_open': (('text', 'bytes'), 'handle:ReadOnlyVaultDirectory', False),
    'vault_read_only_open_default': (('bytes',), 'handle:ReadOnlyVaultDirectory', False),
    'vault_read_only_list_profile_names': (('handle',), 'message:StringList', False),
    'vault_read_only_list_contact_names': (('handle',), 'message:StringList', False),
    'vault_read_only_list_form_aliases': (('handle',), 'message:StringList', False),
    'vault_read_only_list_known_lockboxes': (('handle',), 'message:KnownLockboxList', False),
    'vault_read_only_free': (('handle',), 'void', True),
    'vault_agent_serve': ((), 'bool', False),
    'vault_agent_verify_transport': ((), 'bool', False),
    'vault_agent_get': (('bytes',), 'bytes', False),
    'vault_agent_put': (('bytes', 'bytes'), 'bool', False),
    'vault_agent_forget': (('bytes',), 'bool', False),
    'vault_agent_stop': ((), 'bool', False),
    'vault_agent_start': ((), 'bool', False),
    'vault_agent_list': ((), 'message:AgentEntryList', False),
    'vault_agent_sleep_support': ((), 'message:SleepSupport', False),
    'vault_platform_status': ((), 'message:PlatformStatus', False),
    'vault_platform_set_scope': (('text',), 'bool', False),
    'vault_platform_forget_password': ((), 'bool', False),
    'vault_platform_put_password': (('bytes',), 'bool', False),
    'vault_platform_enable': ((), 'bool', False),
    'vault_platform_disable': ((), 'bool', False),
    'vault_platform_disabled': ((), 'predicate', False),
    'vault_platform_get_password': ((), 'bytes', False),
    'vault_default_directory': ((), 'utf8', False),
    'vault_default_path': ((), 'utf8', False),
    'vault_agent_log_path': ((), 'utf8', False),
    'vault_agent_log_destination': ((), 'utf8', False),
    'vault_agent_get_vault_unlock_key': (('text',), 'bytes', False),
    'vault_agent_put_vault_unlock_key': (('text', 'bytes', 'value'), 'bool', False),
    'vault_agent_forget_vault_unlock_key': (('text',), 'bool', False),
    'vault_agent_get_owner_signing_key': (('text', 'text'), 'handle:SigningKeyPair', False),
    'vault_agent_put_owner_signing_key': (('text', 'text', 'handle', 'value'), 'bool', False),
    'vault_agent_forget_owner_signing_key': (('text', 'text'), 'bool', False),
    'vault_agent_begin_activity': (('text',), 'handle:AgentActivity', False),
    'vault_agent_end_activity': (('handle',), 'void', False),
    'vault_local': ((), 'handle:LocalVault', False),
    'vault_create_lockbox_password': (('handle', 'text', 'bytes'), 'handle:Lockbox', False),
    'vault_open_lockbox_password': (('handle', 'text', 'bytes'), 'handle:Lockbox', False),
    'vault_create_lockbox_content_key': (('handle', 'text', 'bytes', 'handle'), 'handle:Lockbox', False),
    'vault_create_lockbox_contact': (('handle', 'text', 'handle', 'text', 'handle'), 'handle:Lockbox', False),
    'vault_open_lockbox_content_key': (('handle', 'text', 'bytes', 'handle'), 'handle:Lockbox', False),
    'vault_cache_lockbox_password': (('handle', 'text', 'bytes', 'value'), 'bool', False),
    'vault_close_lockbox': (('handle', 'text'), 'bool', False),
    'vault_close_all': (('handle',), 'bool', False),
    'vault_free': (('handle',), 'void', True),
}

__all__ = ['Vault', 'Lockbox', 'ContactKeyPair', 'ContactPublicKey', 'WrappedContactKey', 'SigningKeyPair', 'SigningPublicKey', 'VaultDirectory', 'ReadOnlyVaultDirectory', 'Agent', 'AgentActivity', 'Platform', 'LocalVault', 'Revault']
