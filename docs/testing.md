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

### Polkadot.js.org System Test Walkthrough

In order to test the system you have to have the node running. We assume for this that you have the node on `127.0.0.1:9944`, which means that you can review it on [the explorer](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer).

### Create a DAO
Within the Developer Settings -> Extrinsics, select the `DaoCore` extrinsic and create a DAO.

<img width="1792" alt="Screenshot 2023-02-20 at 08 58 31" src="https://user-images.githubusercontent.com/120174523/220048109-8333223b-5fc9-4086-bbec-7aa26aa36ef2.png">

The minimum and maximum length for the values is configured [here](https://github.com/deep-ink-ventures/genesis-dao-node/blob/main/runtime/local/src/lib.rs#L340-L342), the id may only use uppercase chars or numbers.

> DAO Creation reserves [a portion of your native balance](https://github.com/deep-ink-ventures/genesis-dao-node/blob/main/runtime/local/src/lib.rs#L345). This will be DOT, once this chain becomes a system parachain.

### Issue a token

Within the Developer Settings -> Extrinsics, select the `DaoCore` extrinsic and issue a token.

As the owner of the DAO you can now issue a token with an arbitraty supply. This can only be done once.

<img width="1792" alt="Screenshot 2023-02-20 at 08 59 25" src="https://user-images.githubusercontent.com/120174523/220048891-e1f616af-5fd3-48ba-966b-96973888a212.png">

### Set metadata
A DAO can store metadata such as a logo, description and (optional) contact data in a JSON format that is still [under specification](https://github.com/deep-ink-ventures/genesis-dao-node/issues/6). This JSON format needs to be uploaded either to a CDN or on IPFS. A hash value to validate the integrity of the JSON needs to be provided as well. We are building a [backend services](https://github.com/deep-ink-ventures/genesis-dao-service) to help users doing this without hassle, but everyone is free to just upload their own files and call this extrinsic afterwards.

<img width="1792" alt="Screenshot 2023-02-20 at 09 00 17" src="https://user-images.githubusercontent.com/120174523/220049836-12822e88-0b80-4479-a109-0b3a9a6000bb.png">

You can use this tool to create a valid sha3: https://emn178.github.io/online-tools/sha3_256.html

> We are not verifying the link integrity on the node, as this is rather complex to do in a decentralized environment. Our frontend and backend will provide tooling to verify integer DAOs.

### Validate DAO creation

You can read the chain state to verify that your DAO has been created correctly.

<img width="1787" alt="Screenshot 2023-02-20 at 09 00 44" src="https://user-images.githubusercontent.com/120174523/220050115-6a16255c-167e-4481-abab-6bf9c436295e.png">

### Destroying a DAO
A DAO can be easily destroyed and the reserved balance is freed. However, in order to do so, the token needs to be destroyed first.

In the extrinsics, visit `assets` and start destroying the token.

<img width="1790" alt="Screenshot 2023-02-20 at 09 01 16" src="https://user-images.githubusercontent.com/120174523/220050576-c3f0ce6d-df97-4887-b257-e5cf6a1d1ec0.png">

Next up delete the assets account storage from the asset pallets:

<img width="1787" alt="Screenshot 2023-02-20 at 09 01 38" src="https://user-images.githubusercontent.com/120174523/220050692-d2337540-9a8e-48d8-bf8f-b1326308928c.png">

> Due to upper bound limits in the block size, this step may be required multiple times if you have many accounts.

Next up delete the assets approval storage from the asset pallets (if you've created approvals):

<img width="1792" alt="Screenshot 2023-02-20 at 09 01 59" src="https://user-images.githubusercontent.com/120174523/220050995-88b36651-8dc8-4d70-8450-dcf062b21edc.png">

You can now finish the destroyments of the token:

<img width="1785" alt="Screenshot 2023-02-20 at 09 02 27" src="https://user-images.githubusercontent.com/120174523/220051066-566c5dec-2659-4133-ae12-a12fc990414e.png">

With the token destroyed, you can now destroy the DAO - your balance is returned:

<img width="1788" alt="Screenshot 2023-02-20 at 09 03 03" src="https://user-images.githubusercontent.com/120174523/220051174-fbea52e2-ce7a-47f1-b639-7d81e38588cc.png">
