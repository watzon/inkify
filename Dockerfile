FROM rust:1.73.0-buster as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    libharfbuzz-dev

# Copy files
COPY . .

# Build
RUN cargo build --release

FROM debian:buster-slim as fonts

RUN apt-get update && apt-get install -y \
    wget \
    xz-utils

WORKDIR /data/fonts

COPY ./docker/download_nerd_fonts.sh .

RUN ls -la
RUN chmod +x download_nerd_fonts.sh
RUN bash ./download_nerd_fonts.sh

FROM debian:buster-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libharfbuzz-dev \
    libfontconfig1 \
    fontconfig

# Copy fonts
COPY --from=fonts /data/fonts/nerd_fonts/* /usr/share/fonts/truetype/
RUN fc-cache -fv

# Copy binary
COPY --from=builder /usr/src/app/target/release/inkify /usr/local/bin/inkify

ARG PORT=8080
ARG HOST=0.0.0.0

ENV PORT=$PORT
ENV HOST=$HOST

EXPOSE $PORT

# Run
ENTRYPOINT ["/usr/local/bin/inkify"]