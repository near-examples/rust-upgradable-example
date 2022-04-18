use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, json_types::U128, near_bindgen, require};

pub type SaleId = u64;

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
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            sales: UnorderedMap::new(b"s"),
        }
    }
}

#[near_bindgen]
impl Contract {
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
        require!(!sale.sold, "Sale already sold");
        require!(
            env::attached_deposit() == sale.price,
            "Not enough balance to buy this item"
        );
        sale.sold = true;
        self.sales.insert(&sale_id, &sale);
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<Sale> {
        self.sales.get(&sale_id)
    }
}
