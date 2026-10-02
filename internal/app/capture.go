package app

import (
	"context"
	"time"

	"golang.design/x/clipboard"

	"github.com/tevarindol/hidden-remote-browser/internal/telegram"
)

func (a *App) capture(ctx context.Context) {
	a.log.Info("capture triggered", "vk", a.cfg.Hotkey)

	cap, err := a.client.CaptureActiveTab(a.cfg.SendPDF)
	if err != nil {
		a.log.Error("chrome capture failed", "err", err)
		if cap.HTML == "" {
			return
		}
	}

	if cap.Text != "" {
		wctx, wcancel := context.WithTimeout(ctx, 5*time.Second)
		defer wcancel()
		if _, err := clipboard.Write(wctx, clipboard.FmtText, []byte(cap.Text)); err != nil {
			a.log.Error("clipboard write failed", "err", err)
		} else {
			a.log.Info("text copied to clipboard")
		}
	}

	if err := a.tg.SendPage(ctx, telegram.Page{
		Title:   cap.Title,
		URL:     cap.URL,
		HTML:    cap.HTML,
		PDF:     cap.PDF,
		WithPDF: a.cfg.SendPDF,
	}); err != nil {
		a.log.Error("telegram send failed", "err", err)
	}
}
