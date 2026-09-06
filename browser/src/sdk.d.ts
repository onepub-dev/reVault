export type State = 'ok' | 'extension_missing' | 'native_helper_missing' | 'upgrade_required'
  | 'vault_locked' | 'approval_required' | 'denied' | 'expired' | 'unpaired_recipient'
  | 'lockbox_unavailable' | 'secure_store_unavailable' | 'invalid_request' | 'replayed'
  | 'cancelled' | 'busy' | 'internal_error';
export interface Result { state: State; envelope?: Uint8Array; }
export function getCapabilities(): Promise<Result>;
/** Input SignedUnlockRequest protobuf, output encrypted UnlockEnvelope protobuf. */
export function requestUnlock(serverRequest: Uint8Array): Promise<Result>;
export function cancelRequest(requestId: string): Promise<Result>;
