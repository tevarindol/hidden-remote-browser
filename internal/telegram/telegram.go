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
	first  chan int64
}

func New(token string) (*Sender, error) {
	s := &Sender{first: make(chan int64, 1)}
	api, err := bot.New(token, bot.WithDefaultHandler(
		func(_ context.Context, _ *bot.Bot, upd *models.Update) {
			if upd.Message == nil {
				return
			}
			select {
			case s.first <- upd.Message.Chat.ID:
			default:
			}
		}))
	if err != nil {
		return nil, fmt.Errorf("create telegram bot: %w", err)
	}
	s.api = api
	return s, nil
}

func (s *Sender) EnsureChat(ctx context.Context) error {
	if s.ChatID != 0 {
		return nil
	}
	rctx, rcancel := context.WithCancel(ctx)
	defer rcancel()

	go s.api.Start(rctx)
	select {
	case id := <-s.first:
		s.ChatID = id
		return nil
	case <-ctx.Done():
		return fmt.Errorf("unknown chat: %w (send /start to the bot in telegram)", ctx.Err())
	}
}

type Page struct {
	Title   string
	URL     string
	HTML    string
	PDF     []byte
	WithPDF bool
}

func (s *Sender) SendPage(ctx context.Context, p Page) error {
	if _, err := s.api.SendDocument(ctx, &bot.SendDocumentParams{
		ChatID:   s.ChatID,
		Document: &models.InputFileUpload{Filename: "Task.html", Data: bytes.NewReader([]byte(p.HTML))},
	}); err != nil {
		return fmt.Errorf("send html: %w", err)
	}

	if p.WithPDF && len(p.PDF) > 0 {
		if _, err := s.api.SendDocument(ctx, &bot.SendDocumentParams{
			ChatID:   s.ChatID,
			Document: &models.InputFileUpload{Filename: "Task.pdf", Data: bytes.NewReader(p.PDF)},
		}); err != nil {
			return fmt.Errorf("send pdf: %w", err)
		}
	}
	return nil
}
