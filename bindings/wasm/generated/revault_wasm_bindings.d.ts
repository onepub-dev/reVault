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
    before_call(operation: string): void;
    constructor();
    readonly calls: number;
}

export class WasmContactKey {
    free(): void;
    [Symbol.dispose](): void;
    static from_private_record(record: Uint8Array): WasmContactKey;
    constructor();
    private_record(): Uint8Array;
    public_key(): Uint8Array;
}

export class WasmEntry {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly kind: string;
    readonly length: bigint;
    readonly path: string;
    readonly permissions: number;
}

export class WasmKeySlot {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly algorithm: string;
    readonly id: bigint;
    readonly protection: string;
}

export class WasmLockbox {
    free(): void;
    [Symbol.dispose](): void;
    add_contact(public_key: Uint8Array): bigint;
    add_file(path: string, data: Uint8Array, replace: boolean): void;
    add_file_with_permissions(path: string, data: Uint8Array, permissions: number, replace: boolean): void;
    add_password(password: string): bigint;
    add_symlink(path: string, target: string, replace: boolean): void;
    commit(): void;
    constructor(key: Uint8Array);
    create_dir(path: string, create_parents: boolean): void;
    create_parent_dirs(path: string): void;
    static create_with_password(password: string): WasmLockbox;
    delete(path: string): void;
    delete_key(id: bigint): void;
    delete_variable(name: string): void;
    exists(path: string): boolean;
    get_file(path: string): Uint8Array;
    get_symlink_target(path: string): string;
    get_variable(name: string): string | undefined;
    is_dir(path: string): boolean;
    list(path: string, recursive: boolean): Array<any>;
    list_key_slots(): Array<any>;
    list_variables(): Array<any>;
    lockbox_id(): Uint8Array;
    move_form_record(source: string, destination: string): void;
    move_variable(source: string, destination: string): void;
    static open(bytes: Uint8Array, key: Uint8Array): WasmLockbox;
    static open_with_password(bytes: Uint8Array, password: string): WasmLockbox;
    permissions(path: string): number | undefined;
    read_range(path: string, offset: bigint, len: bigint): Uint8Array;
    remove_dir(path: string, recursive: boolean): void;
    rename(from: string, to: string): void;
    set_permissions(path: string, permissions: number): void;
    set_secret_variable(name: string, value: Uint8Array): void;
    set_variable(name: string, value: string): void;
    set_worker_policy(policy: string, jobs: number): void;
    set_workload_profile(profile: string): void;
    stat(path: string): any;
    to_bytes(): Uint8Array;
    variable_sensitivity(name: string): string | undefined;
    with_secret_variable(name: string, callback: Function): any;
}

export class WasmVariable {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly name: string;
    readonly sensitivity: string;
}

export function decode_hex(value: string): Uint8Array;

export function encode_hex(bytes: Uint8Array): string;

export function lockbox_format_version(): number;

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
