FROM rust:1.27

RUN apt-get update && apt-get install -y \
    libdbus-1-dev \
    libssl-dev
ADD . /kubeception
WORKDIR /kubeception
RUN cargo build --release && cargo install