FROM rust:alpine AS backend-builder

WORKDIR /opt/app

COPY . .

RUN cargo build --release

FROM node:22-alpine AS frontend-builder

RUN npm -g install pnpm

WORKDIR /opt/app

COPY . .

RUN pnpm install --frozen-lockfile

RUN pnpm build

FROM alpine:latest

RUN apk add docker-cli-compose

WORKDIR /opt/app

COPY --from=backend-builder /opt/app/target/release/luminary .
COPY --from=frontend-builder /opt/app/packages/panel/build ./static

CMD ["./luminary"]