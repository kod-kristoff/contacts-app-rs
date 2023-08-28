# quick-dev:
#     cargo watch -q -c -w examples/quick_dev.rs -x 'run --example quick_dev'

serve-dev:
    cargo watch -q -c -w src/ -x 'run '

test:
    cargo test --all
