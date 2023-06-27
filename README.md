# Genesis DAO

Welcome to Genesis DAO node.

This is the central node of the Genesis DAO Node.

## Getting Started

### Sneak peak
We provided a full [walkthrough](https://github.com/deep-ink-ventures/genesis-dao-node/blob/main/docs/walktrough.md) our dApp to give you an overview.

### Docker setup
The fastest way to get the system up and running is with docker compose:

```shell
docker compose build
docker compose up
```

> This setups the entire system, including frontend and services. It may take a while.

Afterwards you have a [frontend](http://localhost:3000/), [api](http://localhost:8000/) and [local node](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2Flocalhost:9944/#/accounts) running.

Look in the console, once the *listener* service is switching from `catching up` to `processing`, you are good to go.

> If you want to run only the node, comment out everything but the `chain` entry.


### Building from source

You can run the node simply with these commands:

```shell
cargo check --release --features local-node
cargo build --release --features local-node

./target/release/genesis-dao --dev
```

Node that `local-node` is the default feature and can be ommitted.

> If you need help setting up rust, please refer to our [rust setup docs](https://github.com/deep-ink-ventures/genesis-dao-node/blob/main/docs/rust-setup.md)

## Testing

Please refer to the [in-depth guide](https://github.com/deep-ink-ventures/genesis-dao-node/blob/main/docs/testing.md) for running our test guides.

## Infrastructure

There are a few accompanying repositories that are in development:

### Frontend dApp
[The frontend repository](https://github.com/deep-ink-ventures/genesis-dao-frontend) of the frontend provides a user friendly interface to setup and manage DAOs.

[The service repository](https://github.com/deep-ink-ventures/genesis-dao-service) is a backend service to support the frontend
by orchestrating tasks between different services (such as substrate and ipfs) and is a friendly helper to structure events and data access.
