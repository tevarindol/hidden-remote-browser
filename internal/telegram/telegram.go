package telegram

import (
	"bytes"
	"context"
	"fmt"

	"github.com/go-telegram/bot"
	"github.com/go-telegram/bot/models"
)

type Sender struct {
	ChatID int64
	api    *bot.Bot
}

func New(token string) (*Sender, error) {
	api, err := bot.New(token)
	if err != nil {
		return nil, fmt.Errorf("create telegram bot: %w", err)
	}
	return &Sender{api: api}, nil
}

type Page struct {
	Title string
	URL   string
	HTML  string
	PNG   []byte
}

func (s *Sender) SendPage(ctx context.Context, p Page) error {
	if _, err := s.api.SendDocument(ctx, &bot.SendDocumentParams{
		ChatID:   s.ChatID,
		Document: &models.InputFileUpload{Filename: "Task.html", Data: bytes.NewReader([]byte(p.HTML))},
	}); err != nil {
		return fmt.Errorf("send html: %w", err)
	}

	if len(p.PNG) > 0 {
		if _, err := s.api.SendDocument(ctx, &bot.SendDocumentParams{
			ChatID:   s.ChatID,
			Document: &models.InputFileUpload{Filename: "Task.png", Data: bytes.NewReader(p.PNG)},
		}); err != nil {
			return fmt.Errorf("send png: %w", err)
		}
	}
	return nil
}
