# DAO Votes Pallet

Vote on proposal to move your DAO forward.

## Overview
This module contains functionality to create and manage proposals for a DAO. It utilizes the checkpoint functionality
of the dao-assets pallet and implements the full lifecycle of proposal management.

## Interface

### Dispatchable Functions
- `create_proposal`: Create a proposal alongside with a hash for a structured JSON file.
- `fault_proposal`: DAO owner can mark a proposal as faulty.
- `finalize_proposal`: Run the counting of a finalized proposal. A high number of votes may require multiple calls.
- `vote`: Vote in favor or against a proposal with your current tokens.
- `set_governance_majority_vote`: Configure the default voting mechanism - majority vote.
