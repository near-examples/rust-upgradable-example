use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, json_types::U128, near_bindgen, require};

pub type SaleId = u64;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sale {
    item: String,
    price: u128,
    sold: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    sales: UnorderedMap<SaleId, Sale>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            sales: UnorderedMap::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn add_sale(&mut self, item: String, price: U128) {
        let sale_id = self.sales.len() as u64;
        self.sales.insert(
            &sale_id,
            &Sale {
                item,
                price: price.into(),
                sold: false,
            },
        );
    }

    #[payable]
    pub fn buy(&mut self, sale_id: SaleId) {
        let mut sale: Sale = self.sales.get(&sale_id).unwrap();
        require!(!sale.sold, "Sale already sold");
        require!(
            env::attached_deposit() == sale.price,
            "Not enough balance to buy this item"
        );
        sale.sold = true;
        self.sales.insert(&sale_id, &sale);
    }
}
