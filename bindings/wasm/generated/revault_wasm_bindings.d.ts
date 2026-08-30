/* tslint:disable */
/* eslint-disable */

/**
 * WebAssembly-side dispatcher used by the full hosted API. Browser-only
 * lockbox methods can remain self-contained, while OS-backed vault, agent and
 * keyring operations are supplied by an explicit host adapter.
 */
export class Runtime {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Validates an operation name and records one hosted dispatch.
     *
     * Unknown names are rejected before control reaches the native host.
     */
    before_call(operation: string): void;
    /**
     * Creates a dispatcher with no recorded calls.
     */
    constructor();
    /**
     * Returns the number of successfully validated hosted calls.
     */
    readonly calls: number;
}

/**
 * Hybrid contact key pair used to wrap and unwrap lockbox content keys.
 */
export class WasmContactKey {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Imports a contact key pair from its encrypted private-key record bytes.
     */
    static from_private_record(record: Uint8Array): WasmContactKey;
    /**
     * Generates a new contact key pair from the runtime random source.
     */
    constructor();
    /**
     * Exports the private-key record.
     *
     * The returned JavaScript bytes contain secret material and should be
     * persisted only in an appropriately protected vault.
     */
    private_record(): Uint8Array;
    /**
     * Exports the public key bytes safe to share with a sender.
     */
    public_key(): Uint8Array;
}

/**
 * Metadata for one file, directory, or symbolic link in a lockbox.
 */
export class WasmEntry {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns `file`, `directory`, or `symlink`.
     */
    readonly kind: string;
    /**
     * Returns the logical file length in bytes.
     */
    readonly length: bigint;
    /**
     * Returns the absolute lockbox path.
     */
    readonly path: string;
    /**
     * Returns the stored Unix permission bits.
     */
    readonly permissions: number;
}

/**
 * Metadata for one password or contact access slot.
 */
export class WasmKeySlot {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns the cryptographic algorithm name stored by the slot.
     */
    readonly algorithm: string;
    /**
     * Returns the stable key-slot identifier.
     */
    readonly id: bigint;
    /**
     * Returns the slot protection type, such as `password` or `contact`.
     */
    readonly protection: string;
}

/**
 * In-memory encrypted lockbox exposed to JavaScript.
 */
export class WasmLockbox {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a contact-recipient access slot and returns its stable id.
     */
    add_contact(public_key: Uint8Array): bigint;
    /**
     * Adds a file at `path`, optionally replacing an existing file.
     */
    add_file(path: string, data: Uint8Array, replace: boolean): void;
    /**
     * Adds a file and stores its Unix permission bits.
     */
    add_file_with_permissions(path: string, data: Uint8Array, permissions: number, replace: boolean): void;
    /**
     * Adds a password access slot and returns its stable id.
     */
    add_password(password: string): bigint;
    /**
     * Adds a symbolic link whose target is another normalized lockbox path.
     */
    add_symlink(path: string, target: string, replace: boolean): void;
    /**
     * Stages removal of the encrypted description; call `commit`.
     * For example: `box.clear_description(); box.commit()`.
     */
    clear_description(): void;
    /**
     * Authenticates and commits all pending mutations.
     */
    commit(): void;
    /**
     * Creates an uncommitted lockbox using a raw content key.
     *
     * Add an access slot and call [`WasmLockbox::commit`] before sharing it.
     */
    constructor(key: Uint8Array);
    /**
     * Creates a directory, optionally creating missing ancestors.
     */
    create_dir(path: string, create_parents: boolean): void;
    /**
     * Creates every missing parent directory of `path`.
     */
    create_parent_dirs(path: string): void;
    /**
     * Creates an uncommitted lockbox with a generated key wrapped by a password.
     */
    static create_with_password(password: string): WasmLockbox;
    /**
     * Deletes a file or symbolic link at `path`.
     */
    delete(path: string): void;
    /**
     * Deletes the access slot identified by `id`.
     */
    delete_key(id: bigint): void;
    /**
     * Deletes the variable named `name`.
     */
    delete_variable(name: string): void;
    /**
     * Returns whether any entry exists at `path`.
     */
    exists(path: string): boolean;
    /**
     * Decrypts and returns the complete file at `path`.
     */
    get_file(path: string): Uint8Array;
    /**
     * Returns the stored target of the symbolic link at `path`.
     */
    get_symlink_target(path: string): string;
    /**
     * Returns a non-secret variable, or `undefined` when it does not exist.
     */
    get_variable(name: string): string | undefined;
    /**
     * Returns whether `path` names a directory.
     */
    is_dir(path: string): boolean;
    /**
     * Lists entries beneath `path`, optionally including all descendants.
     */
    list(path: string, recursive: boolean): Array<any>;
    /**
     * Returns metadata for every access slot without exposing wrapped secrets.
     */
    list_key_slots(): Array<any>;
    /**
     * Lists variable names and sensitivity without returning their values.
     */
    list_variables(): Array<any>;
    /**
     * Returns the stable 16-byte lockbox identifier.
     */
    lockbox_id(): Uint8Array;
    /**
     * Moves one typed form record to another lockbox path.
     */
    move_form_record(source: string, destination: string): void;
    /**
     * Renames one variable atomically.
     */
    move_variable(source: string, destination: string): void;
    /**
     * Opens lockbox bytes for writing using a raw content key.
     */
    static open(bytes: Uint8Array, key: Uint8Array): WasmLockbox;
    /**
     * Opens lockbox bytes for writing using a password access slot.
     */
    static open_with_password(bytes: Uint8Array, password: string): WasmLockbox;
    /**
     * Returns stored Unix permission bits, or `undefined` if absent.
     */
    permissions(path: string): number | undefined;
    /**
     * Reads at most `len` file bytes beginning at `offset`.
     */
    read_range(path: string, offset: bigint, len: bigint): Uint8Array;
    /**
     * Removes a directory, including descendants when `recursive` is true.
     */
    remove_dir(path: string, recursive: boolean): void;
    /**
     * Moves one lockbox entry from `from` to `to`.
     */
    rename(from: string, to: string): void;
    /**
     * Stages encrypted description text; call `commit` to publish it.
     * For example: `box.set_description("Production credentials"); box.commit()`.
     */
    set_description(description: string): void;
    /**
     * Updates the stored Unix permission bits for an entry.
     */
    set_permissions(path: string, permissions: number): void;
    /**
     * Stores a secret variable in secure memory until it is encrypted.
     */
    set_secret_variable(name: string, value: Uint8Array): void;
    /**
     * Stores a non-secret UTF-8 variable.
     */
    set_variable(name: string, value: string): void;
    /**
     * Selects the `auto`, `single`, or `threads` worker policy.
     *
     * `jobs` is used only by the `threads` policy.
     */
    set_worker_policy(policy: string, jobs: number): void;
    /**
     * Selects an `interactive`, `bulk-import`, `read-mostly`, or `extract-many` profile.
     */
    set_workload_profile(profile: string): void;
    /**
     * Returns entry metadata, or JavaScript `null` when `path` is absent.
     */
    stat(path: string): any;
    /**
     * Serializes the committed encrypted lockbox.
     *
     * Call [`WasmLockbox::commit`] first when the lockbox has pending changes.
     */
    to_bytes(): Uint8Array;
    /**
     * Returns `normal`, `secret`, or `undefined` for an absent variable.
     */
    variable_sensitivity(name: string): string | undefined;
    /**
     * Calls `callback` with a temporary copy of a secret variable.
     *
     * The temporary `Uint8Array` is overwritten immediately after the callback
     * returns. Retaining a copy inside the callback is the caller's responsibility.
     */
    with_secret_variable(name: string, callback: Function): any;
    /**
     * Returns the encrypted lockbox description, or `undefined` when unset.
     * For example, set it, commit, then display `box.description` in JavaScript.
     */
    readonly description: string | undefined;
}

/**
 * Name and sensitivity metadata for a lockbox variable.
 */
export class WasmVariable {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns the variable name.
     */
    readonly name: string;
    /**
     * Returns `normal` or `secret`.
     */
    readonly sensitivity: string;
}

/**
 * Decodes hexadecimal text, rejecting malformed input.
 */
export function decode_hex(value: string): Uint8Array;

/**
 * Encodes bytes as lowercase hexadecimal text.
 */
export function encode_hex(bytes: Uint8Array): string;

/**
 * Returns the lockbox file-format version written by this build.
 */
export function lockbox_format_version(): number;

/**
 * Reads a lockbox file-format version without decrypting the archive.
 */
export function probe_lockbox_format_version(bytes: Uint8Array): number;

/**
 * Explicitly permits or rejects the weakened secure-memory implementation
 * required by WebAssembly runtimes.
 *
 * The default is `false`, so callers must acknowledge that browsers cannot
 * provide locked pages, guard pages, or dump/fork exclusion before creating
 * keys or lockboxes.
 */
export function set_weakened_allocation_allowed(allowed: boolean): void;

/**
 * Returns whether the caller has explicitly enabled weakened secure memory.
 */
export function weakened_allocation_allowed(): boolean;
