## Contracts

1. [bAsset vault](#basset-vault)
2. [bAsset vault strategy](#basset-vault-strategy)
3. [psi distributor](#psi-distributor)
4. [nAsset token](#nasset-token)
5. [nAsset token config holder](#nasset-token-config-holder)
6. [nAsset token rewards](#nasset-token-rewards)

## bAsset vault

### v1

Strategy which borrows UST and lend it to Anchor Earn for 20% yeild.
Anchor pays to borrowers, so contract also sell ANC tokens in favor of bAsset depositors and \$PSI token stakers.

Contract ask `basset_vault_strategy` on `Rebalance` to know how much to borrow or repay.

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

## nAsset token

CW20 compatible contract that rewards token **holders** (no need to stake).
`nAsset` token represent share of `bAsset` tokens used as collateral by `basset_vault`.

## nAsset token config holder

Helper contract. CW20 contract have no ability to reward token holders, so some workaround needed. (same as in `bLuna`)

## nAsset token rewards

Helper contract. CW20 contract have no ability to reward token holders, so some workaround needed. (same as in `bLuna`)
