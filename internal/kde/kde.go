package kde

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os/exec"
	"strings"
	"time"
)

const execTimeout = 10 * time.Second

type Sender struct {
	deviceID string
}

func New(deviceID string) *Sender {
	return &Sender{deviceID: deviceID}
}

func (s *Sender) ShareText(ctx context.Context, text string) error {
	ctx, cancel := context.WithTimeout(ctx, execTimeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, "kdeconnect-cli", "--share-text", text, "-d", s.deviceID)
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &out
	if err := cmd.Run(); err != nil {
		if errors.Is(err, exec.ErrNotFound) {
			return errors.New("kdeconnect-cli not found on PATH")
		}
		if ctx.Err() != nil {
			return fmt.Errorf("kdeconnect-cli timed out after %s: %w", execTimeout, ctx.Err())
		}
		return fmt.Errorf("kdeconnect-cli: %w: %s", err, strings.TrimSpace(out.String()))
	}
	return nil
}
