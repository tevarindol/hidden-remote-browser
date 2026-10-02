package app

import (
	"context"
	"log/slog"

	"github.com/tevarindol/hidden-remote-browser/internal/chrome"
	"github.com/tevarindol/hidden-remote-browser/internal/telegram"
)

type App struct {
	log    *slog.Logger
	cfg    Config
	client *chrome.Client
	tg     *telegram.Sender
}

func New(log *slog.Logger) (*App, error) {
	cfg, err := Load()
	if err != nil {
		return nil, err
	}
	return &App{log: log, cfg: cfg}, nil
}

func (a *App) Start(ctx context.Context) error {
	a.client = chrome.New(ctx, a.cfg.ChromeURL)
	if err := a.client.Warmup(ctx); err != nil {
		return err
	}

	tg, err := telegram.New(a.cfg.TelegramToken)
	if err != nil {
		return err
	}
	tg.ChatID = a.cfg.ChatID
	a.tg = tg

	if a.cfg.KDEDeviceID != "" {
		a.log.Info("kde share-text enabled", "device_id", a.cfg.KDEDeviceID)
	}

	a.capture(ctx)
	return nil
}
