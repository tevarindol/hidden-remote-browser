package app

import (
	"context"

	"github.com/tevarindol/hidden-remote-browser/internal/telegram"
)

func (a *App) capture(ctx context.Context) {
	a.log.Info("capture triggered")

	cap, err := a.client.CaptureActiveTab(a.cfg.SendPhoto, a.cfg.HTMLSelector)
	if err != nil {
		a.log.Error("chrome capture failed", "err", err)
		if cap.HTML == "" {
			return
		}
	}

	if err := a.tg.SendPage(ctx, telegram.Page{
		Title: cap.Title,
		URL:   cap.URL,
		HTML:  cap.HTML,
		PNG:   cap.PNG,
	}); err != nil {
		a.log.Error("telegram send failed", "err", err)
	}
}
