[parallel]
dev: _dev_web _dev_node

_dev_node:
    cargo watch -w packages/node -x run

_dev_web:
    pnpm -r dev

install:
    cd packages/panel; pnpm install
    cargo install cargo-watch --locked
    cargo install sqlx-cli --locked

build:
    pnpm build
    cargo build --release
    cp -R packages/panel/build target/release/static

export DATABASE_URL := "sqlite://" + justfile_dir() + "/luminary.db"

[working-directory('packages/node')]
prepare:
    rm -R -f .sqlx
    cargo sqlx database setup
    cargo sqlx prepare 
