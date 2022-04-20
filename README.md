## Contracts

1. [bAsset vault](#basset-vault)
2. [bAsset vault strategy](#basset-vault-strategy)
3. [psi distributor](#psi-distributor)
4. [nAsset token](#nasset-token)
5. [nAsset token config holder](#nasset-token-config-holder)
6. [nAsset token rewards](#nasset-token-rewards)

## Halborn audit report

[link](https://github.com/HalbornSecurity/PublicReports/blob/master/CosmWasm%20Smart%20Contract%20Audits/Nexus_Protocol_CosmWasm_Smart_Contract_Security_Audit_Report_Halborn%20v1.1.pdf)

## bAsset vault

### v1

`basset_vault` is a main contract. The instantiation of this contract implies the instantiation of all child contracts in right order and setting up their interaction:

1. Instantiate `nasset_token_config_holder`.
2. Instantiate `nasset_token`.
3. Instantiate `nasset_token_rewards`.
4. Instantiate `psi_destributor`.

P.S. Anyone can set `nasset_token_rewards` in `nasset_token_config_holder` but only once.

`basset_vault` is able to hadle a list of messages:
- `HonestWork {}`:
    - claim ANC from anchor contract;
    - swap ANC to stablecoins on TerraSwap;
    - few options are possible here: 
        - If stablecoins value < stablecoins value before selling ANC : do nothing;
        - buy psi tokens and destribute rewards
        - deposite to anchor to be able to repay loans later
- `Rebalance {}`:
    Here `basset_vault_strategy` comes up on the stage. Strategy decides what to do to achieve the main `basset_vault` aims.

    Strategy which borrows UST and lend it to Anchor Earn for 20% yeild.
    Anchor pays to borrowers, so contract also sell ANC tokens in favor of bAsset depositors and \$PSI token stakers.

    Contract ask `basset_vault_strategy` on `Rebalance` how much to borrow or repay.

    Query:
    ```rust
    BorrowerAction {
        borrowed_amount: Uint256,
        locked_basset_amount: Uint256,
    }
    ```

    Response:
    ```rust
    pub enum BorrowerActionResponse {
        Nothing,
        Borrow {
            amount: Uint256,
            advised_buffer_size: Uint256,
        },
        Repay {
            amount: Uint256,
            advised_buffer_size: Uint256,
        },
    }
    ```
- `ClaimRemainder {}`:
    The chance that it will happen is slim to none.
    `basset_vault` locks 101% of loans.   
    Query borrowed info from anchor smart contract and check loan_amount. If it's zero (all users have withdrown all bAsset deposits):  
    - if aUST is not zero `basset_vault` redeems UST;
    - `basset_vault` uses extra 1% of UST to byu psi_tokens and sent them to governance stakers (there are no any nasset holders at thit moment).
There is `claim_rewards_delay` parrameter to avoid blockchain spam.

[Rebalance strategy](#basset-vault-strategy)

#### Repayment logic

If contract needs to repay part of loan (in case of bAsset withdraw or bAsset price drops) it should sell 
some of aUST(withdraw from Anchor Earn). But it is possible that this action will fail - there are too many borrowers
and Anchor prevent withdrawing UST that is borrowed. So, contract will not be able to repay loan in that case...
and will be liquidated.

To avoid that we do not deposit all borrowed UST to Anchor (convering to aUST). Instead portion of UST just stays on the balance
and is used when redeeming aUST returns error.
In case of error we repaying loan from UST on balance, and then redeem aUST for exactly the same amount. Repeat that cycle until
balance achieved.

Take a look at [repay logic](./contracts/basset_vault/src/commands.rs#L396).

And on (submsg id: `RedeemStableOnRepayLoan` & `RepayLoan` & `Borrowing` & `RedeemStableOnRemainder`) [reply handler](./contracts/basset_vault/src/contract.rs#L82).

#### Borrow logic

Nothing bad will happen if we fail to borrow more, so no error handling here.
Worth to mention that in case of borrowing error on deposit - we return error to user. Do not want to add logic for claiming rewards for holding bAsset(staking rewards).

### v2

Borrow UST from Anchor and use it in a sophisticated Mirror strategies.
Will be available in the future.

## bAsset vault strategy

### v1

The simplest strategy is to maintain bounds:
* borrow to `aim_ltv` if `ltv` < `minimum_ltv` (75%)
* repay to `aim_ltv` if `ltv` > `minimum_ltv` (85%)
* do nothing otherwise

`aim_ltv`: 80%


Halving LTV if price in bAsset oracle is obsolete.

### v2

Frontrun oracle price and maintain LTV at maximum(`basset_max_ltv` - 0.1%).

## PSi distributor

This contract receive PSi bought by `basset_vault` and distribute it between:
1. `nAsset` token holders
2. governance stakers
3. community pool

## nAsset token

CW20 compatible contract where that CW20_base contracts methods are synhcronized with nAsset_token_rewards to reward nAsset token **holders** (no need to stake).
`nAsset` token represent share of `bAsset` tokens locked in `basset_vault` and used as collateral on Anchor Earn.

## nAsset token config holder

Helper contract. CW20 contract have no ability to reward token holders, so some workaround needed. (same as in `bLuna`)

## nAsset token rewards

Helper contract. CW20 contract have no ability to reward token holders, so some workaround needed. (same as in `bLuna`)

# Release build

To build smart contracts for production, use `build_release.sh` script.

Before running it, make sure you installed tools for WASM:
 1. [WABT](https://github.com/WebAssembly/wabt) (`brew install wabt`)
 2. [Binaryen](https://github.com/WebAssembly/binaryen) (`brew install binaryen`)

# Integration tests

Some contracts have integrations tests feature flag. To build them for integration tests properly, use `integration_tests_build.sh`. This script builds those contracts and puts them to special directory in `contracts_scripts`. 

To configure this you have to set the path to your contracts scripts directory to the `CONTRACTS_SCRIPTS_PATH` env variable. Make sure you installed [WABT](https://github.com/WebAssembly/wabt) (`brew install wabt`) for striping wasm binaries.
# Getting started

## Setup Depedencies

Set up cosmwasm dependencies:
https://docs.cosmwasm.com/docs/0.14/getting-started/installation/

Install docker:
https://www.docker.com/products/docker-desktop/

Install rust nightly
```
rustup install nightly
rustup component add rust-src --toolchain nightly-x86_64-apple-darwin
```

```
# optional, these two may not be needed, try running the build_release.sh and integration_tests_build.sh to see if they already works for you
# if the above does, skip the below installations
brew install wabt
npm i wasm-opt -g
```

## Running integration tests on these contracts

Clone down the integration tests repo: https://github.com/Nexus-Protocol/contracts_scripts

Open your .bash_profile or .zshrc, add the below

```
export CONTRACTS_SCRIPTS_PATH=<path-to-contract-scripts>
```

Start local terra on a (new window)[https://github.com/Nexus-Protocol/contracts_scripts#start-localterra]

Then to run the integration tests

```
cd contracts_scripts

```

# Troubleshooting

- permission denied when running .sh files
https://askubuntu.com/questions/409025/permission-denied-when-running-sh-scripts

