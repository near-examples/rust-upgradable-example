use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;
use near_sdk::{env, AccountId, Promise};
use near_sdk::{
    json_types::{U128, U64},
    near_bindgen, require,
};

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
    seller: AccountId,
    item: String,
    price: u128,
    amount: u64,
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
            UpgradableSale::V1(salev1) => Self {
                seller: env::current_account_id(),
                item: salev1.item,
                price: salev1.price,
                amount: !salev1.sold as u64,
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
            sales: UnorderedMap::new(b"s"),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn add_sale(&mut self, item: String, price: U128, amount: U64) -> SaleId {
        let sale_id = self.sales.len() as u64;
        let seller = env::predecessor_account_id();
        self.sales.insert(
            &sale_id,
            &Sale {
                seller,
                item,
                price: price.into(),
                amount: amount.into(),
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
            format!("Attached deposit is not equal to the price ({})", price)
        );
        sale.amount -= 1;
        let seller = sale.seller.clone();
        self.sales.insert(&sale_id, &sale.into());
        Promise::new(seller).transfer(price)
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<String> {
        self.sales.get(&sale_id).map(|sale| {
            let Sale {
                seller,
                item,
                price,
                amount,
            } = sale.into();

            // we do this because u128 and u64 are too big to be valid json numbers.
            json!({
                "seller": seller,
                "item": item,
                "price": U128(price),
                "amount": U64(amount),
            })
            .to_string()
        })
    }
}
