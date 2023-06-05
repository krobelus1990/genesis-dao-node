This document will give you a brief walkthrough on the core functionality of the Genesis DAO MVP.

# Setup

## Create a DAO
Go ahead and connect your wallet and hit create a New DAO.

<p align="center">
  <img width="1283" alt="Screenshot 2023-06-05 at 10 15 11" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/56015255-7c77-473d-84f2-f3abf24ee41c">
</p>

Think a second or two about the DAO ID - this will as well be the symbol of your token.

<p align="center">
  <img width="402" alt="Screenshot 2023-06-05 at 10 33 48" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/ecd29ad3-e132-40a0-8def-8147e4a8f6fa">
</p>  

Give the world some explanation about your. This information is later editable by the council.

<p align="center">
  <img width="545" alt="Screenshot 2023-06-05 at 10 17 04" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/2e9f5e1d-948b-4a2f-b7de-d8f8f07ad98a">
</p>  

## Configure proposal lifecycle
The next step is to configure your DAO for majority vote. Genesis DAO is currently working on an extension ecosystem that'll give rise to a lot more options.

For now you can

- issue a specific number of tokens
- decide how many of that tokens are required to stake to create a proposal; for spam protection reasons staking with the native currency is required as well
- a minimum majority threshold to require some participation in proposals for being accepted
- how long a proposal should be active for voting, this is internally stored as number in blocks but conveniently you can enter days here

<p align="center">
  <img width="666" alt="Screenshot 2023-06-05 at 10 18 42" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/79b87cad-abad-4f92-b1d3-c09ae14e4c63">
</p>
  
## Council Management

The DAO will be released to a council that controls it's actions with a multisignature account. We use the [multisig module](https://marketplace.substrate.io/pallets/pallet-multisig/) for this.

> The multisig setup currently works via polkadot.js.org (see below), so you should have at least two signers.

You can as well select initial recipients of the DAO tokens. The remainders will be sent to the council.

<p align="center">
  <img width="411" alt="Screenshot 2023-06-05 at 10 39 49" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/c6433dbd-e045-4695-b50c-655a4b0e8013">
</p>  

Welcome to your DAO!

<p align="center">
  <img width="871" alt="Screenshot 2023-06-05 at 10 19 55" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/99f438ae-1f4b-449c-b016-49a5b51fd492">
</p>

# Proposal Lifecycle
## Create a proposal

As a DAO user you can create a proposal if you have at least the number of tokens configured above and ten native tokens. Click `proposals` in your DAO overview page and go ahead and create one!

<p align="center">
  <img width="550" alt="Screenshot 2023-06-05 at 10 20 38" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/134d740f-d48e-43f6-9a46-3da5e5fff75c">
</p>

You have a chance to review it before it goes to vote:

This document will give you a brief walkthrough on the core functionality of the Genesis DAO MVP.

# Setup

## Create a DAO
Go ahead and connect your wallet and hit create a New DAO.

<p align="center">
  <img width="1283" alt="Screenshot 2023-06-05 at 10 15 11" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/56015255-7c77-473d-84f2-f3abf24ee41c">
</p>

Think a second or two about the DAO ID - this will as well be the symbol of your token.

<p align="center">
  <img width="402" alt="Screenshot 2023-06-05 at 10 33 48" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/ecd29ad3-e132-40a0-8def-8147e4a8f6fa">
</p>  

Give the world some explanation about your. This information is later editable by the council.

<p align="center">
  <img width="545" alt="Screenshot 2023-06-05 at 10 17 04" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/2e9f5e1d-948b-4a2f-b7de-d8f8f07ad98a">
</p>  

## Configure proposal lifecycle
The next step is to configure your DAO for majority vote. Genesis DAO is currently working on an extension ecosystem that'll give rise to a lot more options.

For now you can

- issue a specific number of tokens
- decide how many of that tokens are required to stake to create a proposal; for spam protection reasons staking with the native currency is required as well
- a minimum majority threshold to require some participation in proposals for being accepted
- how long a proposal should be active for voting, this is internally stored as number in blocks but conveniently you can enter days here

<p align="center">
  <img width="666" alt="Screenshot 2023-06-05 at 10 18 42" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/79b87cad-abad-4f92-b1d3-c09ae14e4c63">
</p>
  
## Council Management

The DAO will be released to a council that controls it's actions with a multisignature account. We use the [multisig module](https://marketplace.substrate.io/pallets/pallet-multisig/) for this.

> The multisig setup currently works via polkadot.js.org (see below), so you should have at least two signers.

You can as well select initial recipients of the DAO tokens. The remainders will be sent to the council.

<p align="center">
  <img width="411" alt="Screenshot 2023-06-05 at 10 39 49" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/c6433dbd-e045-4695-b50c-655a4b0e8013">
</p>  

Welcome to your DAO!

<p align="center">
  <img width="871" alt="Screenshot 2023-06-05 at 10 19 55" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/99f438ae-1f4b-449c-b016-49a5b51fd492">
</p>

# Proposal Lifecycle
## Create a proposal

As a DAO user you can create a proposal if you have at least the number of tokens configured above and ten native tokens. Click `proposals` in your DAO overview page and go ahead and create one!

<p align="center">
  <img width="550" alt="Screenshot 2023-06-05 at 10 20 38" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/134d740f-d48e-43f6-9a46-3da5e5fff75c">
</p>

You have the chance to review it before it goes to vote.

<p align="center">
  <img width="549" alt="Screenshot 2023-06-05 at 10 20 45" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/ee2841df-811c-43a4-bd55-bd6fec7c81ff">
</p>  

Your DAO is now up for voting.

<p align="center">
  <img width="841" alt="Screenshot 2023-06-05 at 10 21 15" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/b11f2cb0-4f73-4417-8f45-515e22b8dbde">
</p>

## Voting for a proposal as a community member

As a community member you can review the current count or cast your vote.

<p align="center">
  <img width="839" alt="Screenshot 2023-06-05 at 10 21 33" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/a6038b00-e2e6-4b67-95da-58e395d425f0">
</p>

## Marking a proposal as faulty

If you think a proposal is spam you can report it as faulty to the council.

<p align="center">
  <img width="901" alt="Screenshot 2023-06-05 at 10 23 08" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/a66f561e-6e4f-49f6-a652-849ce828673b">
</p>

This will mark this proposal as reported to the council.

<p align="center">
  <img width="838" alt="Screenshot 2023-06-05 at 10 23 40" src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/628fd105-4440-461e-8bd2-5bffc5f2c939">
</p>

> Upcoming version of Genesis DAO will create a non-signed transaction for the Council, but currently this functionality lies within polkadot.js.org





