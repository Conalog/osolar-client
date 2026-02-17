.PHONY: spec test test-ts test-python test-go test-rust live live-ts live-python live-go live-rust live-all live-all-ts live-all-python live-all-go live-all-rust

spec:
	./scripts/fetch-spec.sh

test: test-ts test-python test-go test-rust

test-ts:
	cd clients/ts && npm test

test-python:
	cd clients/python && . .venv/bin/activate && pytest

test-go:
	cd clients/go && go test ./...

test-rust:
	cd clients/rust && cargo test

live: live-ts live-python live-go live-rust

live-ts:
	cd clients/ts && npm run build && npm run example:live

live-python:
	cd clients/python && . .venv/bin/activate && python examples/live_smoke.py

live-go:
	cd clients/go && go run ./examples/live-smoke

live-rust:
	cd clients/rust && cargo run --example live_smoke

live-all: live-all-ts live-all-python live-all-go live-all-rust

live-all-ts:
	cd clients/ts && npm run build && npm run example:live-all

live-all-python:
	cd clients/python && . .venv/bin/activate && python examples/live_all.py

live-all-go:
	cd clients/go && go run ./examples/live-all

live-all-rust:
	cd clients/rust && cargo run --example live_all
