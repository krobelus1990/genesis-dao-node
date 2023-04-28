FROM paritytech/ci-linux:production

WORKDIR /var/www/genesis-dao

COPY . .

RUN cargo build --release --features local-node

EXPOSE 9944
CMD [ "./target/release/genesis-dao", "--dev", "--ws-external"]
