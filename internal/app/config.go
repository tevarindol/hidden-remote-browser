package app

import (
	"github.com/caarlos0/env/v11"
	"github.com/joho/godotenv"
)

type Config struct {
	TelegramToken string `env:"TELEGRAM_TOKEN,required"`
	ChatID        int64  `env:"TG_CHAT_ID,required"`
	ChromeURL     string `env:"CHROME_URL" envDefault:"http://localhost:9222"`
	SendPhoto     bool   `env:"SEND_PHOTO" envDefault:"false"`
	HTMLSelector  string `env:"HTML_SELECTOR" envDefault:"main"`
}

func Load() (Config, error) {
	_ = godotenv.Load()

	return env.ParseAs[Config]()
}
