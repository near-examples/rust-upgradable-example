# Description
This is an example of upgrading rust smart contracts on near. You can use a couple different approaches, depending on the complexity of your contract.

There are two popular ways: 
- Using [migration method](https://www.near-sdk.io/upgrading/production-basics#migration-method)
- Using [enums](https://www.near-sdk.io/upgrading/production-basics#using-enums)
  
# Contracts
1.  Initial contract [not-upgraded-contract](not-upgraded-contract/)
2.  Upgrade using migration [method](migration-upgrade-contract/) 
3.  Upgrade using [enums](enum-upgraded-contract/) 

# Migration
You have a deployed contract and you want to change something in the main structure of the contract without losing old state. Here the migration method will help you to do that.

## Example of migration
At first, let's deploy our initial smart contract.

With the main structure: 
```rust
pub struct Contract {
    sales: UnorderedMap<SaleId, Sale>,
}
```

Inside `not-upgraded-contract/` directory:
```bash
./build.sh
rm -rf neardev # In case you already have neardev
near dev-deploy res/sale_contract.wasm
source neardev/dev-account.env
```

And fill the state a little
```bash
near call $CONTRACT_NAME add_sale '{"item": "foo", "price": "42"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME add_sale '{"item": "bar", "price": "10000"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME buy '{"sale_id": 0}' --accountId $CONTRACT_NAME --depositYocto 42
```

Now we want to upgrade our contract structure, so it looks like this:
```rust
pub struct Contract {
    sales: UnorderedMap<SaleId, Sale>,
    discount: UnorderedMap<AccountId, u32>,
}
```

To do so we should keep [OldContract](migration-upgrade-contract/src/lib.rs#L24) structure and create migrate method [like that](migration-upgrade-contract/src/lib.rs#L41)

Let's migrate our upgraded contract

Inside `migration-upgrade-contract/` directory:
```bash
./build.sh
near deploy $CONTRACT_NAME res/sale_contract.wasm migrate '{}'
```
Now we can test it
```bash
near call $CONTRACT_NAME buy '{"sale_id": 1}' --accountId $CONTRACT_NAME --depositYocto 10000
near view $CONTRACT_NAME get_discount '{"user": "'$CONTRACT_NAME'"}'
```
So the discount field added to the contract.