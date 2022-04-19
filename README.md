# Description
This is an example of upgrading rust smart contracts on near. You can use a couple different approaches, depending on the complexity of your contract.

There are two popular ways: 
- Using [migration method](https://www.near-sdk.io/upgrading/production-basics#migration-method)
- Using [enums](https://www.near-sdk.io/upgrading/production-basics#using-enums)
  
# Contracts
1.  Initial contract [initial-contract](initial-contract/)
2.  Upgrade using [migration method](migration-upgraded-contract/) 
3.  Upgrade using [enums](enum-upgraded-contract/) 
4.  Upgradable contract with [enums](upgradable-contract/)
# Migration
When you have a deployed contract and you need to change something in the main structure of the contract without losing the old state. It is possible to do using migration method.

## Example of migration
At first, let's deploy our initial smart contract.

With the main structure: 
```rust
pub struct Contract {
    sales: LookupMap<SaleId, Sale>,
    next_sale_id: SaleId,
}
```

Inside of `initial-contract/` directory execute
```bash
./build.sh
rm -rf neardev # In case you already have neardev
near dev-deploy res/initial_sale_contract.wasm
source neardev/dev-account.env
```

Add some data to our state
```bash
near call $CONTRACT_NAME add_sale '{"item": "foo", "price": "42"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME add_sale '{"item": "bar", "price": "10000"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME buy '{"sale_id": 0}' --accountId $CONTRACT_NAME --depositYocto 42
```

Later we realised that we also want to have discounts for our users. The main structure will look like this:
```rust
pub struct Contract {
    sales: LookupMap<SaleId, Sale>,
    discount: LookupMap<AccountId, u32>,
    next_sale_id: SaleId,
}
```

To do so we should keep [OldContract](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/migration-upgraded-contract/src/lib.rs#L33) structure and create migrate method [like that](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/migration-upgraded-contract/src/lib.rs#L51)

Let's migrate our upgraded contract

Inside `migration-upgraded-contract/` directory:
```bash
./build.sh
near deploy $CONTRACT_NAME res/migration_upgraded_sale_contract.wasm migrate '{}'
```
Now we can test it
```bash
near call $CONTRACT_NAME buy '{"sale_id": 1}' --accountId $CONTRACT_NAME --depositYocto 10000
near view $CONTRACT_NAME get_discount '{"user": "'$CONTRACT_NAME'"}'
```
result:
```json
1
```
So the discount field added to the contract.

# Enums
You have a deployed contract and you want to change something in the sub structure of the contract without losing old state. Or you can't move huge structure in single call of migration method. Here the enums can help you to do that.

## Example of enums upgrade
At first, let's deploy our initial smart contract.

With the structures: 
```rust
pub struct Sale {
    item: String,
    price: u128,
}

pub struct Contract {
    sales: LookupMap<SaleId, Sale>,
    next_sale_id: SaleId,
}
```

Inside `initial-contract/` directory:
```bash
./build.sh
rm -rf neardev # In case you already have neardev
near dev-deploy res/initial_sale_contract.wasm
source neardev/dev-account.env
```

And fill the state a little
```bash
near call $CONTRACT_NAME add_sale '{"item": "foo", "price": "42"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME add_sale '{"item": "bar", "price": "10000"}' --accountId $CONTRACT_NAME
near call $CONTRACT_NAME buy '{"sale_id": 0}' --accountId $CONTRACT_NAME --depositYocto 42
```

Now we want to upgrade our Sale structure, so it looks like this:
```rust
pub struct Sale {
    seller: AccountId, // now anyone can sell their items on this contract
    item: String,
    price: u128,
    amount: u64, // and we can sell more then one item
}
```

To do so we can use [enum](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L34) and have to [implement](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L40) `From<UpgradableSale> for Sale` so your contract know how to adopt old structure.

Also would be useful to [implement](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L54) `From<Sale> for UpgradableSale` so we can call `.into()` method when you insert your Sales.

Add [migrate method](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L80) so we can use `UpgradableSale` and replace [get(&sale_id) calls](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L118). With [method](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L136) that will keep `legacy_sales`, or the [method](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/enum-upgraded-contract/src/lib.rs#L152), that will move old sales to the `sale`(if you want to get rid of `legacy_sales` at the end).

Inside `enum-upgraded-contract/` directory:
```bash
./build.sh
near deploy $CONTRACT_NAME res/enum_upgraded_sale_contract.wasm migrate '{}' 
```

Now we can test it
```bash
near call $CONTRACT_NAME buy '{"sale_id": 1}' --accountId $CONTRACT_NAME --depositYocto 10000
near call $CONTRACT_NAME add_sale '{"item": "banana", "price": "500", "amount": "5"}' --accountId $CONTRACT_NAME
near view $CONTRACT_NAME get_sale '{"sale_id": 2}'
```
result
```javascript
'{"seller":"dev-1650279768195-86068399571956","item":"banana","price":"500","amount":"5"}'
```

And now you can add new versions to UpgradableSale, without migrating it.

# Upgradable contract
If you plan to upgrade your contracts throughout their lifetime, start with enums. Adding them only after you decide to upgrade is (usually) possible, but will result in harder-to-follow (and thus more error-prone) code.

## Example of upgradable contract
[Contract](upgradable-contract/) already using enums, so you can just upgrade it by adding new Enum variants to [UpgradableSale](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/upgradable-contract/src/lib.rs#L35). And fixing [implementation](https://github.com/near-examples/rust-upgradable-example/blob/cd8b7c5a437906e2ff040ac05176ebe5e025ea66/upgradable-contract/src/lib.rs#L41) `From<UpgradableSale> for Sale` so your contract know how to use adopt old variants of Sale.