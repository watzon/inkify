FROM debian:buster-slim as tensorflow

WORKDIR /usr/src/build

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    wget \
    gnupg \
    python3 \
    python3-dev \
    python3-pip \
    python3-numpy \
    llvm \
    clang

RUN pip3 install wheel packaging requests opt_einsum
RUN pip3 install keras_preprocessing --no-deps

# Install bazel
RUN wget https://github.com/bazelbuild/bazelisk/releases/download/v1.18.0/bazelisk-linux-amd64
RUN chmod +x bazelisk-linux-amd64
RUN mv bazelisk-linux-amd64 /usr/local/bin/bazel

# Install tensorflow
RUN git clone https://github.com/tensorflow/tensorflow \
    && cd tensorflow \
    && git checkout v2.5.0
RUN cd tensorflow && ./configure
RUN cd tensorflow && bazel build --compilation_mode=opt --copt=-march=native --jobs=12 tensorflow:libtensorflow.so

FROM rust:1.73.0-buster as builder

WORKDIR /usr/src/app

# Copy tensorflow shared libraries from tensorflow image
COPY --from=tensorflow /usr/src/build/tensorflow/bazel-bin/tensorflow/libtensorflow.so* /usr/local/lib/
COPY --from=tensorflow /usr/src/build/tensorflow/bazel-bin/tensorflow/libtensorflow_framework.so* /usr/local/lib/

RUN ldconfig

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

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libharfbuzz-dev \
    libfontconfig1 \
    fontconfig

# Copy fonts
COPY --from=fonts /data/fonts/nerd_fonts/* /usr/share/fonts/truetype/
RUN fc-cache -fv

# Copy binary abd tensorflow model files
COPY --from=builder /usr/src/app/target/release/inkify /usr/src/app/
COPY --from=builder /usr/src/app/src/tensorflow /usr/src/app/tensorflow/

# Copy tensorflow shared libraries from tensorflow image
COPY --from=tensorflow /usr/src/build/tensorflow/bazel-bin/tensorflow/libtensorflow.so* /usr/local/lib/
COPY --from=tensorflow /usr/src/build/tensorflow/bazel-bin/tensorflow/libtensorflow_framework.so* /usr/local/lib/

RUN ldconfig

ARG PORT=8080
ARG HOST=0.0.0.0

ENV PORT=$PORT
ENV HOST=$HOST

EXPOSE $PORT

# Run
CMD ["/usr/src/app/inkify", "--tensorflow-model-dir", "/usr/src/app/tensorflow"]