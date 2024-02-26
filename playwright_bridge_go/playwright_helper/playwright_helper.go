package playwright_helper

import (
	"fmt"
	"log"
	"sync"

	"github.com/playwright-community/playwright-go"
)

type PlaywrightHelper struct {
	pw      *playwright.Playwright
	mutex   sync.Mutex
	browser playwright.Browser
}

func New() (*PlaywrightHelper, error) {
	pw, err := playwright.Run()
	if err != nil {
		return nil, fmt.Errorf("could not start playwright: %v", err)
	}

	browser, err := pw.WebKit.Launch(playwright.BrowserTypeLaunchOptions{
		Headless: playwright.Bool(true),
	})
	if err != nil {
		return nil, fmt.Errorf("could not launch browser: %v", err)
	}

	res := PlaywrightHelper{
		pw:      pw,
		browser: browser,
	}

	return &res, nil
}

func (pwh *PlaywrightHelper) GetPageContent(url string, timeout int) (*string, error) {
	pwh.mutex.Lock()
	defer pwh.mutex.Unlock()

	page, err := (pwh.browser).NewPage()
	if err != nil {
		return nil, err
	}

	page.Route("**/*.{css,png,jpg,jpeg,mp4,mp3,ttf,ttf2,woff,woff2,webp,svg,xml}", func(route playwright.Route) {
		route.Abort()
	})

	if _, err = page.Goto(url, playwright.PageGotoOptions{
		Timeout: playwright.Float(float64(timeout * 1000)),
	}); err != nil {
		return nil, err
	}

	content, err := page.Content()
	if err != nil {
		log.Fatalf("could not read content: %v", err)
	}

	err = page.Close()
	if err != nil {
		log.Fatalf("failed to close page: %v", err)
	}

	return &content, nil
}

func (pwh *PlaywrightHelper) Close() {
	pwh.mutex.Lock()
	defer pwh.mutex.Unlock()

	if err := pwh.browser.Close(); err != nil {
		log.Fatalf("could not close browser: %v", err)
	}
	if err := pwh.pw.Stop(); err != nil {
		log.Fatalf("could not stop Playwright: %v", err)
	}
}
