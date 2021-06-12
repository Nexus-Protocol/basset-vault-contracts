# Yield optimizers

1. [Anchor yield optimizer](#anchor-yield-optimizer)
2. Mirror UST stratege (todo)

## Anchor yield optimizer

### v1

Simple strategy which borrow UST and place it to Anchor Deposit for 20% yeild.
Anchor pays to borrowers, so contract also sell ANC tokens in favor of bAsset depositors and \$PSI token stakers.

Contract ask `basset_farmer_config` on `Rebalance` to know how much to borrow or repay.

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
    Nothing {},
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

[Rebalance config](#anchor-yield-optimizer-config)

#### Repayment logic

If contract needs to repay part of loan (in case of bAsset withdraw or bAsset price drop) it should sell 
some of aUST(withdraw from Anchor Earn). But it is possible that this action will fail - there are too many borrowers
and Anchor prevent withdrawing UST that is borrowed. So, contract will not be able to repay loan in that case...
and will be liquidated.

To avoid that we do not deposit all borrowed UST to Anchor (convering to aUST). Instead portion of UST just stays on the balance
and is used when redeeming aUST returns error.
In case of error we repaying loan from UST on balance, and then redeem aUST for exactly the same amount. Repeat that cycle until
balance achieved.

Take a look at [repay logic](./contracts/basset_farmer/src/commands.rs#L213).

And on [reply handler](./contracts/basset_farmer/src/contract.rs#L115).

#### Borrow logic

Nothing bad will happen if we fail to borrow more, so no error handling here.

### v2

Borrow UST from Anchor and use it in a sophisticated Mirror strategies.
Will be available in the future.

### Anchor yield optimizer config

The simplest config is to maintain bounds:
* borrow to `aim_ltv` if `ltv` < `minimum_ltv` (75%)
* repay to `aim_ltv` if `ltv` > `minimum_ltv` (85%)
* do nothing otherwise

`aim_ltv`: 80%

---

* [anchor-yield-optimizer](#anchor-yield-optimizer)
* [anchor-yield-optimizer-config](#anchor-yield-optimizer-config)
