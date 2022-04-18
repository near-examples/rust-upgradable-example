use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, json_types::U128, near_bindgen, require};

pub type SaleId = u64;

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
    next_sale_id: SaleId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            sales: LookupMap::new(b"s"),
            next_sale_id: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
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
        require!(
            env::attached_deposit() == sale.price,
            "Not enough balance to buy this item"
        );
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<SaleJson> {
        let sale = self.sales.get(&sale_id);
        sale.map(|s| SaleJson {
            item: s.item,
            price: U128(s.price),
        })
    }
}
