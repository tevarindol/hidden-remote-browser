package app

import (
	"context"
	"log/slog"

	"github.com/tevarindol/hidden-remote-browser/internal/chrome"
	"github.com/tevarindol/hidden-remote-browser/internal/hotkey"
	"github.com/tevarindol/hidden-remote-browser/internal/telegram"
)

type App struct {
	log    *slog.Logger
	cfg    Config
	client *chrome.Client
	tg     *telegram.Sender
	chatID int64
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
	if err := tg.EnsureChat(ctx); err != nil {
		return err
	}
	if a.cfg.ChatID == 0 {
		a.log.Info("chat resolved; set TG_CHAT_ID to pin it", "chatID", tg.ChatID)
	}
	a.tg = tg
	a.chatID = tg.ChatID

	if err := hotkey.Start(ctx, a.cfg.Hotkey, func() { a.capture(ctx) }); err != nil {
		return err
	}

	a.log.Info("running",
		"chrome", a.cfg.ChromeURL,
		"chatID", a.chatID,
		"pdf", a.cfg.SendPDF,
		"hotkey", a.cfg.Hotkey,
		"selector", a.cfg.HTMLSelector)
	return nil
}

func (a *App) Close() {
	if a.client != nil {
		a.client.Close()
	}
}
