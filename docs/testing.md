# Testing

Genesis DAO features three ways of ensuring quality assurance via testing:

- Unit tests on a per-pallet basis
- Integration tests using an external substrate client to run against a docker container
- Manual testing on the UI/Frontend

## Unit tests

The unit tests are packaged per pallet.

While the suite is still under development all features submitted as part of
a Milestone Proposal are fully tested.

From the root directory of the node, you can test all custom pallets of ours
by running these lines:

```bash
cd pallets/dao-core && cargo test && cd ../..
cd pallets/dao-assets && cargo test && cd ../..
cd pallets/dao-votes && cargo test && cd ../..
```


## Integration Tests

The `integration-wrapper` directory contains an integration test suite that
is utilizing the `subxt` substrate client. The test cases are located in the
`tests` folder - for the time being we have full integration tests for the
`dao-core` module.

The integration tests are running against our docker container.

Steps to run it:

- Run `docker compose up` to start a docker container with the node
- Wait until all dependencies are installed and the node starts producing blocks
- Enter the `integration-wrapper` directory and run `cargo test`


## Manual Testing
The implementation of the wireframes and HiFi Designs are part of the next milestone.

We, however, have already build a frontend application that communicates with the node
and supports all steps required.

That gives you two options for the manual testing:

### Polkadot.js.org interface

Compile & run the node by either
 - using `cargo release --dev --features local-node && ./target/release/genesis-dao --dev`
 - or running `docker compose up`

Afterwards navigate to to the [respective port config](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer)
on Polkadot.js.org and test the intrinsics.


### Built in interface

There are again, different ways of running this - with docker or without.

Dockerized:

1. Within the [node](https://github.com/deep-ink-ventures/genesis-dao-node) directory run `docker compose up` and wait for the node to produce blocks
2. Add the node to a local network using `docker network create genesis && docker network connect genesis genesis-dao`
3. Run `docker network inspect genesis | grep Gateway` and add the IP address to the `.env` file of the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository
4. Run `docker compose up` in the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository and visit the interface at `localhost:3000`

Running the frontend locally:

1. Start the node by either compiling or with docker.
2. Run `yarn && yarn dev` in the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository and visit the interface at `localhost:3000`
