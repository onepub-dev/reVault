/** Run: REVAULT_LIBRARY=/path/to/librevault_api.so node --test bindings/e2e/javascript/file_lifecycle.js */
import assert from 'node:assert/strict';
import { test } from 'node:test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { Revault, Lockbox, RevaultError } from '../../javascript/index.js';

const runtime = await Revault.load();
const key = Buffer.alloc(32, 75);
process.env.LOCKBOX_LOCK_TIMEOUT_MS = '60';

test('native file lifecycle, shared readers, writer exclusion and replacement', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'revault-js-'));
  const archive = path.join(root, 'résumé.lbox');
  const signer = runtime.generateProfileSigningKeyPair();
  const payload = Buffer.from([0, 255, 128, 10, 0, 64]);
  const create = extra => Lockbox.create(archive, {contentKey: key, signingKey: signer, ...extra});
  const open = extra => Lockbox.open(archive, {contentKey: key, ...extra});
  try {
    assert.throws(() => open(), RevaultError);
    assert.equal(fs.existsSync(archive), false);
    let box = create();
    try { box.addFile('/hello', payload, false); box.commit(); box.commit(); } finally { box.close(); }
    const reader = open();
    try {
      box = open();
      try { assert.deepEqual(box.getFile('/hello'), payload); } finally { box.close(); }
      assert.throws(() => reader.setOwnerSigningKey(signer), RevaultError);
      assert.throws(() => reader.addFile('/bad', payload, false), RevaultError);
      assert.throws(() => open({signingKey: signer}), RevaultError);
      assert.throws(() => create({overwrite: true}), RevaultError);
    } finally { reader.close(); }
    box = open({signingKey: signer});
    try {
      const module = new URL('../../javascript/index.js', import.meta.url).href;
      const child = spawnSync(process.execPath, ['--input-type=module', '-e', `import {Lockbox} from ${JSON.stringify(module)}; const box=Lockbox.open(process.argv[1],{contentKey:Buffer.alloc(32,75)}); box.close();`, archive], {env: process.env, timeout: 15000});
      assert.equal(child.error, undefined);
      assert.notEqual(child.status, 0);
      assert.match(child.stderr.toString(), /lock/i);
      box.addFile('/hello', Buffer.from('replacement'), true);
      box.addFile('/added', payload, false);
      box.commit();
    } finally { box.close(); }
    box = open();
    try { assert.equal(box.getFile('/hello').toString(), 'replacement'); assert.deepEqual(box.getFile('/added'), payload); } finally { box.close(); }
    box = open({signingKey: signer});
    try { box.delete('/hello'); box.commit(); } finally { box.close(); }
    box = open();
    try { assert.equal(box.exists('/hello'), false); assert.deepEqual(box.getFile('/added'), payload); } finally { box.close(); }
    assert.throws(() => create(), RevaultError);
    box = create({overwrite: true});
    try { box.addFile('/new', payload, false); box.commit(); } finally { box.close(); }
    box = open();
    try { assert.equal(box.exists('/added'), false); assert.deepEqual(box.getFile('/new'), payload); } finally { box.close(); }
    assert.deepEqual(fs.readdirSync(root), ['résumé.lbox']);
  } finally { signer.close(); fs.rmSync(root, {recursive: true, force: true}); }
});

test('password and contact files preserve native credentials and write access', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'revault-js-credentials-'));
  const archive = path.join(root, 'archive.lbox');
  const signer = runtime.generateProfileSigningKeyPair();
  const contact = runtime.keyContactGenerate();
  const publicKey = runtime.keyContactPublicFromBytes(contact.public());
  try {
    for (const [creating, opening] of [[{password: Buffer.from('test password')}, {password: Buffer.from('test password')}], [{contact: publicKey}, {contact}]]) {
      let box = Lockbox.create(archive, {...creating, signingKey: signer, overwrite: true});
      try { box.addFile('/hello', Buffer.from('original'), false); box.commit(); } finally { box.close(); }
      box = Lockbox.open(archive, opening);
      try { assert.equal(box.getFile('/hello').toString(), 'original'); } finally { box.close(); }
      box = Lockbox.open(archive, {...opening, signingKey: signer});
      try { box.addFile('/hello', Buffer.from('updated'), true); box.commit(); } finally { box.close(); }
      box = Lockbox.open(archive, opening);
      try { assert.equal(box.getFile('/hello').toString(), 'updated'); } finally { box.close(); }
    }
  } finally { publicKey.close(); contact.close(); signer.close(); fs.rmSync(root, {recursive: true, force: true}); }
});

test('read-only bind mount', {skip: !process.env.REVAULT_READONLY_ARCHIVE}, () => {
  const box = Lockbox.open(process.env.REVAULT_READONLY_ARCHIVE, {contentKey: key});
  try { assert.equal(box.getFile('/hello').toString(), 'read only'); } finally { box.close(); }
});
