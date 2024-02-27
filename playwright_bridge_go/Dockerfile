FROM golang:latest as builder
WORKDIR /app
COPY go.mod go.sum .
RUN go mod download
RUN go run github.com/playwright-community/playwright-go/cmd/playwright@latest install webkit
COPY *.go .
COPY playwright_helper ./playwright_helper
RUN CGO_ENABLED=0 GOOS=linux go build -o /app/playwright-bridge-go

FROM mcr.microsoft.com/playwright:v1.41.1-jammy
COPY --from=builder /app/playwright-bridge-go ./app/
COPY --from=builder ../root/.cache/ms-playwright-go ../root/.cache/ms-playwright-go
ENTRYPOINT ["/app/playwright-bridge-go"]
