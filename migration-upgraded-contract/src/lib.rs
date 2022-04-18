use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId};
use near_sdk::{json_types::U128, near_bindgen, require};

pub type SaleId = u64;

const MAX_DISCOUNT: u32 = 20;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    item: String,
    price: u128,
    sold: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    sales: UnorderedMap<SaleId, Sale>,
    discount: UnorderedMap<AccountId, u32>,
}

// This struct can be removed after upgrade
#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldContract {
    sales: UnorderedMap<SaleId, Sale>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            sales: UnorderedMap::new(b"s"),
            discount: UnorderedMap::new(b"d"),
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
            discount: UnorderedMap::new(b"d"),
        }
    }

    // private because only the owner can add items for sale (?)
    #[private]
    pub fn add_sale(&mut self, item: String, price: U128) -> SaleId {
        let sale_id = self.sales.len() as u64;
        self.sales.insert(
            &sale_id,
            &Sale {
                item,
                price: price.into(),
                sold: false,
            },
        );
        sale_id
    }

    #[payable]
    pub fn buy(&mut self, sale_id: SaleId) {
        let mut sale: Sale = self.sales.get(&sale_id).expect("No sale with this id");
        let user = env::predecessor_account_id();
        require!(!sale.sold, "Sale already sold");

        let price = self.get_price_internal(&sale, &user);
        require!(
            env::attached_deposit() == price.into(),
            "Not enough balance to buy this item"
        );
        sale.sold = true;
        self.sales.insert(&sale_id, &sale);

        // increase the discount by 1 percent for every sale up to max discount
        let new_discount = std::cmp::min(self.discount.get(&user).unwrap_or(0) + 1, MAX_DISCOUNT);
        self.discount.insert(&user, &new_discount);
    }

    /// returns the price of the item with discount
    pub fn get_price(&self, sale_id: SaleId, user: AccountId) -> U128 {
        let sale: Sale = self.sales.get(&sale_id).unwrap();
        self.get_price_internal(&sale, &user).into()
    }

    fn get_price_internal(&self, sale: &Sale, user: &AccountId) -> u128 {
        let discount = self.discount.get(user).unwrap_or(0);
        sale.price * (100 - discount) as u128 / 100
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<Sale> {
        self.sales.get(&sale_id)
    }

    /// returns discount (%) for particular user
    pub fn get_discount(self, user: AccountId) -> u32 {
        self.discount.get(&user).unwrap_or(0)
    }
}
