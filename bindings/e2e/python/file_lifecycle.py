"""Public file-facade conformance: native persistence, locks and replacement.

Run with PYTHONPATH=bindings/python REVAULT_LIBRARY=/path/to/librevault_api.so
python bindings/e2e/python/file_lifecycle.py. Filesystem permission changes and
subprocesses model OS conditions that cannot be created by archive operations.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from revault_api import Revault, Lockbox, RevaultError


class NativeFiles(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='revault-files-')
        self.path = Path(self.temp.name) / 'résumé.lbox'
        self.runtime = Revault.load()
        self.signer = self.runtime.generate_profile_signing_key_pair()
        self.key = b'K' * 32
        self.previous_timeout = os.environ.get('LOCKBOX_LOCK_TIMEOUT_MS')
        os.environ['LOCKBOX_LOCK_TIMEOUT_MS'] = '60'

    def tearDown(self):
        self.signer.close()
        self.temp.cleanup()
        if self.previous_timeout is None:
            os.environ.pop('LOCKBOX_LOCK_TIMEOUT_MS', None)
        else:
            os.environ['LOCKBOX_LOCK_TIMEOUT_MS'] = self.previous_timeout

    def create(self, **extra):
        return Lockbox.create(self.path, content_key=self.key, signing_key=self.signer, **extra)

    def open(self, **extra):
        return Lockbox.open(self.path, content_key=self.key, **extra)

    def test_persistence_and_replacement(self):
        payload = bytes(range(256)) * 7 + b'\0end'
        with self.create() as box:
            box.add_file('/hello', payload, False)
            box.commit()
            box.commit()  # no-change repeat
        with self.open() as box:
            self.assertEqual(box.get_file('/hello'), payload)
        with self.open(signing_key=self.signer) as box:
            box.add_file('/hello', b'replacement\0', True)
            box.add_file('/added', b'added', False)
            box.commit()
        with self.open() as box:
            self.assertEqual(box.get_file('/hello'), b'replacement\0')
            self.assertEqual(box.get_file('/added'), b'added')
        with self.open(signing_key=self.signer) as box:
            box.delete('/hello')
            box.commit()
        with self.open() as box:
            self.assertFalse(box.exists('/hello'))
            self.assertEqual(box.get_file('/added'), b'added')
        with self.assertRaises(RevaultError):
            self.create()
        with self.create(overwrite=True) as box:
            box.add_file('/new', b'new archive', False)
            box.commit()
        with self.open() as box:
            self.assertFalse(box.exists('/added'))
            self.assertEqual(box.get_file('/new'), b'new archive')
        self.assertEqual(sorted(p.name for p in self.path.parent.iterdir()), [self.path.name])

    def test_shared_readers_exclude_writers_and_cannot_upgrade(self):
        with self.create() as box:
            box.add_file('/hello', b'original', False)
            box.commit()
        original = self.path.read_bytes()
        with self.open() as reader, self.open() as other:
            self.assertEqual(other.get_file('/hello'), b'original')
            with self.assertRaises(RevaultError):
                reader.set_owner_signing_key(self.signer)
            with self.assertRaises(RevaultError):
                reader.add_file('/injected', b'bad', False)
            with self.assertRaises(RevaultError):
                self.open(signing_key=self.signer)
            with self.assertRaises(RevaultError):
                self.create(overwrite=True)
            # A separate process must also join the native locking protocol.
            code = "from revault_api import Lockbox; import sys; b=Lockbox.open(sys.argv[1],content_key=b'K'*32); assert b.get_file('/hello')==b'original'; b.close()"
            subprocess.run([sys.executable, '-c', code, str(self.path)], check=True, timeout=15)
        self.assertEqual(self.path.read_bytes(), original)
        with self.open(signing_key=self.signer):
            result = subprocess.run([sys.executable, '-c', code, str(self.path)], capture_output=True, timeout=15)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'lock', result.stderr.lower())
        with self.open() as box:
            self.assertEqual(box.get_file('/hello'), b'original')

    def test_permissions_and_no_sidecar(self):
        with self.create() as box:
            box.add_file('/hello', b'read only', False)
            box.commit()
        self.path.chmod(0o444)
        self.path.parent.chmod(0o555)
        try:
            with self.open() as box:
                self.assertEqual(box.get_file('/hello'), b'read only')
            self.assertEqual(list(self.path.parent.iterdir()), [self.path])
        finally:
            self.path.parent.chmod(0o700)
            self.path.chmod(0o600)

    def test_password_contact_and_options(self):
        contact = self.runtime.key_contact_generate()
        public = self.runtime.key_contact_public_from_bytes(contact.public())
        try:
            credentials = [({'password': b'correct horse battery'}, {'password': b'correct horse battery'}),
                           ({'contact': public}, {'contact': contact})]
            for create, opening in credentials:
                with Lockbox.create(self.path, signing_key=self.signer, overwrite=True, **create) as box:
                    box.add_file('/hello', b'credential roundtrip', False)
                    box.commit()
                with Lockbox.open(self.path, **opening) as box:
                    self.assertEqual(box.get_file('/hello'), b'credential roundtrip')
                with Lockbox.open(self.path, signing_key=self.signer, **opening) as box:
                    box.add_file('/hello', b'updated', True)
                    box.commit()
                with Lockbox.open(self.path, **opening) as box:
                    self.assertEqual(box.get_file('/hello'), b'updated')
        finally:
            public.close()
            contact.close()

    def test_missing_and_invalid_inputs_do_not_create(self):
        with self.assertRaises(RevaultError):
            self.open()
        self.assertFalse(self.path.exists())
        with self.assertRaises(ValueError):
            Lockbox.create(self.path, password=b'password', content_key=self.key)
        self.assertFalse(self.path.exists())


if __name__ == '__main__':
    unittest.main()
