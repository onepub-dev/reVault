using System;
using System.Runtime.InteropServices;

namespace Revault;

[StructLayout(LayoutKind.Sequential)]
internal struct RevaultBuffer { public IntPtr Ptr; public nuint Length; }

internal static partial class RevaultNative
{
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern uint api_abi_version();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr buffer_last_error();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer buffer_last_error_details();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void buffer_free(RevaultBuffer value);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool secret_len(IntPtr handle, out nuint length);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool secret_copy(IntPtr handle, IntPtr destination, nuint destination_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void secret_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ushort lockbox_format_version();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ushort lockbox_probe_format_version(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_create(IntPtr key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_create_with_options(IntPtr key, nuint key_len, IntPtr cache_mode, nuint cache_len, ulong cache_bytes, IntPtr workload, nuint workload_len, IntPtr worker, nuint worker_len, nuint jobs);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_create_password(IntPtr password, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_create_contact(IntPtr contact);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_create_with_signing_key(IntPtr content_key, nuint key_len, IntPtr signing_key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_open(IntPtr archive, nuint archive_len, IntPtr key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_open_with_options(IntPtr archive, nuint archive_len, IntPtr key, nuint key_len, IntPtr cache_mode, nuint cache_len, ulong cache_bytes, IntPtr workload, nuint workload_len, IntPtr worker, nuint worker_len, nuint jobs);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_open_password(IntPtr archive, nuint archive_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_open_contact(IntPtr archive, nuint archive_len, IntPtr contact);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_add_file(IntPtr handle, IntPtr path, nuint path_len, IntPtr data, nuint data_len, [MarshalAs(UnmanagedType.I1)] bool replace);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_add_file_with_permissions(IntPtr handle, IntPtr path, nuint path_len, IntPtr data, nuint data_len, uint permissions, [MarshalAs(UnmanagedType.I1)] bool replace);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_get_file(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_extract_file(IntPtr handle, IntPtr source, nuint source_len, IntPtr destination, nuint destination_len, [MarshalAs(UnmanagedType.I1)] bool replace);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_extract_directory(IntPtr handle, IntPtr destination, nuint destination_len, ulong max_file_bytes, ulong max_total_bytes, nuint max_files, [MarshalAs(UnmanagedType.I1)] bool restore_symlinks, [MarshalAs(UnmanagedType.I1)] bool restore_permissions, [MarshalAs(UnmanagedType.I1)] bool overwrite);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_stream_content(IntPtr handle, [MarshalAs(UnmanagedType.I1)] bool physical);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_cache_stats(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_import_stats(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_reset_import_stats(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_inspect_file(IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_page_inspection(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_recovery_report(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_recovery_report_render(IntPtr handle, [MarshalAs(UnmanagedType.I1)] bool verbose, nuint max_entries);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_recovery_scan_path(IntPtr path, nuint path_len, IntPtr key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong lockbox_storage_len(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_workload_profile(IntPtr handle, IntPtr profile, nuint profile_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_worker_policy(IntPtr handle, IntPtr mode, nuint mode_len, nuint jobs);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_runtime_options(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_commit(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_create_dir(IntPtr handle, IntPtr path, nuint path_len, [MarshalAs(UnmanagedType.I1)] bool create_parents);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_delete(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_remove_dir(IntPtr handle, IntPtr path, nuint path_len, [MarshalAs(UnmanagedType.I1)] bool recursive);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_create_parent_dirs(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_rename(IntPtr handle, IntPtr from, nuint from_len, IntPtr to, nuint to_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list(IntPtr handle, IntPtr path, nuint path_len, [MarshalAs(UnmanagedType.I1)] bool recursive);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_with_options(IntPtr handle, IntPtr path, nuint path_len, IntPtr glob, nuint glob_len, [MarshalAs(UnmanagedType.I1)] bool recursive, [MarshalAs(UnmanagedType.I1)] bool include_files, [MarshalAs(UnmanagedType.I1)] bool include_symlinks, [MarshalAs(UnmanagedType.I1)] bool include_directories, nuint limit);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_stat(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_variable(IntPtr handle, IntPtr name, nuint name_len, IntPtr value, nuint value_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_secret_variable(IntPtr handle, IntPtr name, nuint name_len, IntPtr value, nuint value_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_get_variable(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_get_secret_variable(IntPtr handle, IntPtr name, nuint name_len, out IntPtr output);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_delete_variable(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_move_variables(IntPtr handle, IntPtr moves_proto, nuint moves_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_variables(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_variable_sensitivity(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_add_symlink(IntPtr handle, IntPtr path, nuint path_len, IntPtr target, nuint target_len, [MarshalAs(UnmanagedType.I1)] bool replace);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_get_symlink_target(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_id(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_exists(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_is_dir(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern uint lockbox_permissions(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_permissions(IntPtr handle, IntPtr path, nuint path_len, uint permissions);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_read_range(IntPtr handle, IntPtr path, nuint path_len, ulong offset, ulong len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_recovery_scan(IntPtr bytes, nuint len, IntPtr key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr lockbox_recovery_salvage(IntPtr bytes, nuint len, IntPtr key, nuint key_len, IntPtr signing_key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong lockbox_add_password(IntPtr handle, IntPtr password, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong lockbox_add_contact(IntPtr handle, IntPtr contact, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_delete_key(IntPtr handle, ulong id);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_key_slots(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_owner_signing_key(IntPtr handle, IntPtr key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_owner_inspection(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_define_form(IntPtr handle, IntPtr alias, nuint alias_len, IntPtr name, nuint name_len, IntPtr description, nuint description_len, IntPtr fields_proto, nuint fields_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_form_definitions(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_resolve_form(IntPtr handle, IntPtr reference, nuint reference_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_form_revisions(IntPtr handle, IntPtr type_id, nuint type_id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_create_form_record(IntPtr handle, IntPtr path, nuint path_len, IntPtr type_reference, nuint type_len, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_form_field(IntPtr handle, IntPtr path, nuint path_len, IntPtr field, nuint field_len, IntPtr value, nuint value_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_set_secret_form_field(IntPtr handle, IntPtr path, nuint path_len, IntPtr field, nuint field_len, IntPtr value, nuint value_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_list_form_records(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_get_form_record(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_delete_form_record(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_move_form_records(IntPtr handle, IntPtr moves_proto, nuint moves_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_get_form_field(IntPtr handle, IntPtr path, nuint path_len, IntPtr field, nuint field_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool lockbox_get_secret_form_field(IntPtr handle, IntPtr path, nuint path_len, IntPtr field, nuint field_len, out IntPtr output);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer lockbox_to_bytes(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void lockbox_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_is_running();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_forget_all();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_contact_generate();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_contact_from_private(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_public(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_private(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_contact_public_from_bytes(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void key_contact_public_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void key_contact_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_contact_encrypt(IntPtr contact, IntPtr content_key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_decrypt(IntPtr contact, IntPtr wrapped);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_wrapped_public(IntPtr wrapped);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_wrapped_ciphertext(IntPtr wrapped);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_contact_wrapped_encrypted(IntPtr wrapped);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void key_contact_wrapped_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_signing_generate();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_signing_from_private(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_signing_public(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer key_signing_private(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr key_signing_public_from_bytes(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void key_signing_public_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void key_signing_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_export_private(IntPtr key, IntPtr format, nuint format_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_export_public(IntPtr key, IntPtr format, nuint format_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_key_import_private(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_key_import_public(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_fingerprint(IntPtr key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_format_hex(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_decode_hex(IntPtr text, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_format_crockford(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_format_crockford_reading(IntPtr code, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_decode_crockford(IntPtr code, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_hex_encode(IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_key_hex_decode(IntPtr text, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_open(IntPtr root, nuint root_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern uint vault_structure_version_current();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern uint vault_directory_probe_structure_version(IntPtr root, nuint root_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_open_or_create_default(IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_replace_default(IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_change_password(IntPtr root, nuint root_len, IntPtr old_password, nuint old_len, IntPtr new_password, nuint new_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_change_default_password(IntPtr old_password, nuint old_len, IntPtr new_password, nuint new_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_replace(IntPtr root, nuint root_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_open_or_create(IntPtr root, nuint root_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_root(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern uint vault_directory_structure_version(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_private_keys(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_private_key_names(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_contact_names(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_form_aliases(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_private_key_exists(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_delete_private_key(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_store_private_key(IntPtr handle, IntPtr name, nuint name_len, IntPtr key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_private_key(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_private_key_generation(IntPtr handle, IntPtr name, nuint name_len, ushort index);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_store_contact(IntPtr handle, IntPtr name, nuint name_len, IntPtr key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_contact(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_contact_exists(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_delete_contact(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_contacts(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_store_profile_email(IntPtr handle, IntPtr name, nuint name_len, IntPtr email, nuint email_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_profile_email(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_store_backup(IntPtr handle, IntPtr id, nuint id_len, IntPtr bytes, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_load_backup(IntPtr handle, IntPtr id, nuint id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern ulong vault_directory_backup_count(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_restore_private_key(IntPtr handle, IntPtr name, nuint name_len, IntPtr key, IntPtr signing_key, [MarshalAs(UnmanagedType.I1)] bool overwrite);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_owner_signing_key(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_owner_signing_key_generation(IntPtr handle, IntPtr name, nuint name_len, ushort index);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_store_contact_signing_key(IntPtr handle, IntPtr name, nuint name_len, IntPtr key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_directory_load_contact_signing_key(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_profile_generations(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_rotate_private_key(IntPtr handle, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_remember_lockbox(IntPtr handle, IntPtr id, nuint id_len, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_known_lockboxes(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_forget_lockbox(IntPtr handle, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_remember_access_slot_label(IntPtr handle, IntPtr id, nuint id_len, ulong slot_id, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_access_slot_labels(IntPtr handle, IntPtr id, nuint id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_find_access_slot_labels(IntPtr handle, IntPtr id, nuint id_len, IntPtr name, nuint name_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_forget_access_slot_label(IntPtr handle, IntPtr id, nuint id_len, ulong slot_id);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_define_form(IntPtr handle, IntPtr alias, nuint alias_len, IntPtr name, nuint name_len, IntPtr description, nuint description_len, IntPtr fields_proto, nuint fields_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_resolve_form(IntPtr handle, IntPtr reference, nuint reference_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_forms(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_list_form_revisions(IntPtr handle, IntPtr type_id, nuint type_id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern nuint vault_directory_seed_forms(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_directory_remember_password(IntPtr handle, IntPtr id, nuint id_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_directory_remembered_password(IntPtr handle, IntPtr id, nuint id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_backup_default(IntPtr path, nuint path_len, [MarshalAs(UnmanagedType.I1)] bool overwrite);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_restore_default(IntPtr path, nuint path_len, [MarshalAs(UnmanagedType.I1)] bool overwrite);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void vault_directory_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_read_only_open(IntPtr root, nuint root_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_read_only_open_default(IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_read_only_list_profile_names(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_read_only_list_contact_names(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_read_only_list_form_aliases(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_read_only_list_known_lockboxes(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void vault_read_only_free(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_serve();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_verify_transport();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_get(IntPtr id, nuint id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_put(IntPtr id, nuint id_len, IntPtr key, nuint key_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_forget(IntPtr id, nuint id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_stop();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_start();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_list();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_sleep_support();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_platform_status();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_set_scope(IntPtr scope, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_forget_password();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_put_password(IntPtr password, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_enable();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_disable();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_platform_disabled();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_platform_get_password();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_default_directory();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_default_path();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_log_path();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_log_destination();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern RevaultBuffer vault_agent_get_vault_unlock_key(IntPtr vault_id, nuint vault_id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_put_vault_unlock_key(IntPtr vault_id, nuint vault_id_len, IntPtr key, nuint key_len, ulong ttl_seconds);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_forget_vault_unlock_key(IntPtr vault_id, nuint vault_id_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_agent_get_owner_signing_key(IntPtr vault_id, nuint vault_len, IntPtr profile, nuint profile_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_put_owner_signing_key(IntPtr vault_id, nuint vault_len, IntPtr profile, nuint profile_len, IntPtr key, ulong ttl_seconds);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_agent_forget_owner_signing_key(IntPtr vault_id, nuint vault_len, IntPtr profile, nuint profile_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_agent_begin_activity(IntPtr kind, nuint len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void vault_agent_end_activity(IntPtr handle);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_local();
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_create_lockbox_password(IntPtr vault, IntPtr path, nuint path_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_open_lockbox_password(IntPtr vault, IntPtr path, nuint path_len, IntPtr password, nuint password_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_create_lockbox_content_key(IntPtr vault, IntPtr path, nuint path_len, IntPtr content_key, nuint key_len, IntPtr signing_key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_create_lockbox_contact(IntPtr vault, IntPtr path, nuint path_len, IntPtr contact, IntPtr name, nuint name_len, IntPtr signing_key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr vault_open_lockbox_content_key(IntPtr vault, IntPtr path, nuint path_len, IntPtr content_key, nuint key_len, IntPtr signing_key);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_cache_lockbox_password(IntPtr vault, IntPtr path, nuint path_len, IntPtr password, nuint password_len, ulong ttl_seconds);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_close_lockbox(IntPtr vault, IntPtr path, nuint path_len);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    public static extern bool vault_close_all(IntPtr vault);
    [DllImport("revault_api", CallingConvention = CallingConvention.Cdecl)]
    public static extern void vault_free(IntPtr vault);
}
