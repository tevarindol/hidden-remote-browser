package chrome

import (
	"context"
	"fmt"
	"time"

	"github.com/chromedp/cdproto/page"
	"github.com/chromedp/cdproto/target"
	"github.com/chromedp/chromedp"
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
	ctx   context.Context
	url   string
	close context.CancelFunc
}

func New(ctx context.Context, url string) *Client {
	ctx, cancel := chromedp.NewRemoteAllocator(ctx, url)
	return &Client{ctx: ctx, url: url, close: cancel}
}

func (c *Client) Close() { c.close() }

func (c *Client) Warmup(ctx context.Context) error {
	wctx, cancel := context.WithTimeout(c.ctx, captureTimeout)
	defer cancel()
	if _, err := chromedp.Targets(wctx); err != nil {
		return fmt.Errorf(
			"connect to chrome at %s: %w (start chrome with --remote-debugging-port=9222; since chrome 136 a dedicated --user-data-dir is required)",
			c.url, err)
	}
	return nil
}

func (c *Client) CaptureActiveTab(withPDF bool, selector string) (Capture, error) {
	t, err := c.focusedTarget()
	if err != nil {
		return Capture{}, err
	}

	tctx, cancel := context.WithTimeout(c.ctx, captureTimeout)
	defer cancel()
	tctx, cancel = chromedp.NewContext(tctx, chromedp.WithTargetID(t.TargetID))
	defer cancel()

	var cap Capture
	actions := []chromedp.Action{
		chromedp.Evaluate(`location.href`, &cap.URL),
		chromedp.Title(&cap.Title),
		chromedp.Evaluate(fmt.Sprintf(`(function (sel) {
			var el = document.querySelector(sel);
			return el ? el.outerHTML : '';
		})(%q)`, selector), &cap.HTML),
	}
	if withPDF {
		actions = append(actions, chromedp.ActionFunc(func(ctx context.Context) error {
			data, _, err := page.PrintToPDF().WithPrintBackground(true).Do(ctx)
			if err != nil {
				return fmt.Errorf("printToPDF: %w", err)
			}
			cap.PDF = data
			return nil
		}))
	}

	if err := chromedp.Run(tctx, actions...); err != nil {
		if cap.HTML == "" {
			return Capture{}, fmt.Errorf("capture %s: %w", clip(t.URL, 60), err)
		}
		cap.PDF = nil
		return cap, fmt.Errorf("render pdf: %w", err)
	}
	if cap.HTML == "" {
		return Capture{}, fmt.Errorf("no element matches selector %q on %s", selector, clip(t.URL, 60))
	}
	return cap, nil
}

func (c *Client) focusedTarget() (*target.Info, error) {
	targets, err := chromedp.Targets(c.ctx)
	if err != nil {
		return nil, fmt.Errorf("list chrome targets: %w", err)
	}

	var pages, visible []*target.Info
	var focused *target.Info
	var names []string
	for _, t := range targets {
		if t.Type != "page" {
			continue
		}
		pages = append(pages, t)

		tctx, tcancel := chromedp.NewContext(c.ctx, chromedp.WithTargetID(t.TargetID))
		var st struct {
			Focused bool `json:"focused"`
			Visible bool `json:"visible"`
			Top     bool `json:"top"`
		}
		sctx, scancel := context.WithTimeout(tctx, tabStateTime)
		err := chromedp.Run(sctx, chromedp.Evaluate(`(() => ({
			focused: document.hasFocus(),
			visible: document.visibilityState === 'visible',
			top: window.top === window
		}))()`, &st))
		scancel()
		tcancel()
		if err != nil {
			continue
		}
		if st.Focused && st.Visible && st.Top {
			focused = t
			break
		}
		if st.Visible && st.Top {
			visible = append(visible, t)
			names = append(names, clip(t.URL, 60))
		}
	}

	switch {
	case focused != nil:
		return focused, nil
	case len(pages) == 1:
		return pages[0], nil
	case len(visible) == 1:
		return visible[0], nil
	case len(visible) == 0:
		return nil, fmt.Errorf("no visible chrome tab (open tabs: %d)", len(pages))
	default:
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
