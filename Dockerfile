FROM rust:alpine AS backend-builder

COPY / /opt/app

WORKDIR /opt/app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/opt/app/target \
	cargo build --release && \
	cp /opt/app/target/release/luminary /opt/app/luminary

FROM node:22-alpine AS frontend-builder

RUN npm -g install pnpm

COPY / /opt/app

WORKDIR /opt/app

RUN pnpm install --frozen-lockfile

RUN pnpm build

FROM alpine:latest

RUN apk add docker-cli-compose

COPY --from=backend-builder /opt/app/luminary /opt/app/luminary
COPY --from=frontend-builder /opt/app/packages/panel/build /opt/app/static

WORKDIR /opt/app

ENTRYPOINT ["/opt/app/luminary"]