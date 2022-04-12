# Description
This is an example of upgrading rust smart contracts on near. You can use a couple different approaches, depending on the complexity of your contract.

There are two popular ways: 
- Using [migration method](https://www.near-sdk.io/upgrading/production-basics#migration-method)
- Using [enums](https://www.near-sdk.io/upgrading/production-basics#using-enums)
  
# Contracts
1.  Initial contract [not-upgraded-contract](not-upgraded-contract/)
2.  Upgrade using [migration method](migration-upgrade-contract/) 
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
result:
```json
1
```
So the discount field added to the contract.

# Enums
You have a deployed contract and you want to change something in the sub structure of the contract without losing old state. Or you can't move huge structure in single call of migration method. Here the enums can help you to do that.

## Example of enums
At first, let's deploy our initial smart contract.

With the structures: 
```rust
pub struct Sale {
    item: String,
    price: u128,
    sold: bool,
}

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

Now we want to upgrade our Sale structure, so it looks like this:
```rust
pub struct Sale {
    saler: AccountId, // now anyone can sell their items on this contract
    item: String,
    price: u128,
    amount: u8, // and we can sell more then one item
}
```

To do so we can use [enum](enum-upgraded-contract/src/lib.rs#L26) and have to [implement](enum-upgraded-contract/src/lib.rs#L31) `From<UpgradableSale> for Sale` so your contract know how to use adopt old structure

Also would be useful to [implement](enum-upgraded-contract/src/lib.rs#L45) `From<Sale> for UpgradableSale` so we can call `.into()` method when you insert your Sales.

Add [migrate method](enum-upgraded-contract/src/lib.rs#L71) and replace [get(&sale_id) calls](enum-upgraded-contract/src/lib.rs#L102). With [method](enum-upgraded-contract/src/lib.rs#L116) that will keep `legacy_sales`, or the [method](enum-upgraded-contract/src/lib.rs#L125), that will move old sales to the `sale`(if you want to get rid of `legacy_sales` at the end).
Inside `enum-upgraded-contract/` directory:
```bash
./build.sh
near deploy $CONTRACT_NAME res/sale_contract.wasm migrate '{}' 
```

Now we can test it
```bash
near call $CONTRACT_NAME buy '{"sale_id": 1}' --accountId $CONTRACT_NAME --depositYocto 10000
near call $CONTRACT_NAME add_sale '{"item": "banana", "price": "500", "amount": 5}' --accountId $CONTRACT_NAME
near view $CONTRACT_NAME get_sale '{"sale_id": 2}'
```
result
```json
{
  saler: 'dev-1649758277593-26135201163112',
  item: 'banana',
  price: 500,
  amount: 5
}
```

And now you can add new versions to UpgradableSale, without migrating it.