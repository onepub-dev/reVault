/* @ts-self-types="./revault_wasm_bindings.d.ts" */

/**
 * WebAssembly-side dispatcher used by the full hosted API. Browser-only
 * lockbox methods can remain self-contained, while OS-backed vault, agent and
 * keyring operations are supplied by an explicit host adapter.
 */
class Runtime {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RuntimeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_runtime_free(ptr, 0);
    }
    /**
     * Validates an operation name and records one hosted dispatch.
     *
     * Unknown names are rejected before control reaches the native host.
     * @param {string} operation
     */
    before_call(operation) {
        const ptr0 = passStringToWasm0(operation, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.runtime_before_call(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Returns the number of successfully validated hosted calls.
     * @returns {number}
     */
    get calls() {
        const ret = wasm.runtime_calls(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Creates a dispatcher with no recorded calls.
     */
    constructor() {
        const ret = wasm.runtime_new();
        this.__wbg_ptr = ret;
        RuntimeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) Runtime.prototype[Symbol.dispose] = Runtime.prototype.free;
exports.Runtime = Runtime;

/**
 * Hybrid contact key pair used to wrap and unwrap lockbox content keys.
 */
class WasmContactKey {
    static __wrap(ptr) {
        const obj = Object.create(WasmContactKey.prototype);
        obj.__wbg_ptr = ptr;
        WasmContactKeyFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmContactKeyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcontactkey_free(ptr, 0);
    }
    /**
     * Imports a contact key pair from its encrypted private-key record bytes.
     * @param {Uint8Array} record
     * @returns {WasmContactKey}
     */
    static from_private_record(record) {
        const ptr0 = passArray8ToWasm0(record, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcontactkey_from_private_record(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmContactKey.__wrap(ret[0]);
    }
    /**
     * Generates a new contact key pair from the runtime random source.
     */
    constructor() {
        const ret = wasm.wasmcontactkey_generate();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmContactKeyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Exports the private-key record.
     *
     * The returned JavaScript bytes contain secret material and should be
     * persisted only in an appropriately protected vault.
     * @returns {Uint8Array}
     */
    private_record() {
        const ret = wasm.wasmcontactkey_private_record(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Exports the public key bytes safe to share with a sender.
     * @returns {Uint8Array}
     */
    public_key() {
        const ret = wasm.wasmcontactkey_public_key(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
}
if (Symbol.dispose) WasmContactKey.prototype[Symbol.dispose] = WasmContactKey.prototype.free;
exports.WasmContactKey = WasmContactKey;

/**
 * Metadata for one file, directory, or symbolic link in a lockbox.
 */
class WasmEntry {
    static __wrap(ptr) {
        const obj = Object.create(WasmEntry.prototype);
        obj.__wbg_ptr = ptr;
        WasmEntryFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmEntryFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmentry_free(ptr, 0);
    }
    /**
     * Returns `file`, `directory`, or `symlink`.
     * @returns {string}
     */
    get kind() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmentry_kind(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the logical file length in bytes.
     * @returns {bigint}
     */
    get length() {
        const ret = wasm.wasmentry_length(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Returns the absolute lockbox path.
     * @returns {string}
     */
    get path() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmentry_path(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the stored Unix permission bits.
     * @returns {number}
     */
    get permissions() {
        const ret = wasm.wasmentry_permissions(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmEntry.prototype[Symbol.dispose] = WasmEntry.prototype.free;
exports.WasmEntry = WasmEntry;

/**
 * Metadata for one password or contact access slot.
 */
class WasmKeySlot {
    static __wrap(ptr) {
        const obj = Object.create(WasmKeySlot.prototype);
        obj.__wbg_ptr = ptr;
        WasmKeySlotFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmKeySlotFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmkeyslot_free(ptr, 0);
    }
    /**
     * Returns the cryptographic algorithm name stored by the slot.
     * @returns {string}
     */
    get algorithm() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmkeyslot_algorithm(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the stable key-slot identifier.
     * @returns {bigint}
     */
    get id() {
        const ret = wasm.wasmkeyslot_id(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Returns the slot protection type, such as `password` or `contact`.
     * @returns {string}
     */
    get protection() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmkeyslot_protection(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmKeySlot.prototype[Symbol.dispose] = WasmKeySlot.prototype.free;
exports.WasmKeySlot = WasmKeySlot;

/**
 * In-memory encrypted lockbox exposed to JavaScript.
 */
class WasmLockbox {
    static __wrap(ptr) {
        const obj = Object.create(WasmLockbox.prototype);
        obj.__wbg_ptr = ptr;
        WasmLockboxFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLockboxFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmlockbox_free(ptr, 0);
    }
    /**
     * Adds a contact-recipient access slot and returns its stable id.
     * @param {Uint8Array} public_key
     * @returns {bigint}
     */
    add_contact(public_key) {
        const ptr0 = passArray8ToWasm0(public_key, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_add_contact(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Adds a file at `path`, optionally replacing an existing file.
     * @param {string} path
     * @param {Uint8Array} data
     * @param {boolean} replace
     */
    add_file(path, data, replace) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_add_file(this.__wbg_ptr, ptr0, len0, ptr1, len1, replace);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Adds a file and stores its Unix permission bits.
     * @param {string} path
     * @param {Uint8Array} data
     * @param {number} permissions
     * @param {boolean} replace
     */
    add_file_with_permissions(path, data, permissions, replace) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_add_file_with_permissions(this.__wbg_ptr, ptr0, len0, ptr1, len1, permissions, replace);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Adds a password access slot and returns its stable id.
     * @param {string} password
     * @returns {bigint}
     */
    add_password(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_add_password(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Adds a symbolic link whose target is another normalized lockbox path.
     * @param {string} path
     * @param {string} target
     * @param {boolean} replace
     */
    add_symlink(path, target, replace) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(target, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_add_symlink(this.__wbg_ptr, ptr0, len0, ptr1, len1, replace);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Stages removal of the encrypted description; call `commit`.
     * For example: `box.clear_description(); box.commit()`.
     */
    clear_description() {
        const ret = wasm.wasmlockbox_clear_description(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Authenticates and commits all pending mutations.
     */
    commit() {
        const ret = wasm.wasmlockbox_commit(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Creates an uncommitted lockbox using a raw content key.
     *
     * Add an access slot and call [`WasmLockbox::commit`] before sharing it.
     * @param {Uint8Array} key
     */
    constructor(key) {
        const ptr0 = passArray8ToWasm0(key, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_create(ptr0, len0);
        this.__wbg_ptr = ret;
        WasmLockboxFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Creates a directory, optionally creating missing ancestors.
     * @param {string} path
     * @param {boolean} create_parents
     */
    create_dir(path, create_parents) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_create_dir(this.__wbg_ptr, ptr0, len0, create_parents);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Creates every missing parent directory of `path`.
     * @param {string} path
     */
    create_parent_dirs(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_create_parent_dirs(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Creates an uncommitted lockbox with a generated key wrapped by a password.
     * @param {string} password
     * @returns {WasmLockbox}
     */
    static create_with_password(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_create_with_password(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmLockbox.__wrap(ret[0]);
    }
    /**
     * Deletes a file or symbolic link at `path`.
     * @param {string} path
     */
    delete(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_delete(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Deletes the access slot identified by `id`.
     * @param {bigint} id
     */
    delete_key(id) {
        const ret = wasm.wasmlockbox_delete_key(this.__wbg_ptr, id);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Deletes the variable named `name`.
     * @param {string} name
     */
    delete_variable(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_delete_variable(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Returns the encrypted lockbox description, or `undefined` when unset.
     * For example, set it, commit, then display `box.description` in JavaScript.
     * @returns {string | undefined}
     */
    get description() {
        const ret = wasm.wasmlockbox_description(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns whether any entry exists at `path`.
     * @param {string} path
     * @returns {boolean}
     */
    exists(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_exists(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Decrypts and returns the complete file at `path`.
     * @param {string} path
     * @returns {Uint8Array}
     */
    get_file(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_get_file(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * Returns the stored target of the symbolic link at `path`.
     * @param {string} path
     * @returns {string}
     */
    get_symlink_target(path) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmlockbox_get_symlink_target(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Returns a non-secret variable, or `undefined` when it does not exist.
     * @param {string} name
     * @returns {string | undefined}
     */
    get_variable(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_get_variable(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * Returns whether `path` names a directory.
     * @param {string} path
     * @returns {boolean}
     */
    is_dir(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_is_dir(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Lists entries beneath `path`, optionally including all descendants.
     * @param {string} path
     * @param {boolean} recursive
     * @returns {Array<any>}
     */
    list(path, recursive) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_list(this.__wbg_ptr, ptr0, len0, recursive);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Returns metadata for every access slot without exposing wrapped secrets.
     * @returns {Array<any>}
     */
    list_key_slots() {
        const ret = wasm.wasmlockbox_list_key_slots(this.__wbg_ptr);
        return ret;
    }
    /**
     * Lists variable names and sensitivity without returning their values.
     * @returns {Array<any>}
     */
    list_variables() {
        const ret = wasm.wasmlockbox_list_variables(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Returns the stable 16-byte lockbox identifier.
     * @returns {Uint8Array}
     */
    lockbox_id() {
        const ret = wasm.wasmlockbox_lockbox_id(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Moves one typed form record to another lockbox path.
     * @param {string} source
     * @param {string} destination
     */
    move_form_record(source, destination) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(destination, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_move_form_record(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Renames one variable atomically.
     * @param {string} source
     * @param {string} destination
     */
    move_variable(source, destination) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(destination, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_move_variable(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Opens lockbox bytes for writing using a raw content key.
     * @param {Uint8Array} bytes
     * @param {Uint8Array} key
     * @returns {WasmLockbox}
     */
    static open(bytes, key) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(key, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_open(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmLockbox.__wrap(ret[0]);
    }
    /**
     * Opens lockbox bytes for writing using a password access slot.
     * @param {Uint8Array} bytes
     * @param {string} password
     * @returns {WasmLockbox}
     */
    static open_with_password(bytes, password) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_open_with_password(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmLockbox.__wrap(ret[0]);
    }
    /**
     * Returns stored Unix permission bits, or `undefined` if absent.
     * @param {string} path
     * @returns {number | undefined}
     */
    permissions(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_permissions(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] === Number.MAX_SAFE_INTEGER ? undefined : ret[0];
    }
    /**
     * Reads at most `len` file bytes beginning at `offset`.
     * @param {string} path
     * @param {bigint} offset
     * @param {bigint} len
     * @returns {Uint8Array}
     */
    read_range(path, offset, len) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_read_range(this.__wbg_ptr, ptr0, len0, offset, len);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * Removes a directory, including descendants when `recursive` is true.
     * @param {string} path
     * @param {boolean} recursive
     */
    remove_dir(path, recursive) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_remove_dir(this.__wbg_ptr, ptr0, len0, recursive);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Moves one lockbox entry from `from` to `to`.
     * @param {string} from
     * @param {string} to
     */
    rename(from, to) {
        const ptr0 = passStringToWasm0(from, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(to, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_rename(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Stages encrypted description text; call `commit` to publish it.
     * For example: `box.set_description("Production credentials"); box.commit()`.
     * @param {string} description
     */
    set_description(description) {
        const ptr0 = passStringToWasm0(description, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_description(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Updates the stored Unix permission bits for an entry.
     * @param {string} path
     * @param {number} permissions
     */
    set_permissions(path, permissions) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_permissions(this.__wbg_ptr, ptr0, len0, permissions);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Stores a secret variable in secure memory until it is encrypted.
     * @param {string} name
     * @param {Uint8Array} value
     */
    set_secret_variable(name, value) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(value, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_secret_variable(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Stores a non-secret UTF-8 variable.
     * @param {string} name
     * @param {string} value
     */
    set_variable(name, value) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_variable(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Selects the `auto`, `single`, or `threads` worker policy.
     *
     * `jobs` is used only by the `threads` policy.
     * @param {string} policy
     * @param {number} jobs
     */
    set_worker_policy(policy, jobs) {
        const ptr0 = passStringToWasm0(policy, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_worker_policy(this.__wbg_ptr, ptr0, len0, jobs);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Selects an `interactive`, `bulk-import`, `read-mostly`, or `extract-many` profile.
     * @param {string} profile
     */
    set_workload_profile(profile) {
        const ptr0 = passStringToWasm0(profile, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_set_workload_profile(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Returns entry metadata, or JavaScript `null` when `path` is absent.
     * @param {string} path
     * @returns {any}
     */
    stat(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_stat(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Serializes the committed encrypted lockbox.
     *
     * Call [`WasmLockbox::commit`] first when the lockbox has pending changes.
     * @returns {Uint8Array}
     */
    to_bytes() {
        const ret = wasm.wasmlockbox_to_bytes(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Returns `normal`, `secret`, or `undefined` for an absent variable.
     * @param {string} name
     * @returns {string | undefined}
     */
    variable_sensitivity(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_variable_sensitivity(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * Calls `callback` with a temporary copy of a secret variable.
     *
     * The temporary `Uint8Array` is overwritten immediately after the callback
     * returns. Retaining a copy inside the callback is the caller's responsibility.
     * @param {string} name
     * @param {Function} callback
     * @returns {any}
     */
    with_secret_variable(name, callback) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmlockbox_with_secret_variable(this.__wbg_ptr, ptr0, len0, callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) WasmLockbox.prototype[Symbol.dispose] = WasmLockbox.prototype.free;
exports.WasmLockbox = WasmLockbox;

/**
 * Name and sensitivity metadata for a lockbox variable.
 */
class WasmVariable {
    static __wrap(ptr) {
        const obj = Object.create(WasmVariable.prototype);
        obj.__wbg_ptr = ptr;
        WasmVariableFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmVariableFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmvariable_free(ptr, 0);
    }
    /**
     * Returns the variable name.
     * @returns {string}
     */
    get name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmvariable_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns `normal` or `secret`.
     * @returns {string}
     */
    get sensitivity() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmvariable_sensitivity(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmVariable.prototype[Symbol.dispose] = WasmVariable.prototype.free;
exports.WasmVariable = WasmVariable;

/**
 * Decodes hexadecimal text, rejecting malformed input.
 * @param {string} value
 * @returns {Uint8Array}
 */
function decode_hex(value) {
    const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.decode_hex(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}
exports.decode_hex = decode_hex;

/**
 * Encodes bytes as lowercase hexadecimal text.
 * @param {Uint8Array} bytes
 * @returns {string}
 */
function encode_hex(bytes) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encode_hex(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}
exports.encode_hex = encode_hex;

/**
 * Returns the lockbox file-format version written by this build.
 * @returns {number}
 */
function lockbox_format_version() {
    const ret = wasm.lockbox_format_version();
    return ret;
}
exports.lockbox_format_version = lockbox_format_version;

/**
 * Reads a lockbox file-format version without decrypting the archive.
 * @param {Uint8Array} bytes
 * @returns {number}
 */
function probe_lockbox_format_version(bytes) {
    const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.probe_lockbox_format_version(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}
exports.probe_lockbox_format_version = probe_lockbox_format_version;

/**
 * Explicitly permits or rejects the weakened secure-memory implementation
 * required by WebAssembly runtimes.
 *
 * The default is `false`, so callers must acknowledge that browsers cannot
 * provide locked pages, guard pages, or dump/fork exclusion before creating
 * keys or lockboxes.
 * @param {boolean} allowed
 */
function set_weakened_allocation_allowed(allowed) {
    wasm.set_weakened_allocation_allowed(allowed);
}
exports.set_weakened_allocation_allowed = set_weakened_allocation_allowed;

/**
 * Returns whether the caller has explicitly enabled weakened secure memory.
 * @returns {boolean}
 */
function weakened_allocation_allowed() {
    const ret = wasm.weakened_allocation_allowed();
    return ret !== 0;
}
exports.weakened_allocation_allowed = weakened_allocation_allowed;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_9c31b086c2b26051: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_dfde26266607c996: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_getRandomValues_cc7f052a444bb2ce: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_length_56fcd3e2b7e0299d: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_310879b66b6e95e1: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_from_slice_269e35316ed2d061: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_push_b77c476b01548d0a: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_set_index_6216eb6926d2fbc6: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbg_wasmentry_new: function(arg0) {
            const ret = WasmEntry.__wrap(arg0);
            return ret;
        },
        __wbg_wasmkeyslot_new: function(arg0) {
            const ret = WasmKeySlot.__wrap(arg0);
            return ret;
        },
        __wbg_wasmvariable_new: function(arg0) {
            const ret = WasmVariable.__wrap(arg0);
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./revault_wasm_bindings_bg.js": import0,
    };
}

const RuntimeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_runtime_free(ptr, 1));
const WasmContactKeyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcontactkey_free(ptr, 1));
const WasmEntryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmentry_free(ptr, 1));
const WasmKeySlotFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmkeyslot_free(ptr, 1));
const WasmLockboxFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmlockbox_free(ptr, 1));
const WasmVariableFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmvariable_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

const wasmPath = `${__dirname}/revault_wasm_bindings_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
let wasmInstance = new WebAssembly.Instance(wasmModule, __wbg_get_imports());
let wasm = wasmInstance.exports;
wasm.__wbindgen_start();
