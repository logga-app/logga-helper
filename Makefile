# tests need to run sequentially because of Keychain
test:
	cargo test -- --test-threads 1 --nocapture