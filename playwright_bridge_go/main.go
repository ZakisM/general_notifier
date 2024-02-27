package main

import (
	"fmt"
	"log"
	"log/slog"
	"net/http"
	"strconv"

	"main/playwright_helper"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

const (
	playwrightContextKey string = "playwright"
)

func contentHandler(c echo.Context) error {
	url := c.QueryParam("url")

	timeoutStr := c.QueryParam("timeout")
	timeout, err := strconv.Atoi(timeoutStr)
	if err != nil {
		return err
	}

	slog.Info(fmt.Sprintf("%s, %v", url, timeout))

	pwh := c.Get(playwrightContextKey).(*playwright_helper.PlaywrightHelper)

	content, err := pwh.GetPageContent(url, timeout)
	if err != nil {
		return err
	}

	return c.String(http.StatusOK, *content)
}

func main() {
	pwh, err := playwright_helper.New()
	defer pwh.Close()
	if err != nil {
		log.Fatalf("%v", err)
	}

	e := echo.New()
	e.GET("/", contentHandler)
	e.Use(func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			c.Set(playwrightContextKey, pwh)
			return next(c)
		}
	})
	e.Use(middleware.Logger())
	e.Logger.Fatal(e.Start("0.0.0.0:8050"))

}
