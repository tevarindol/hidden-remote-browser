package hotkey

import (
	"testing"

	"golang.design/x/hotkey"
)

func TestParse(t *testing.T) {
	if key, err := Parse("120"); err != nil || key != hotkey.Key(0x78) {
		t.Fatalf("120 = %v, %v", key, err)
	}
	if _, err := Parse("0"); err == nil {
		t.Fatal("0: expected error")
	}
	if _, err := Parse("256"); err == nil {
		t.Fatal("256: expected error")
	}
	if _, err := Parse("F9"); err == nil {
		t.Fatal("F9: expected error")
	}
	if _, err := Parse(""); err == nil {
		t.Fatal("empty: expected error")
	}
}
