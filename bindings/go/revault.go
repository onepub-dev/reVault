package revault

/*
#cgo LDFLAGS: -lrevault_api
#include "../../rust/revault_bindings/revault_api.h"
*/
import "C"

import (
	"encoding/binary"
	"errors"
	"fmt"
	"unsafe"

	"github.com/onepub-dev/revault-api/messages"
	"google.golang.org/protobuf/proto"
)

func init() {
	if C.api_abi_version() != 2 {
		panic("revault-api native ABI mismatch; expected 2")
	}
}

func lastError() error  { return errors.New(C.GoString(C.buffer_last_error())) }
// LastError returns the diagnostic from the most recent failed native call on this thread.
func LastError() string { return C.GoString(C.buffer_last_error()) }
// LastErrorDetails returns structured diagnostics for the most recent native failure.
func LastErrorDetails() (*messages.ErrorDetails, error) {
	result := &messages.ErrorDetails{}
	return result, decodeFrame(C.buffer_last_error_details(), result)
}
func LockboxFormatVersion() uint16 { return uint16(C.lockbox_format_version()) }
func ProbeLockboxFormatVersion(value []byte) uint16 { return uint16(C.lockbox_probe_format_version(bytePointer(value), C.size_t(len(value)))) }
func CurrentVaultStructureVersion() uint32 { return uint32(C.vault_structure_version_current()) }
func ProbeVaultStructureVersion(root string, password []byte) uint32 {
	return uint32(C.vault_directory_probe_structure_version(charPointer(root), C.size_t(len(root)), bytePointer(password), C.size_t(len(password))))
}
func charPointer(value string) *C.char {
	if value == "" {
		return nil
	}
	return (*C.char)(unsafe.Pointer(unsafe.StringData(value)))
}
func bytePointer(value []byte) *C.uint8_t {
	if len(value) == 0 {
		return nil
	}
	return (*C.uint8_t)(unsafe.Pointer(unsafe.SliceData(value)))
}
func takeBuffer(value C.RevaultBuffer) ([]byte, error) {
	if value.ptr == nil {
		return nil, lastError()
	}
	result := C.GoBytes(unsafe.Pointer(value.ptr), C.int(value.len))
	C.buffer_free(value)
	return result, nil
}
func takeString(value C.RevaultBuffer) (string, error) {
	result, err := takeBuffer(value)
	return string(result), err
}
func decodeFrame(value C.RevaultBuffer, result proto.Message) error {
	frame, err := takeBuffer(value)
	if err != nil {
		return err
	}
	if len(frame) < 12 || string(frame[:4]) != "LBWF" {
		return errors.New("invalid reVault binary frame")
	}
	length := int(binary.BigEndian.Uint32(frame[8:12]))
	if length+12 != len(frame) {
		return errors.New("invalid reVault binary frame length")
	}
	if err := proto.Unmarshal(frame[12:], result); err != nil {
		return fmt.Errorf("invalid reVault protobuf payload: %w", err)
	}
	return nil
}
func require(ok bool) error {
	if !ok {
		return lastError()
	}
	return nil
}

// withSecret limits plaintext lifetime to callback and clears the Go copy on return.
func withSecret(get func(*unsafe.Pointer) bool, callback func([]byte) error) error {
	var handle unsafe.Pointer
	if !get(&handle) {
		return lastError()
	}
	if handle == nil {
		return nil
	}
	defer C.secret_free(handle)
	var length C.size_t
	if !bool(C.secret_len(handle, &length)) {
		return lastError()
	}
	secret := make([]byte, int(length))
	defer clear(secret)
	if !bool(C.secret_copy(handle, bytePointer(secret), length)) {
		return lastError()
	}
	return callback(secret)
}

// ContactPublicKey is a shareable key used to encrypt a recipient content key.
type ContactPublicKey struct{ handle unsafe.Pointer }

func NewContactPublicKey(value []byte) (*ContactPublicKey, error) {
	h := C.key_contact_public_from_bytes(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactPublicKey{handle: h}, nil
}
func ImportContactPublicKey(value []byte) (*ContactPublicKey, error) {
	h := C.vault_key_import_public(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactPublicKey{handle: h}, nil
}
func (key *ContactPublicKey) Close() {
	if key != nil && key.handle != nil {
		C.key_contact_public_free(key.handle)
		key.handle = nil
	}
}
func (key *ContactPublicKey) Export(format string) ([]byte, error) {
	return takeBuffer(C.vault_key_export_public(key.handle, charPointer(format), C.size_t(len(format))))
}
func (key *ContactPublicKey) Fingerprint() ([]byte, error) {
	return takeBuffer(C.vault_key_fingerprint(key.handle))
}
func (key *ContactPublicKey) Encrypt(contentKey []byte) (*WrappedContactKey, error) {
	h := C.key_contact_encrypt(key.handle, bytePointer(contentKey), C.size_t(len(contentKey)))
	if h == nil {
		return nil, lastError()
	}
	return &WrappedContactKey{handle: h}, nil
}

// WrappedContactKey owns an encrypted content-key envelope for one recipient.
type WrappedContactKey struct{ handle unsafe.Pointer }

func (key *WrappedContactKey) Close() {
	if key != nil && key.handle != nil {
		C.key_contact_wrapped_free(key.handle)
		key.handle = nil
	}
}
func (key *WrappedContactKey) PublicBytes() ([]byte, error) {
	return takeBuffer(C.key_contact_wrapped_public(key.handle))
}
func (key *WrappedContactKey) Ciphertext() ([]byte, error) {
	return takeBuffer(C.key_contact_wrapped_ciphertext(key.handle))
}
func (key *WrappedContactKey) EncryptedBytes() ([]byte, error) {
	return takeBuffer(C.key_contact_wrapped_encrypted(key.handle))
}

// ContactKeyPair owns the private key used to decrypt received content keys.
type ContactKeyPair struct{ handle unsafe.Pointer }

func GenerateContactKeyPair() (*ContactKeyPair, error) {
	h := C.key_contact_generate()
	if h == nil {
		return nil, lastError()
	}
	return &ContactKeyPair{handle: h}, nil
}
func ContactKeyPairFromPrivate(value []byte) (*ContactKeyPair, error) {
	h := C.key_contact_from_private(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactKeyPair{handle: h}, nil
}
func ImportContactKeyPair(value []byte) (*ContactKeyPair, error) {
	h := C.vault_key_import_private(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactKeyPair{handle: h}, nil
}
func (key *ContactKeyPair) Close() {
	if key != nil && key.handle != nil {
		C.key_contact_free(key.handle)
		key.handle = nil
	}
}
func (key *ContactKeyPair) PublicBytes() ([]byte, error) {
	return takeBuffer(C.key_contact_public(key.handle))
}
func (key *ContactKeyPair) PrivateRecord() ([]byte, error) {
	return takeBuffer(C.key_contact_private(key.handle))
}
func (key *ContactKeyPair) PublicKey() (*ContactPublicKey, error) {
	value, err := key.PublicBytes()
	if err != nil {
		return nil, err
	}
	return NewContactPublicKey(value)
}
func (key *ContactKeyPair) Export(format string) ([]byte, error) {
	return takeBuffer(C.vault_key_export_private(key.handle, charPointer(format), C.size_t(len(format))))
}
func (key *ContactKeyPair) Decrypt(wrapped *WrappedContactKey) ([]byte, error) {
	return takeBuffer(C.key_contact_decrypt(key.handle, wrapped.handle))
}

// SigningPublicKey verifies owner-authorized lockbox commits.
type SigningPublicKey struct{ handle unsafe.Pointer }

func NewSigningPublicKey(value []byte) (*SigningPublicKey, error) {
	h := C.key_signing_public_from_bytes(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &SigningPublicKey{handle: h}, nil
}
func (key *SigningPublicKey) Close() {
	if key != nil && key.handle != nil {
		C.key_signing_public_free(key.handle)
		key.handle = nil
	}
}

// SigningKeyPair owns the private key used to authorize mutable lockbox commits.
type SigningKeyPair struct{ handle unsafe.Pointer }

func GenerateSigningKeyPair() (*SigningKeyPair, error) {
	h := C.key_signing_generate()
	if h == nil {
		return nil, lastError()
	}
	return &SigningKeyPair{handle: h}, nil
}
func SigningKeyPairFromPrivate(value []byte) (*SigningKeyPair, error) {
	h := C.key_signing_from_private(bytePointer(value), C.size_t(len(value)))
	if h == nil {
		return nil, lastError()
	}
	return &SigningKeyPair{handle: h}, nil
}
func (key *SigningKeyPair) Close() {
	if key != nil && key.handle != nil {
		C.key_signing_free(key.handle)
		key.handle = nil
	}
}
func (key *SigningKeyPair) PublicBytes() ([]byte, error) {
	return takeBuffer(C.key_signing_public(key.handle))
}
func (key *SigningKeyPair) PrivateRecord() ([]byte, error) {
	return takeBuffer(C.key_signing_private(key.handle))
}
func (key *SigningKeyPair) PublicKey() (*SigningPublicKey, error) {
	value, err := key.PublicBytes()
	if err != nil {
		return nil, err
	}
	return NewSigningPublicKey(value)
}

// LockboxOptions configures runtime cache and worker behavior.
type LockboxOptions struct {
	CacheMode  string
	CacheBytes uint64
	Workload   string
	Worker     string
	Jobs       uint
}

func DefaultLockboxOptions() LockboxOptions {
	return LockboxOptions{CacheMode: "bytes", CacheBytes: 64 << 20, Workload: "interactive", Worker: "auto"}
}

// Lockbox is an owned, mutable view of one encrypted archive.
// Call Close when it is no longer required.
type Lockbox struct{ handle unsafe.Pointer }

func adoptLockbox(handle unsafe.Pointer) (*Lockbox, error) {
	if handle == nil {
		return nil, lastError()
	}
	return &Lockbox{handle: handle}, nil
}
func Create(key []byte) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_create(bytePointer(key), C.size_t(len(key))))
}
func CreateWithOptions(key []byte, options LockboxOptions) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_create_with_options(bytePointer(key), C.size_t(len(key)), charPointer(options.CacheMode), C.size_t(len(options.CacheMode)), C.uint64_t(options.CacheBytes), charPointer(options.Workload), C.size_t(len(options.Workload)), charPointer(options.Worker), C.size_t(len(options.Worker)), C.size_t(options.Jobs)))
}
func CreateWithPassword(password []byte) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_create_password(bytePointer(password), C.size_t(len(password))))
}
func CreateForContact(contact *ContactPublicKey) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_create_contact(contact.handle))
}
func CreateSigned(contentKey []byte, signing *SigningKeyPair) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_create_with_signing_key(bytePointer(contentKey), C.size_t(len(contentKey)), signing.handle))
}
func Open(archive, key []byte) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_open(bytePointer(archive), C.size_t(len(archive)), bytePointer(key), C.size_t(len(key))))
}
func OpenWithOptions(archive, key []byte, options LockboxOptions) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_open_with_options(bytePointer(archive), C.size_t(len(archive)), bytePointer(key), C.size_t(len(key)), charPointer(options.CacheMode), C.size_t(len(options.CacheMode)), C.uint64_t(options.CacheBytes), charPointer(options.Workload), C.size_t(len(options.Workload)), charPointer(options.Worker), C.size_t(len(options.Worker)), C.size_t(options.Jobs)))
}
func OpenWithPassword(archive, password []byte) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_open_password(bytePointer(archive), C.size_t(len(archive)), bytePointer(password), C.size_t(len(password))))
}
func OpenWithContact(archive []byte, contact *ContactKeyPair) (*Lockbox, error) {
	return adoptLockbox(C.lockbox_open_contact(bytePointer(archive), C.size_t(len(archive)), contact.handle))
}
func (box *Lockbox) Close() {
	if box != nil && box.handle != nil {
		C.lockbox_free(box.handle)
		box.handle = nil
	}
}
func (box *Lockbox) AddFile(path string, value []byte, replace bool) error {
	return require(bool(C.lockbox_add_file(box.handle, charPointer(path), C.size_t(len(path)), bytePointer(value), C.size_t(len(value)), C.bool(replace))))
}
func (box *Lockbox) AddFileWithPermissions(path string, value []byte, permissions uint32, replace bool) error {
	return require(bool(C.lockbox_add_file_with_permissions(box.handle, charPointer(path), C.size_t(len(path)), bytePointer(value), C.size_t(len(value)), C.uint32_t(permissions), C.bool(replace))))
}
func (box *Lockbox) GetFile(path string) ([]byte, error) {
	return takeBuffer(C.lockbox_get_file(box.handle, charPointer(path), C.size_t(len(path))))
}
func (box *Lockbox) Commit() error { return require(bool(C.lockbox_commit(box.handle))) }
func (box *Lockbox) CreateDirectory(path string, parents bool) error {
	return require(bool(C.lockbox_create_dir(box.handle, charPointer(path), C.size_t(len(path)), C.bool(parents))))
}
func (box *Lockbox) Delete(path string) error {
	return require(bool(C.lockbox_delete(box.handle, charPointer(path), C.size_t(len(path)))))
}
func (box *Lockbox) RemoveDirectory(path string, recursive bool) error {
	return require(bool(C.lockbox_remove_dir(box.handle, charPointer(path), C.size_t(len(path)), C.bool(recursive))))
}
func (box *Lockbox) CreateParentDirectories(path string) error {
	return require(bool(C.lockbox_create_parent_dirs(box.handle, charPointer(path), C.size_t(len(path)))))
}
func (box *Lockbox) Rename(from, to string) error {
	return require(bool(C.lockbox_rename(box.handle, charPointer(from), C.size_t(len(from)), charPointer(to), C.size_t(len(to)))))
}
func (box *Lockbox) Exists(path string) bool {
	return bool(C.lockbox_exists(box.handle, charPointer(path), C.size_t(len(path))))
}
func (box *Lockbox) IsDirectory(path string) bool {
	return bool(C.lockbox_is_dir(box.handle, charPointer(path), C.size_t(len(path))))
}
func (box *Lockbox) Permissions(path string) uint32 {
	return uint32(C.lockbox_permissions(box.handle, charPointer(path), C.size_t(len(path))))
}
func (box *Lockbox) SetPermissions(path string, value uint32) error {
	return require(bool(C.lockbox_set_permissions(box.handle, charPointer(path), C.size_t(len(path)), C.uint32_t(value))))
}
func (box *Lockbox) ReadRange(path string, offset, length uint64) ([]byte, error) {
	return takeBuffer(C.lockbox_read_range(box.handle, charPointer(path), C.size_t(len(path)), C.uint64_t(offset), C.uint64_t(length)))
}
func (box *Lockbox) StorageLength() uint64  { return uint64(C.lockbox_storage_len(box.handle)) }
func (box *Lockbox) Bytes() ([]byte, error) { return takeBuffer(C.lockbox_to_bytes(box.handle)) }
func (box *Lockbox) ExtractFile(source, destination string, replace bool) error {
	return require(bool(C.lockbox_extract_file(box.handle, charPointer(source), C.size_t(len(source)), charPointer(destination), C.size_t(len(destination)), C.bool(replace))))
}
func (box *Lockbox) ExtractDirectory(destination string, maxFileBytes, maxTotalBytes uint64, maxFiles uint, restoreSymlinks, restorePermissions, overwrite bool) error {
	return require(bool(C.lockbox_extract_directory(box.handle, charPointer(destination), C.size_t(len(destination)), C.uint64_t(maxFileBytes), C.uint64_t(maxTotalBytes), C.size_t(maxFiles), C.bool(restoreSymlinks), C.bool(restorePermissions), C.bool(overwrite))))
}
func (box *Lockbox) StreamContent(physical bool) (*messages.StreamChunkList, error) {
	result := &messages.StreamChunkList{}
	return result, decodeFrame(C.lockbox_stream_content(box.handle, C.bool(physical)), result)
}
func (box *Lockbox) CacheStats() (*messages.CacheStats, error) {
	result := &messages.CacheStats{}
	return result, decodeFrame(C.lockbox_cache_stats(box.handle), result)
}
func (box *Lockbox) ImportStats() (*messages.ImportStats, error) {
	result := &messages.ImportStats{}
	return result, decodeFrame(C.lockbox_import_stats(box.handle), result)
}
func (box *Lockbox) ResetImportStats() error {
	return require(bool(C.lockbox_reset_import_stats(box.handle)))
}
func InspectLockboxFile(path string) (*messages.FileInspection, error) {
	result := &messages.FileInspection{}
	return result, decodeFrame(C.lockbox_inspect_file(charPointer(path), C.size_t(len(path))), result)
}
func (box *Lockbox) PageInspection() (*messages.PageInspectionList, error) {
	result := &messages.PageInspectionList{}
	return result, decodeFrame(C.lockbox_page_inspection(box.handle), result)
}
func (box *Lockbox) RecoveryReport() (*messages.RecoveryReport, error) {
	result := &messages.RecoveryReport{}
	return result, decodeFrame(C.lockbox_recovery_report(box.handle), result)
}
func (box *Lockbox) RenderRecoveryReport(verbose bool, maxEntries uint) (string, error) {
	return takeString(C.lockbox_recovery_report_render(box.handle, C.bool(verbose), C.size_t(maxEntries)))
}
func ScanLockboxPath(path string, key []byte) (*messages.RecoveryReport, error) {
	result := &messages.RecoveryReport{}
	return result, decodeFrame(C.lockbox_recovery_scan_path(charPointer(path), C.size_t(len(path)), bytePointer(key), C.size_t(len(key))), result)
}
func ScanLockbox(archive, key []byte) (*messages.RecoveryReport, error) {
	result := &messages.RecoveryReport{}
	return result, decodeFrame(C.lockbox_recovery_scan(bytePointer(archive), C.size_t(len(archive)), bytePointer(key), C.size_t(len(key))), result)
}
func SalvageLockbox(archive, key []byte, signing *SigningKeyPair) (*Lockbox, error) {
	var signingHandle unsafe.Pointer
	if signing != nil {
		signingHandle = signing.handle
	}
	return adoptLockbox(C.lockbox_recovery_salvage(bytePointer(archive), C.size_t(len(archive)), bytePointer(key), C.size_t(len(key)), signingHandle))
}
func (box *Lockbox) SetWorkloadProfile(profile string) error {
	return require(bool(C.lockbox_set_workload_profile(box.handle, charPointer(profile), C.size_t(len(profile)))))
}
func (box *Lockbox) SetWorkerPolicy(mode string, jobs uint) error {
	return require(bool(C.lockbox_set_worker_policy(box.handle, charPointer(mode), C.size_t(len(mode)), C.size_t(jobs))))
}
func (box *Lockbox) RuntimeOptions() (*messages.RuntimeOptions, error) {
	result := &messages.RuntimeOptions{}
	return result, decodeFrame(C.lockbox_runtime_options(box.handle), result)
}
func (box *Lockbox) List(path string, recursive bool) (*messages.LockboxEntryList, error) {
	result := &messages.LockboxEntryList{}
	return result, decodeFrame(C.lockbox_list(box.handle, charPointer(path), C.size_t(len(path)), C.bool(recursive)), result)
}
func (box *Lockbox) ListWithOptions(path, glob string, recursive, includeFiles, includeSymlinks, includeDirectories bool, limit uint) (*messages.LockboxEntryList, error) {
	result := &messages.LockboxEntryList{}
	return result, decodeFrame(C.lockbox_list_with_options(box.handle, charPointer(path), C.size_t(len(path)), charPointer(glob), C.size_t(len(glob)), C.bool(recursive), C.bool(includeFiles), C.bool(includeSymlinks), C.bool(includeDirectories), C.size_t(limit)), result)
}
func (box *Lockbox) Stat(path string) (*messages.OptionalLockboxEntry, error) {
	result := &messages.OptionalLockboxEntry{}
	return result, decodeFrame(C.lockbox_stat(box.handle, charPointer(path), C.size_t(len(path))), result)
}
func (box *Lockbox) SetVariable(name, value string) error {
	return require(bool(C.lockbox_set_variable(box.handle, charPointer(name), C.size_t(len(name)), charPointer(value), C.size_t(len(value)))))
}
func (box *Lockbox) SetSecretVariable(name string, value []byte) error {
	return require(bool(C.lockbox_set_secret_variable(box.handle, charPointer(name), C.size_t(len(name)), bytePointer(value), C.size_t(len(value)))))
}
func (box *Lockbox) GetVariable(name string) (*string, error) {
	result := &messages.OptionalString{}
	if err := decodeFrame(C.lockbox_get_variable(box.handle, charPointer(name), C.size_t(len(name))), result); err != nil {
		return nil, err
	}
	if !result.Present {
		return nil, nil
	}
	return &result.Value, nil
}
func (box *Lockbox) WithSecretVariable(name string, callback func([]byte) error) error {
	return withSecret(func(output *unsafe.Pointer) bool {
		return bool(C.lockbox_get_secret_variable(box.handle, charPointer(name), C.size_t(len(name)), output))
	}, callback)
}
func (box *Lockbox) DeleteVariable(name string) error {
	return require(bool(C.lockbox_delete_variable(box.handle, charPointer(name), C.size_t(len(name)))))
}
func (box *Lockbox) MoveVariables(moves *messages.PathMoveList) error {
	encoded, err := proto.Marshal(moves); if err != nil { return err }
	return require(bool(C.lockbox_move_variables(box.handle, bytePointer(encoded), C.size_t(len(encoded)))))
}
func (box *Lockbox) ListVariables() (*messages.VariableList, error) {
	result := &messages.VariableList{}
	return result, decodeFrame(C.lockbox_list_variables(box.handle), result)
}
func (box *Lockbox) VariableSensitivity(name string) (*messages.OptionalString, error) {
	result := &messages.OptionalString{}
	return result, decodeFrame(C.lockbox_variable_sensitivity(box.handle, charPointer(name), C.size_t(len(name))), result)
}
func (box *Lockbox) AddSymlink(path, target string, replace bool) error {
	return require(bool(C.lockbox_add_symlink(box.handle, charPointer(path), C.size_t(len(path)), charPointer(target), C.size_t(len(target)), C.bool(replace))))
}
func (box *Lockbox) SymlinkTarget(path string) (string, error) {
	return takeString(C.lockbox_get_symlink_target(box.handle, charPointer(path), C.size_t(len(path))))
}
func (box *Lockbox) ID() ([]byte, error) { return takeBuffer(C.lockbox_id(box.handle)) }
func (box *Lockbox) AddPassword(password []byte) (uint64, error) {
	id := uint64(C.lockbox_add_password(box.handle, bytePointer(password), C.size_t(len(password))))
	if id == ^uint64(0) {
		return 0, lastError()
	}
	return id, nil
}
func (box *Lockbox) AddContact(contact *ContactPublicKey, name string) (uint64, error) {
	id := uint64(C.lockbox_add_contact(box.handle, contact.handle, charPointer(name), C.size_t(len(name))))
	if id == ^uint64(0) {
		return 0, lastError()
	}
	return id, nil
}
func (box *Lockbox) DeleteKey(id uint64) error {
	return require(bool(C.lockbox_delete_key(box.handle, C.uint64_t(id))))
}
func (box *Lockbox) ListKeySlots() (*messages.KeySlotList, error) {
	result := &messages.KeySlotList{}
	return result, decodeFrame(C.lockbox_list_key_slots(box.handle), result)
}
func (box *Lockbox) SetOwnerSigningKey(key *SigningKeyPair) error {
	return require(bool(C.lockbox_set_owner_signing_key(box.handle, key.handle)))
}
func (box *Lockbox) OwnerInspection() (*messages.OwnerInspection, error) {
	result := &messages.OwnerInspection{}
	return result, decodeFrame(C.lockbox_owner_inspection(box.handle), result)
}
func (box *Lockbox) DefineForm(alias, name, description string, fields *messages.FormFieldList) (*messages.FormDefinition, error) {
	encoded, err := proto.Marshal(fields)
	if err != nil {
		return nil, err
	}
	result := &messages.FormDefinition{}
	return result, decodeFrame(C.lockbox_define_form(box.handle, charPointer(alias), C.size_t(len(alias)), charPointer(name), C.size_t(len(name)), charPointer(description), C.size_t(len(description)), bytePointer(encoded), C.size_t(len(encoded))), result)
}
func (box *Lockbox) ListFormDefinitions() (*messages.FormDefinitionList, error) {
	result := &messages.FormDefinitionList{}
	return result, decodeFrame(C.lockbox_list_form_definitions(box.handle), result)
}
func (box *Lockbox) ResolveForm(reference string) (*messages.FormDefinition, error) {
	result := &messages.FormDefinition{}
	return result, decodeFrame(C.lockbox_resolve_form(box.handle, charPointer(reference), C.size_t(len(reference))), result)
}
func (box *Lockbox) ListFormRevisions(typeID string) (*messages.FormDefinitionList, error) {
	result := &messages.FormDefinitionList{}
	return result, decodeFrame(C.lockbox_list_form_revisions(box.handle, charPointer(typeID), C.size_t(len(typeID))), result)
}
func (box *Lockbox) CreateFormRecord(path, typeReference, name string) (*messages.FormRecord, error) {
	result := &messages.FormRecord{}
	return result, decodeFrame(C.lockbox_create_form_record(box.handle, charPointer(path), C.size_t(len(path)), charPointer(typeReference), C.size_t(len(typeReference)), charPointer(name), C.size_t(len(name))), result)
}
func (box *Lockbox) SetFormField(path, field, value string) error {
	return require(bool(C.lockbox_set_form_field(box.handle, charPointer(path), C.size_t(len(path)), charPointer(field), C.size_t(len(field)), charPointer(value), C.size_t(len(value)))))
}
func (box *Lockbox) SetSecretFormField(path, field string, value []byte) error {
	return require(bool(C.lockbox_set_secret_form_field(box.handle, charPointer(path), C.size_t(len(path)), charPointer(field), C.size_t(len(field)), bytePointer(value), C.size_t(len(value)))))
}
func (box *Lockbox) ListFormRecords() (*messages.FormRecordList, error) {
	result := &messages.FormRecordList{}
	return result, decodeFrame(C.lockbox_list_form_records(box.handle), result)
}
func (box *Lockbox) GetFormRecord(path string) (*messages.OptionalFormRecord, error) {
	result := &messages.OptionalFormRecord{}
	return result, decodeFrame(C.lockbox_get_form_record(box.handle, charPointer(path), C.size_t(len(path))), result)
}
func (box *Lockbox) DeleteFormRecord(path string) error {
	return require(bool(C.lockbox_delete_form_record(box.handle, charPointer(path), C.size_t(len(path)))))
}
func (box *Lockbox) MoveFormRecords(moves *messages.PathMoveList) error {
	encoded, err := proto.Marshal(moves); if err != nil { return err }
	return require(bool(C.lockbox_move_form_records(box.handle, bytePointer(encoded), C.size_t(len(encoded)))))
}
func (box *Lockbox) GetFormField(path, field string) (*messages.OptionalFormValue, error) {
	result := &messages.OptionalFormValue{}
	return result, decodeFrame(C.lockbox_get_form_field(box.handle, charPointer(path), C.size_t(len(path)), charPointer(field), C.size_t(len(field))), result)
}
func (box *Lockbox) WithSecretFormField(path, field string, callback func([]byte) error) error {
	return withSecret(func(output *unsafe.Pointer) bool {
		return bool(C.lockbox_get_secret_form_field(box.handle, charPointer(path), C.size_t(len(path)), charPointer(field), C.size_t(len(field)), output))
	}, callback)
}

func FormatKeyHex(value []byte) (string, error) {
	return takeString(C.vault_key_format_hex(bytePointer(value), C.size_t(len(value))))
}
func DecodeKeyHex(value string) ([]byte, error) {
	return takeBuffer(C.vault_key_decode_hex(charPointer(value), C.size_t(len(value))))
}
func FormatKeyCrockford(value []byte) (string, error) {
	return takeString(C.vault_key_format_crockford(bytePointer(value), C.size_t(len(value))))
}
func FormatKeyCrockfordReading(value string) (string, error) {
	return takeString(C.vault_key_format_crockford_reading(charPointer(value), C.size_t(len(value))))
}
func DecodeKeyCrockford(value string) ([]byte, error) {
	return takeBuffer(C.vault_key_decode_crockford(charPointer(value), C.size_t(len(value))))
}
func HexEncode(value []byte) (string, error) {
	return takeString(C.vault_key_hex_encode(bytePointer(value), C.size_t(len(value))))
}
func HexDecode(value string) ([]byte, error) {
	return takeBuffer(C.vault_key_hex_decode(charPointer(value), C.size_t(len(value))))
}

// VaultDirectory is a writable, password-protected local metadata vault.
type VaultDirectory struct{ handle unsafe.Pointer }

func adoptVaultDirectory(handle unsafe.Pointer) (*VaultDirectory, error) {
	if handle == nil {
		return nil, lastError()
	}
	return &VaultDirectory{handle: handle}, nil
}
func OpenVaultDirectory(root string, password []byte) (*VaultDirectory, error) {
	return adoptVaultDirectory(C.vault_directory_open(charPointer(root), C.size_t(len(root)), bytePointer(password), C.size_t(len(password))))
}
func OpenOrCreateVaultDirectory(root string, password []byte) (*VaultDirectory, error) {
	return adoptVaultDirectory(C.vault_directory_open_or_create(charPointer(root), C.size_t(len(root)), bytePointer(password), C.size_t(len(password))))
}
func ReplaceVaultDirectory(root string, password []byte) (*VaultDirectory, error) {
	return adoptVaultDirectory(C.vault_directory_replace(charPointer(root), C.size_t(len(root)), bytePointer(password), C.size_t(len(password))))
}
func OpenOrCreateDefaultVaultDirectory(password []byte) (*VaultDirectory, error) {
	return adoptVaultDirectory(C.vault_directory_open_or_create_default(bytePointer(password), C.size_t(len(password))))
}
func ReplaceDefaultVaultDirectory(password []byte) (*VaultDirectory, error) {
	return adoptVaultDirectory(C.vault_directory_replace_default(bytePointer(password), C.size_t(len(password))))
}
func ChangeVaultDirectoryPassword(root string, oldPassword, newPassword []byte) error {
	return require(bool(C.vault_directory_change_password(charPointer(root), C.size_t(len(root)), bytePointer(oldPassword), C.size_t(len(oldPassword)), bytePointer(newPassword), C.size_t(len(newPassword)))))
}
func ChangeDefaultVaultDirectoryPassword(oldPassword, newPassword []byte) error {
	return require(bool(C.vault_directory_change_default_password(bytePointer(oldPassword), C.size_t(len(oldPassword)), bytePointer(newPassword), C.size_t(len(newPassword)))))
}
func DefaultVaultDirectory() (string, error) { return takeString(C.vault_default_directory()) }
func DefaultVaultPath() (string, error)      { return takeString(C.vault_default_path()) }
func BackupDefaultVault(path string, overwrite bool) (*messages.VaultBackupManifest, error) {
	result := &messages.VaultBackupManifest{}
	return result, decodeFrame(C.vault_backup_default(charPointer(path), C.size_t(len(path)), C.bool(overwrite)), result)
}
func RestoreDefaultVault(path string, overwrite bool) (*messages.VaultBackupManifest, error) {
	result := &messages.VaultBackupManifest{}
	return result, decodeFrame(C.vault_restore_default(charPointer(path), C.size_t(len(path)), C.bool(overwrite)), result)
}
func (vault *VaultDirectory) Close() {
	if vault != nil && vault.handle != nil {
		C.vault_directory_free(vault.handle)
		vault.handle = nil
	}
}
func (vault *VaultDirectory) Root() (string, error) {
	return takeString(C.vault_directory_root(vault.handle))
}
func (vault *VaultDirectory) StructureVersion() uint32 {
	return uint32(C.vault_directory_structure_version(vault.handle))
}
func (vault *VaultDirectory) ListPrivateKeys() (*messages.StringList, error) {
	result := &messages.StringList{}
	return result, decodeFrame(C.vault_directory_list_private_keys(vault.handle), result)
}
func (vault *VaultDirectory) ListPrivateKeyNames() (*messages.StringList, error) {
	result := &messages.StringList{}
	return result, decodeFrame(C.vault_directory_list_private_key_names(vault.handle), result)
}
func (vault *VaultDirectory) ListContactNames() (*messages.StringList, error) {
	result := &messages.StringList{}
	return result, decodeFrame(C.vault_directory_list_contact_names(vault.handle), result)
}
func (vault *VaultDirectory) ListFormAliases() (*messages.StringList, error) {
	result := &messages.StringList{}
	return result, decodeFrame(C.vault_directory_list_form_aliases(vault.handle), result)
}
func (vault *VaultDirectory) PrivateKeyExists(name string) bool {
	return bool(C.vault_directory_private_key_exists(vault.handle, charPointer(name), C.size_t(len(name))))
}
func (vault *VaultDirectory) DeletePrivateKey(name string) error {
	return require(bool(C.vault_directory_delete_private_key(vault.handle, charPointer(name), C.size_t(len(name)))))
}
func (vault *VaultDirectory) StorePrivateKey(name string, key *ContactKeyPair) error {
	return require(bool(C.vault_directory_store_private_key(vault.handle, charPointer(name), C.size_t(len(name)), key.handle)))
}
func (vault *VaultDirectory) LoadPrivateKey(name string) (*ContactKeyPair, error) {
	h := C.vault_directory_load_private_key(vault.handle, charPointer(name), C.size_t(len(name)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactKeyPair{handle: h}, nil
}
func (vault *VaultDirectory) LoadPrivateKeyGeneration(name string, index uint16) (*ContactKeyPair, error) {
	h := C.vault_directory_load_private_key_generation(vault.handle, charPointer(name), C.size_t(len(name)), C.uint16_t(index))
	if h == nil {
		return nil, lastError()
	}
	return &ContactKeyPair{handle: h}, nil
}
func (vault *VaultDirectory) StoreContact(name string, key *ContactPublicKey) error {
	return require(bool(C.vault_directory_store_contact(vault.handle, charPointer(name), C.size_t(len(name)), key.handle)))
}
func (vault *VaultDirectory) LoadContact(name string) (*ContactPublicKey, error) {
	h := C.vault_directory_load_contact(vault.handle, charPointer(name), C.size_t(len(name)))
	if h == nil {
		return nil, lastError()
	}
	return &ContactPublicKey{handle: h}, nil
}
func (vault *VaultDirectory) ContactExists(name string) bool {
	return bool(C.vault_directory_contact_exists(vault.handle, charPointer(name), C.size_t(len(name))))
}
func (vault *VaultDirectory) DeleteContact(name string) error {
	return require(bool(C.vault_directory_delete_contact(vault.handle, charPointer(name), C.size_t(len(name)))))
}
func (vault *VaultDirectory) ListContacts() (*messages.ContactList, error) {
	result := &messages.ContactList{}
	return result, decodeFrame(C.vault_directory_list_contacts(vault.handle), result)
}
func (vault *VaultDirectory) StoreProfileEmail(name, email string) error {
	return require(bool(C.vault_directory_store_profile_email(vault.handle, charPointer(name), C.size_t(len(name)), charPointer(email), C.size_t(len(email)))))
}
func (vault *VaultDirectory) ProfileEmail(name string) (*messages.OptionalString, error) {
	result := &messages.OptionalString{}
	return result, decodeFrame(C.vault_directory_profile_email(vault.handle, charPointer(name), C.size_t(len(name))), result)
}
func (vault *VaultDirectory) StoreBackup(id, value []byte) error {
	return require(bool(C.vault_directory_store_backup(vault.handle, bytePointer(id), C.size_t(len(id)), bytePointer(value), C.size_t(len(value)))))
}
func (vault *VaultDirectory) LoadBackup(id []byte) ([]byte, error) {
	return takeBuffer(C.vault_directory_load_backup(vault.handle, bytePointer(id), C.size_t(len(id))))
}
func (vault *VaultDirectory) BackupCount() uint64 {
	return uint64(C.vault_directory_backup_count(vault.handle))
}
func (vault *VaultDirectory) RestorePrivateKey(name string, key *ContactKeyPair, signing *SigningKeyPair, overwrite bool) error {
	return require(bool(C.vault_directory_restore_private_key(vault.handle, charPointer(name), C.size_t(len(name)), key.handle, signing.handle, C.bool(overwrite))))
}
func (vault *VaultDirectory) LoadOwnerSigningKey(name string) (*SigningKeyPair, error) {
	h := C.vault_directory_load_owner_signing_key(vault.handle, charPointer(name), C.size_t(len(name)))
	if h == nil {
		return nil, lastError()
	}
	return &SigningKeyPair{handle: h}, nil
}
func (vault *VaultDirectory) LoadOwnerSigningKeyGeneration(name string, index uint16) (*SigningKeyPair, error) {
	h := C.vault_directory_load_owner_signing_key_generation(vault.handle, charPointer(name), C.size_t(len(name)), C.uint16_t(index))
	if h == nil {
		return nil, lastError()
	}
	return &SigningKeyPair{handle: h}, nil
}
func (vault *VaultDirectory) StoreContactSigningKey(name string, key *SigningPublicKey) error {
	return require(bool(C.vault_directory_store_contact_signing_key(vault.handle, charPointer(name), C.size_t(len(name)), key.handle)))
}
func (vault *VaultDirectory) LoadContactSigningKey(name string) (*SigningPublicKey, error) {
	h := C.vault_directory_load_contact_signing_key(vault.handle, charPointer(name), C.size_t(len(name)))
	if h == nil {
		return nil, lastError()
	}
	return &SigningPublicKey{handle: h}, nil
}
func (vault *VaultDirectory) ListProfileGenerations(name string) (*messages.ProfileHistory, error) {
	result := &messages.ProfileHistory{}
	return result, decodeFrame(C.vault_directory_list_profile_generations(vault.handle, charPointer(name), C.size_t(len(name))), result)
}
func (vault *VaultDirectory) RotatePrivateKey(name string) (*messages.ProfileHistory, error) {
	result := &messages.ProfileHistory{}
	return result, decodeFrame(C.vault_directory_rotate_private_key(vault.handle, charPointer(name), C.size_t(len(name))), result)
}
func (vault *VaultDirectory) RememberLockbox(id []byte, path string) error {
	return require(bool(C.vault_directory_remember_lockbox(vault.handle, bytePointer(id), C.size_t(len(id)), charPointer(path), C.size_t(len(path)))))
}
func (vault *VaultDirectory) ListKnownLockboxes() (*messages.KnownLockboxList, error) {
	result := &messages.KnownLockboxList{}
	return result, decodeFrame(C.vault_directory_list_known_lockboxes(vault.handle), result)
}
func (vault *VaultDirectory) ForgetLockbox(path string) error {
	return require(bool(C.vault_directory_forget_lockbox(vault.handle, charPointer(path), C.size_t(len(path)))))
}
func (vault *VaultDirectory) RememberAccessSlotLabel(id []byte, slotID uint64, name string) error {
	return require(bool(C.vault_directory_remember_access_slot_label(vault.handle, bytePointer(id), C.size_t(len(id)), C.uint64_t(slotID), charPointer(name), C.size_t(len(name)))))
}
func (vault *VaultDirectory) ListAccessSlotLabels(id []byte) (*messages.AccessSlotLabelList, error) {
	result := &messages.AccessSlotLabelList{}
	return result, decodeFrame(C.vault_directory_list_access_slot_labels(vault.handle, bytePointer(id), C.size_t(len(id))), result)
}
func (vault *VaultDirectory) FindAccessSlotLabels(id []byte, name string) (*messages.AccessSlotLabelList, error) {
	result := &messages.AccessSlotLabelList{}
	return result, decodeFrame(C.vault_directory_find_access_slot_labels(vault.handle, bytePointer(id), C.size_t(len(id)), charPointer(name), C.size_t(len(name))), result)
}
func (vault *VaultDirectory) ForgetAccessSlotLabel(id []byte, slotID uint64) error {
	return require(bool(C.vault_directory_forget_access_slot_label(vault.handle, bytePointer(id), C.size_t(len(id)), C.uint64_t(slotID))))
}
func (vault *VaultDirectory) DefineForm(alias, name, description string, fields *messages.FormFieldList) (*messages.FormDefinition, error) {
	encoded, err := proto.Marshal(fields)
	if err != nil {
		return nil, err
	}
	result := &messages.FormDefinition{}
	return result, decodeFrame(C.vault_directory_define_form(vault.handle, charPointer(alias), C.size_t(len(alias)), charPointer(name), C.size_t(len(name)), charPointer(description), C.size_t(len(description)), bytePointer(encoded), C.size_t(len(encoded))), result)
}
func (vault *VaultDirectory) ResolveForm(reference string) (*messages.FormDefinition, error) {
	result := &messages.FormDefinition{}
	return result, decodeFrame(C.vault_directory_resolve_form(vault.handle, charPointer(reference), C.size_t(len(reference))), result)
}
func (vault *VaultDirectory) ListForms() (*messages.FormDefinitionList, error) {
	result := &messages.FormDefinitionList{}
	return result, decodeFrame(C.vault_directory_list_forms(vault.handle), result)
}
func (vault *VaultDirectory) ListFormRevisions(typeID string) (*messages.FormDefinitionList, error) {
	result := &messages.FormDefinitionList{}
	return result, decodeFrame(C.vault_directory_list_form_revisions(vault.handle, charPointer(typeID), C.size_t(len(typeID))), result)
}

// ReadOnlyVaultDirectory never loads an owner signing key.
type ReadOnlyVaultDirectory struct{ handle unsafe.Pointer }
func OpenReadOnlyVaultDirectory(root string, password []byte) (*ReadOnlyVaultDirectory, error) {
	h := C.vault_read_only_open(charPointer(root), C.size_t(len(root)), bytePointer(password), C.size_t(len(password))); if h == nil { return nil, lastError() }; return &ReadOnlyVaultDirectory{handle:h}, nil
}
func OpenDefaultReadOnlyVaultDirectory(password []byte) (*ReadOnlyVaultDirectory, error) {
	h := C.vault_read_only_open_default(bytePointer(password), C.size_t(len(password))); if h == nil { return nil, lastError() }; return &ReadOnlyVaultDirectory{handle:h}, nil
}
func (vault *ReadOnlyVaultDirectory) ListProfileNames() (*messages.StringList, error) { result:=&messages.StringList{}; return result, decodeFrame(C.vault_read_only_list_profile_names(vault.handle), result) }
func (vault *ReadOnlyVaultDirectory) ListContactNames() (*messages.StringList, error) { result:=&messages.StringList{}; return result, decodeFrame(C.vault_read_only_list_contact_names(vault.handle), result) }
func (vault *ReadOnlyVaultDirectory) ListFormAliases() (*messages.StringList, error) { result:=&messages.StringList{}; return result, decodeFrame(C.vault_read_only_list_form_aliases(vault.handle), result) }
func (vault *ReadOnlyVaultDirectory) ListKnownLockboxes() (*messages.KnownLockboxList, error) { result:=&messages.KnownLockboxList{}; return result, decodeFrame(C.vault_read_only_list_known_lockboxes(vault.handle), result) }
func (vault *ReadOnlyVaultDirectory) Close() { if vault != nil && vault.handle != nil { C.vault_read_only_free(vault.handle); vault.handle=nil } }
func (vault *VaultDirectory) SeedForms() uint {
	return uint(C.vault_directory_seed_forms(vault.handle))
}
func (vault *VaultDirectory) RememberPassword(id, password []byte) error {
	return require(bool(C.vault_directory_remember_password(vault.handle, bytePointer(id), C.size_t(len(id)), bytePointer(password), C.size_t(len(password)))))
}
func (vault *VaultDirectory) RememberedPassword(id []byte) ([]byte, error) {
	return takeBuffer(C.vault_directory_remembered_password(vault.handle, bytePointer(id), C.size_t(len(id))))
}

func AgentIsRunning() bool         { return bool(C.vault_is_running()) }
func ServeAgent() error            { return require(bool(C.vault_agent_serve())) }
func VerifyAgentTransport() error  { return require(bool(C.vault_agent_verify_transport())) }
func ForgetAllAgentSecrets() error { return require(bool(C.vault_forget_all())) }
func StopAgent() error             { return require(bool(C.vault_agent_stop())) }
func StartAgent() error            { return require(bool(C.vault_agent_start())) }
func PutAgentKey(id, key []byte) error {
	return require(bool(C.vault_agent_put(bytePointer(id), C.size_t(len(id)), bytePointer(key), C.size_t(len(key)))))
}
func GetAgentKey(id []byte) ([]byte, error) {
	return takeBuffer(C.vault_agent_get(bytePointer(id), C.size_t(len(id))))
}
func ForgetAgentKey(id []byte) error {
	return require(bool(C.vault_agent_forget(bytePointer(id), C.size_t(len(id)))))
}
func ListAgentKeys() (*messages.AgentEntryList, error) {
	result := &messages.AgentEntryList{}
	return result, decodeFrame(C.vault_agent_list(), result)
}
func AgentSleepSupport() (*messages.SleepSupport, error) {
	result := &messages.SleepSupport{}
	return result, decodeFrame(C.vault_agent_sleep_support(), result)
}
func AgentLogPath() (string, error)        { return takeString(C.vault_agent_log_path()) }
func AgentLogDestination() (string, error) { return takeString(C.vault_agent_log_destination()) }
func PutAgentVaultUnlockKey(vaultID string, key []byte, ttlSeconds uint64) error {
	return require(bool(C.vault_agent_put_vault_unlock_key(charPointer(vaultID), C.size_t(len(vaultID)), bytePointer(key), C.size_t(len(key)), C.uint64_t(ttlSeconds))))
}
func GetAgentVaultUnlockKey(vaultID string) ([]byte, error) {
	return takeBuffer(C.vault_agent_get_vault_unlock_key(charPointer(vaultID), C.size_t(len(vaultID))))
}
func ForgetAgentVaultUnlockKey(vaultID string) error {
	return require(bool(C.vault_agent_forget_vault_unlock_key(charPointer(vaultID), C.size_t(len(vaultID)))))
}
func PutAgentOwnerSigningKey(vaultID, profile string, key *SigningKeyPair, ttlSeconds uint64) error {
	return require(bool(C.vault_agent_put_owner_signing_key(charPointer(vaultID), C.size_t(len(vaultID)), charPointer(profile), C.size_t(len(profile)), key.handle, C.uint64_t(ttlSeconds))))
}
func GetAgentOwnerSigningKey(vaultID, profile string) (*SigningKeyPair, error) {
	h := C.vault_agent_get_owner_signing_key(charPointer(vaultID), C.size_t(len(vaultID)), charPointer(profile), C.size_t(len(profile)))
	if h == nil {
		return nil, lastError()
	}
	return &SigningKeyPair{handle: h}, nil
}
func ForgetAgentOwnerSigningKey(vaultID, profile string) error {
	return require(bool(C.vault_agent_forget_owner_signing_key(charPointer(vaultID), C.size_t(len(vaultID)), charPointer(profile), C.size_t(len(profile)))))
}

// AgentActivity registers an operation that currently requires secret access.
type AgentActivity struct{ handle unsafe.Pointer }

func BeginAgentActivity(kind string) (*AgentActivity, error) {
	h := C.vault_agent_begin_activity(charPointer(kind), C.size_t(len(kind)))
	if h == nil {
		return nil, lastError()
	}
	return &AgentActivity{handle: h}, nil
}
func (activity *AgentActivity) Close() {
	if activity != nil && activity.handle != nil {
		C.vault_agent_end_activity(activity.handle)
		activity.handle = nil
	}
}

func PlatformStatus() (*messages.PlatformStatus, error) {
	result := &messages.PlatformStatus{}
	return result, decodeFrame(C.vault_platform_status(), result)
}
func SetPlatformScope(scope string) error {
	return require(bool(C.vault_platform_set_scope(charPointer(scope), C.size_t(len(scope)))))
}
func EnablePlatformStore() error  { return require(bool(C.vault_platform_enable())) }
func DisablePlatformStore() error { return require(bool(C.vault_platform_disable())) }
func PlatformStoreDisabled() bool { return bool(C.vault_platform_disabled()) }
func PutPlatformPassword(password []byte) error {
	return require(bool(C.vault_platform_put_password(bytePointer(password), C.size_t(len(password)))))
}
func GetPlatformPassword() ([]byte, error) { return takeBuffer(C.vault_platform_get_password()) }
func ForgetPlatformPassword() error        { return require(bool(C.vault_platform_forget_password())) }

// LocalVault provides workflows for local metadata and remembered lockboxes.
type LocalVault struct{ handle unsafe.Pointer }

func OpenLocalVault() (*LocalVault, error) {
	h := C.vault_local()
	if h == nil {
		return nil, lastError()
	}
	return &LocalVault{handle: h}, nil
}
func (vault *LocalVault) Close() {
	if vault != nil && vault.handle != nil {
		C.vault_free(vault.handle)
		vault.handle = nil
	}
}
func (vault *LocalVault) CreateWithPassword(path string, password []byte) (*Lockbox, error) {
	return adoptLockbox(C.vault_create_lockbox_password(vault.handle, charPointer(path), C.size_t(len(path)), bytePointer(password), C.size_t(len(password))))
}
func (vault *LocalVault) OpenWithPassword(path string, password []byte) (*Lockbox, error) {
	return adoptLockbox(C.vault_open_lockbox_password(vault.handle, charPointer(path), C.size_t(len(path)), bytePointer(password), C.size_t(len(password))))
}
func (vault *LocalVault) CreateWithContentKey(path string, key []byte, signing *SigningKeyPair) (*Lockbox, error) {
	return adoptLockbox(C.vault_create_lockbox_content_key(vault.handle, charPointer(path), C.size_t(len(path)), bytePointer(key), C.size_t(len(key)), signing.handle))
}
func (vault *LocalVault) OpenWithContentKey(path string, key []byte, signing *SigningKeyPair) (*Lockbox, error) {
	return adoptLockbox(C.vault_open_lockbox_content_key(vault.handle, charPointer(path), C.size_t(len(path)), bytePointer(key), C.size_t(len(key)), signing.handle))
}
func (vault *LocalVault) CreateForContact(path string, contact *ContactPublicKey, name string, signing *SigningKeyPair) (*Lockbox, error) {
	return adoptLockbox(C.vault_create_lockbox_contact(vault.handle, charPointer(path), C.size_t(len(path)), contact.handle, charPointer(name), C.size_t(len(name)), signing.handle))
}
func (vault *LocalVault) CachePassword(path string, password []byte, ttlSeconds uint64) error {
	return require(bool(C.vault_cache_lockbox_password(vault.handle, charPointer(path), C.size_t(len(path)), bytePointer(password), C.size_t(len(password)), C.uint64_t(ttlSeconds))))
}
func (vault *LocalVault) CloseLockbox(path string) error {
	return require(bool(C.vault_close_lockbox(vault.handle, charPointer(path), C.size_t(len(path)))))
}
func (vault *LocalVault) CloseAll() error { return require(bool(C.vault_close_all(vault.handle))) }
