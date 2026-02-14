[parallel]
dev: _dev_web _dev_node

[working-directory('packages/node')]
_dev_node:
    cargo watch -x run

[working-directory('packages/panel')]
_dev_web:
    pnpm vite dev

install:
    cd packages/panel; pnpm install
    cargo install cargo-watch --locked
