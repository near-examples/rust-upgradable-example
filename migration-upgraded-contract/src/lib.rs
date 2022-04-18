use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId};
use near_sdk::{json_types::U128, near_bindgen, require};

pub type SaleId = u64;

const MAX_DISCOUNT: u32 = 20;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sale {
    item: String,
    price: u128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleJson {
    item: String,
    price: U128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    sales: LookupMap<SaleId, Sale>,
    discount: LookupMap<AccountId, u32>,
    next_sale_id: SaleId,
}

// This struct can be removed after upgrade
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldContract {
    sales: LookupMap<SaleId, Sale>,
    next_sale_id: SaleId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            sales: LookupMap::new(b"s"),
            discount: LookupMap::new(b"d"),
            next_sale_id: 0,
        }
    }
}
#[near_bindgen]
impl Contract {
    // This method can be removed after upgrade
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: OldContract = env::state_read().expect("failed to read old state");
        Contract {
            sales: old_state.sales,
            discount: LookupMap::new(b"d"),
            next_sale_id: old_state.next_sale_id,
        }
    }

    // private because only the owner can add items for sale (?)
    #[private]
    pub fn add_sale(&mut self, item: String, price: U128) -> SaleId {
        let sale_id = self.next_sale_id;
        self.sales.insert(
            &sale_id,
            &Sale {
                item,
                price: price.into(),
            },
        );
        self.next_sale_id += 1;
        sale_id
    }

    #[payable]
    pub fn buy(&mut self, sale_id: SaleId) {
        let sale: Sale = self.sales.remove(&sale_id).expect("No sale with this id");
        let user = env::predecessor_account_id();
        let discount = self.discount.get(&user).unwrap_or(0);
        let price = sale.price * (100 - discount) as u128 / 100;
        require!(
            env::attached_deposit() == price,
            "Not enough balance to buy this item"
        );

        // increase the discount by 1 percent for every sale up to max discount
        let new_discount = std::cmp::min(self.discount.get(&user).unwrap_or(0) + 1, MAX_DISCOUNT);
        self.discount.insert(&user, &new_discount);
    }

    /// returns the price of the item with discount
    pub fn get_price(&self, sale_id: SaleId, user: AccountId) -> Option<U128> {
        self.sales.get(&sale_id).map(|s| {
            let discount = self.discount.get(&user).unwrap_or(0);
            let price = s.price * (100 - discount) as u128 / 100;
            U128(price)
        })
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<SaleJson> {
        let sale = self.sales.get(&sale_id);
        sale.map(|s| SaleJson {
            item: s.item,
            price: U128(s.price),
        })
    }

    /// returns discount (%) for particular user
    pub fn get_discount(self, user: AccountId) -> u32 {
        self.discount.get(&user).unwrap_or(0)
    }
}
