FROM paritytech/ci-linux:production

WORKDIR /var/www/genesis-dao

COPY . .

RUN rustup default stable
RUN rustup update
RUN rustup update nightly
RUN  rustup target add wasm32-unknown-unknown --toolchain nightly

RUN cargo build --release --features local-node

EXPOSE 9944
CMD [ "./target/release/genesis-dao", "--dev", "--ws-external"]
