package revault

import (
	"bytes"
	"path/filepath"
	"testing"
)

func TestNativeFileLifecycle(t *testing.T) {
	t.Setenv("LOCKBOX_LOCK_TIMEOUT_MS", "60")
	path := filepath.Join(t.TempDir(), "archive.lbox")
	key := bytes.Repeat([]byte{'K'}, 32)
	signer, err := GenerateProfileSigningKeyPair()
	if err != nil {
		t.Fatal(err)
	}
	defer signer.Close()
	writer, err := CreateFile(path, key, signer, false)
	if err != nil {
		t.Fatal(err)
	}
	payload := []byte{0, 255, 128, 10}
	if err := writer.AddFile("/hello", payload, false); err != nil {
		t.Fatal(err)
	}
	if err := writer.Commit(); err != nil {
		t.Fatal(err)
	}
	writer.Close()
	reader, err := OpenFile(path, key, nil)
	if err != nil {
		t.Fatal(err)
	}
	second, err := OpenFile(path, key, nil)
	if err != nil {
		t.Fatal(err)
	}
	content, err := second.GetFile("/hello")
	if err != nil || !bytes.Equal(content, payload) {
		t.Fatalf("read: %x %v", content, err)
	}
	second.Close()
	if unexpected, err := OpenFile(path, key, signer); err == nil {
		unexpected.Close()
		t.Fatal("writer ignored reader lock")
	}
	if unexpected, err := CreateFile(path, key, signer, true); err == nil {
		unexpected.Close()
		t.Fatal("replacement ignored reader lock")
	}
	reader.Close()
	writer, err = OpenFile(path, key, signer)
	if err != nil {
		t.Fatal(err)
	}
	if err := writer.AddFile("/hello", []byte("updated"), true); err != nil {
		t.Fatal(err)
	}
	if err := writer.Commit(); err != nil {
		t.Fatal(err)
	}
	writer.Close()
	reader, err = OpenFile(path, key, nil)
	if err != nil {
		t.Fatal(err)
	}
	defer reader.Close()
	content, err = reader.GetFile("/hello")
	if err != nil || string(content) != "updated" {
		t.Fatalf("persisted: %x %v", content, err)
	}
}
