import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import { Revault, Lockbox } from '../../javascript/index.js';

const [archive, entry, expectedFile] = process.argv.slice(2);
if (!archive || !entry || !expectedFile) {
  throw new Error('usage: node plaintext_file_conformance.js ARCHIVE ENTRY EXPECTED_FILE');
}

await Revault.load(process.env.REVAULT_LIBRARY);
const digest = () => createHash('sha256').update(fs.readFileSync(archive)).digest('hex');
const before = digest();
const box = Lockbox.openUnencrypted(archive);
try {
  assert.deepEqual(Buffer.from(box.getFile(entry)), fs.readFileSync(expectedFile));
  assert.throws(() => box.addFile('/must-not-write', Buffer.from('denied'), false));
  assert.throws(() => box.commit());
} finally {
  box.close();
}
assert.equal(digest(), before);
console.log('PASS\tjavascript\tplaintext read-only archive');
