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


## Service
The user interface is available at [genesis-dao.org](https://www.genesis-dao.org/).
The backend service documentation is available [here](https://service.genesis-dao.org/redoc/).
Our testnet node is available [here](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fnode.genesis-dao.org#/explorer).

## Running the node from source

Compile & run the node by either
 - using `cargo release --dev --features local-node && ./target/release/genesis-dao --dev`
 - or running `docker compose up`

Afterwards navigate to to the [respective port config](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer)
on Polkadot.js.org and test the intrinsics.

## Running the interface from source

There are again, different ways of running this - with docker or without.

Dockerized:

1. Within the [node](https://github.com/deep-ink-ventures/genesis-dao-node) directory run `docker compose up` and wait for the node to produce blocks
2. Add the node to a local network using `docker network create genesis && docker network connect genesis genesis-dao`
3. Run `docker network inspect genesis | grep Gateway` and add the IP address to the `.env` file of the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository
4. Run `docker compose up` in the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository and visit the interface at `localhost:3000`

Running the frontend locally:

1. Start the node by either compiling or with docker.
2. Run `yarn && yarn dev` in the [frontend](https://github.com/deep-ink-ventures/genesis-dao-frontend) repository and visit the interface at `localhost:3000`

## Manual Voting
The voting process user interface is part of the third milestone. We therefore walk you through the entire process in polkadot.js.org whilst we are busy implementing it in milestone 3.

If you walk through the frontend you have a DAO setup with a token issued and it is released to a multisig council. We will create a fresh DAO within polkadot.js.org during this walkthrough to make sure everyone knows what we are doing.

### Create a DAO with an identifier and a name

<img width="1790" alt="Screenshot 2023-04-04 at 17 17 06" src="https://user-images.githubusercontent.com/120174523/229842186-5409e8b7-9071-4d9c-8287-6de5462b7ecb.png">

### Issue a token to it

<img width="1792" alt="Screenshot 2023-04-04 at 17 17 48" src="https://user-images.githubusercontent.com/120174523/229842330-8b676a0b-0c37-40de-9b49-60be59bac4d5.png">

### Set the governance mode to majority voting.

<img width="1792" alt="Screenshot 2023-04-04 at 17 19 03" src="https://user-images.githubusercontent.com/120174523/229842440-e523f901-5bf0-4297-b0ca-4b82b82f56bd.png">

For this, configure the proposal duration in blocks, the required tokens in terms of the DAOs own tokens for a new proposal (we require DOT on top, to prevent spam) and how many more ayes than nays there must be for proposal acceptance (thus proposal acceptance requires: ayes >= nays + token_supply / 1024 * minimum_majority_per_1024)

### Create a proposal

<img width="1792" alt="Screenshot 2023-04-04 at 17 20 20" src="https://user-images.githubusercontent.com/120174523/229843023-7b26d390-5718-4917-9d32-2d0945ef112d.png">

The metadata url and the corresponding hash are abstracted away in the dApp in the same way that we are doing it on the metadata for a DAO. It's a JSON file containing all the information about the proposal, such as description and a link to a forum discussion.

### Vote in favour of your new proposal
 
<img width="1792" alt="Screenshot 2023-04-04 at 17 20 55" src="https://user-images.githubusercontent.com/120174523/229843433-d3be9b0c-dfb2-4a1f-931b-cd070ecdb1c9.png">

Note that this creates a checkpoint in the account history of the dao assets pallet:

<img width="1792" alt="Screenshot 2023-04-04 at 17 25 32" src="https://user-images.githubusercontent.com/120174523/229843578-69f274c3-1a2d-46a8-b7ee-53cd84c1c050.png">

Optional: Mark the proposal as faulty if it contains spam.

<img width="1786" alt="Screenshot 2023-04-04 at 17 27 56" src="https://user-images.githubusercontent.com/120174523/229843774-0dc01510-1196-4381-a5bb-c7733beddb63.png">

### Finalize the proposal
Wait for the number of blocks configured above and finalize the proposal. Everyone can do this!

<img width="1791" alt="Screenshot 2023-04-04 at 17 27 35" src="https://user-images.githubusercontent.com/120174523/229844082-b83f3b49-bfb8-4051-b7f3-f8447d13a2ea.png">

Veryify that the proposal got accepted:

<img width="1792" alt="Screenshot 2023-04-04 at 17 36 38" src="https://user-images.githubusercontent.com/120174523/229844415-b38c4294-619f-4097-b96c-0517077eab1c.png">

