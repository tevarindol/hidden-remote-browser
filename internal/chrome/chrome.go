package chrome

import (
	"context"
	"fmt"
	"io"
	"time"

	"github.com/go-rod/rod"
	"github.com/go-rod/rod/lib/launcher"
	"github.com/go-rod/rod/lib/proto"
)

const (
	captureTimeout = 15 * time.Second
	tabStateTime   = 2 * time.Second
)

type Capture struct {
	URL   string
	Title string
	HTML  string
	PDF   []byte
}

type Client struct {
	ctx  context.Context
	url  string
	br   *rod.Browser
	spun bool
}

func New(ctx context.Context, url string) *Client {
	return &Client{ctx: ctx, url: url}
}

func (c *Client) Close() {
	if c.spun {
		c.br.Close()
	}
}

func (c *Client) Warmup(ctx context.Context) error {
	ws, err := launcher.ResolveURL(c.url)
	if err != nil {
		return fmt.Errorf("resolve cdp endpoint %s: %w", c.url, err)
	}
	c.br = rod.New().Context(c.ctx).NoDefaultDevice().ControlURL(ws)
	if err := c.br.Connect(); err != nil {
		return fmt.Errorf(
			"connect to chrome at %s: %w (start chrome with --remote-debugging-port=9222; since chrome 136 a dedicated --user-data-dir is required)",
			c.url, err)
	}
	c.spun = true
	if _, err := c.br.Timeout(captureTimeout).Pages(); err != nil {
		return fmt.Errorf("list chrome targets: %w", err)
	}
	return nil
}

func (c *Client) CaptureActiveTab(withPDF bool, selector string) (Capture, error) {
	p, err := c.focusedPage()
	if err != nil {
		return Capture{}, err
	}
	p = p.Timeout(captureTimeout)

	info, err := p.Info()
	if err != nil {
		return Capture{}, fmt.Errorf("read tab info: %w", err)
	}

	el, err := p.Element(selector)
	if err != nil {
		return Capture{}, fmt.Errorf("no element matches selector %q on %s: %w", selector, clip(info.URL, 60), err)
	}
	html, err := el.HTML()
	if err != nil {
		return Capture{}, fmt.Errorf("read element html: %w", err)
	}

	cap := Capture{URL: info.URL, Title: info.Title, HTML: html}
	if !withPDF {
		return cap, nil
	}

	sr, err := p.PDF(&proto.PagePrintToPDF{PrintBackground: true})
	if err != nil {
		return cap, fmt.Errorf("render pdf: %w", err)
	}
	defer sr.Close()
	pdf, err := io.ReadAll(sr)
	if err != nil {
		return cap, fmt.Errorf("read pdf: %w", err)
	}
	cap.PDF = pdf
	return cap, nil
}

func (c *Client) focusedPage() (*rod.Page, error) {
	pages, err := c.br.Timeout(captureTimeout).Pages()
	if err != nil {
		return nil, fmt.Errorf("list chrome targets: %w", err)
	}

	type cand struct {
		p   *rod.Page
		url string
	}
	var visible []cand
	for _, p := range pages {
		pctx := p.Timeout(tabStateTime)
		probe, err := pctx.Eval(`(() => ({
			focused: document.hasFocus(),
			visible: document.visibilityState === 'visible',
			top: window.top === window
		}))()`)
		if err != nil {
			continue
		}
		if probe.Value.Get("focused").Bool() && probe.Value.Get("visible").Bool() && probe.Value.Get("top").Bool() {
			return p, nil
		}
		if probe.Value.Get("visible").Bool() && probe.Value.Get("top").Bool() {
			if info, err := p.Info(); err == nil {
				visible = append(visible, cand{p: p, url: clip(info.URL, 60)})
			}
		}
	}

	switch {
	case len(pages) == 1:
		return pages[0], nil
	case len(visible) == 1:
		return visible[0].p, nil
	case len(visible) == 0:
		return nil, fmt.Errorf("no visible chrome tab (open tabs: %d)", len(pages))
	default:
		var names []string
		for _, v := range visible {
			names = append(names, v.url)
		}
		return nil, fmt.Errorf(
			"cannot determine active tab, %d visible candidates: %s (close extra windows)",
			len(visible), names)
	}
}

func clip(s string, n int) string {
	if len(s) > n {
		return s[:n] + "…"
	}
	return s
}
