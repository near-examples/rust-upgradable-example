use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId, Promise};
use near_sdk::{json_types::U128, near_bindgen, require};

pub type SaleId = u64;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SaleV1 {
    item: String,
    price: u128,
    sold: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    saler: AccountId,
    item: String,
    price: u128,
    amount: u8,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum UpgradableSale {
    V1(SaleV1),
    V2(Sale),
}

impl From<UpgradableSale> for Sale {
    fn from(sale: UpgradableSale) -> Self {
        match sale {
            UpgradableSale::V2(sale) => sale,
            UpgradableSale::V1(SaleV1 { item, price, sold }) => Sale {
                saler: env::current_account_id(),
                item,
                price,
                amount: !sold as u8,
            },
        }
    }
}

impl From<Sale> for UpgradableSale {
    fn from(sale: Sale) -> Self {
        UpgradableSale::V2(sale)
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    sales: UnorderedMap<SaleId, UpgradableSale>,
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
    pub fn add_sale(&mut self, item: String, price: U128, amount: u8) -> SaleId {
        let sale_id = self.sales.len() as u64;
        let saler = env::predecessor_account_id();
        self.sales.insert(
            &sale_id,
            &Sale {
                saler,
                item,
                price: price.into(),
                amount,
            }
            .into(), // added .into() when using Sale
        );
        sale_id
    }

    #[payable]
    pub fn buy(&mut self, sale_id: SaleId) -> Promise {
        let mut sale: Sale = self
            .sales
            .get(&sale_id)
            .expect("No sale with this id")
            .into();
        require!(sale.amount > 0, "Sale already sold");
        let price = sale.price;
        require!(
            env::attached_deposit() == price,
            format!("Attached deposit is not equal to price({})", sale.price)
        );
        sale.amount -= 1;
        let saler = sale.saler.clone();
        self.sales.insert(&sale_id, &sale.into());
        Promise::new(saler).transfer(price)
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<Sale> {
        self.sales.get(&sale_id).map(|s| s.into())
    }
}
