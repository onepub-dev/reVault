import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { test } from 'node:test';

const require = createRequire(import.meta.url);
const { WasmLockbox, set_weakened_allocation_allowed } =
  require('../../wasm/generated/revault_wasm_bindings.cjs');
// Portable WASM cannot provide the native locked-memory guarantees.
set_weakened_allocation_allowed(true);

function withBox(box, action) {
  let primary;
  try { return action(box); }
  catch (error) { primary = error; throw error; }
  finally {
    try { box.free(); }
    catch (cleanup) {
      if (primary === undefined) throw cleanup;
      console.error('WASM cleanup after primary failure:', cleanup);
    }
  }
}

test('generated WASM password archives reopen for writes without an owner key', () => {
  const password = 'portable password fixture';
  let bytes = withBox(WasmLockbox.create_with_password(password), box => {
    box.add_file('/keep', Buffer.from('original'), false);
    box.add_file('/remove', Buffer.from('temporary'), false);
    box.set_variable('VALUE', 'first');
    box.commit();
    return box.to_bytes();
  });
  assert.throws(() => WasmLockbox.open_with_password(bytes, 'wrong password'));
  bytes = withBox(WasmLockbox.open_with_password(bytes, password), box => {
    assert.deepEqual(Buffer.from(box.get_file('/keep')), Buffer.from('original'));
    box.commit();
    box.add_file('/keep', Buffer.from('replacement'), true);
    box.delete('/remove');
    box.set_variable('VALUE', 'second');
    box.commit();
    return box.to_bytes();
  });
  withBox(WasmLockbox.open_with_password(bytes, password), box => {
    assert.deepEqual(Buffer.from(box.get_file('/keep')), Buffer.from('replacement'));
    assert.equal(box.exists('/remove'), false);
    assert.equal(box.get_variable('VALUE'), 'second');
  });
});
