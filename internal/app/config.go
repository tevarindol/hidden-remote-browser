package app

import (
	"github.com/caarlos0/env/v11"
	"github.com/joho/godotenv"
)

type Config struct {
	TelegramToken string `env:"TELEGRAM_TOKEN,required"`
	ChatID        int64  `env:"TG_CHAT_ID"`
	ChromeURL     string `env:"CHROME_URL" envDefault:"http://localhost:9222"`
	Hotkey        string `env:"HOTKEY" envDefault:"120"`
	SendPDF       bool   `env:"SEND_PDF" envDefault:"false"`
}

func Load() (Config, error) {
	_ = godotenv.Load()

	return env.ParseAs[Config]()
}
