FROM paritytech/ci-linux:production

WORKDIR /var/www/genesis-dao

COPY . .

RUN rustup install nightly-2023-03-13-x86_64-unknown-linux-gnu
RUN rustup default nightly-2023-03-13-x86_64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown

RUN cargo build --release --features local-node

EXPOSE 9944
CMD [ "./target/release/genesis-dao", "--dev", "--ws-external"]
