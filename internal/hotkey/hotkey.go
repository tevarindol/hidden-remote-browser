package hotkey

import (
	"context"
	"fmt"
	"runtime"
	"strconv"

	"golang.design/x/hotkey"
)

func Parse(spec string) (hotkey.Key, error) {
	n, err := strconv.Atoi(spec)
	if err != nil || n <= 0 || n > 0xFF {
		return 0, fmt.Errorf("invalid hotkey %q: expected a VK code 1-255", spec)
	}
	return hotkey.Key(n), nil
}

func Start(ctx context.Context, spec string, fn func()) error {
	key, err := Parse(spec)
	if err != nil {
		return err
	}

	hk := hotkey.New(nil, key)
	if err := hk.Register(); err != nil {
		if runtime.GOOS == "darwin" {
			return fmt.Errorf("register hotkey %q: %w (macOS: grant accessibility input monitoring in system settings)", spec, err)
		}
		return fmt.Errorf("register hotkey %q: %w (possibly taken by another app)", spec, err)
	}

	go func() {
		defer hk.Unregister()
		for {
			select {
			case <-ctx.Done():
				return
			case <-hk.Keydown():
				fn()
			}
		}
	}()
	return nil
}
