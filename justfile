build:
    go build ./...

build-cli:
    go build -o tanca-cli ./cmd/tanca-cli/

install-cli:
    go install ./cmd/tanca-cli/
