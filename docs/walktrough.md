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

## Finalize
Once the proposal duration is over everyone can hit the finalize button to move change the proposals status (it is automatically prohibited to vote once the duration is over, but change states on the blockchain require an action on behalf of someone).


<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/b8590ffd-3289-4636-b7ae-a0cd319c30cc">
</p>

## Acting as a council

> Upcoming version of Genesis DAO will create a non-signed transaction for the Council, but currently this functionality lies within polkadot.js.org

Go ahead and click create multisig in [polkadot.js.og](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fnode.genesis-dao.org#/)

<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/441b3ea3-d2dd-40b4-8dcd-e88b4c2911fc" />
</p>  

Select the signers that you have used in the DAO creation. In order for this to match the Multisig created within your DAO the order and the threshold must be identical because they are used to derive the address.

<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/6caeaa06-02d1-46dc-a9b2-dee566b9dd95" />
</p>  

The multisig is now mirrored in polkadot.js.org:

<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/b7e8da19-d1e8-4196-a48e-ff915a9f197d" />
</p>  

You can now go ahead and interact with the DAO - most prominently fault or mark a proposal as implemented.

> After a proposal is over it's `finalized`, meaning people can no longer vote and it changes it status. `implemented` means that the requested action has been taken - e.g. someone went to the website and made the logo bigger. While everyone can mark as `finalized` ones a proposal is over, `implemented` is a council action.

Be sure to copy the encoded call data, as it's required for the multisig.

<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/ee170fce-ff19-485f-8ce9-5f0dfd2b87b3" />
</p>  

You can now authorize the transcation in your multisig wallet.

And approve it:

<p align="center">
  <img src="https://github.com/deep-ink-ventures/genesis-dao-node/assets/120174523/1fbebb3b-977d-4b08-bf8a-2bd5694b65c0" />
</p>  

We have created a video on all of this [here](https://drive.google.com/file/d/1nD1wxHhs0jP_zL49tCeidj46ZLFTgm4X/view).


