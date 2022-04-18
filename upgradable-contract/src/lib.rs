use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::Serialize;
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
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Sale {
    seller: AccountId,
    item: String,
    price: u128,
    amount: u64,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleJson {
    seller: AccountId,
    item: String,
    price: U128,
    amount: U64,
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
                amount: 1,
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
    sales: LookupMap<SaleId, UpgradableSale>,
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
    pub fn add_sale(&mut self, item: String, price: U128, amount: U64) -> SaleId {
        let amount: u64 = amount.into();
        require!(amount > 0, "Amount must be greater than 0");
        let sale_id = self.next_sale_id;
        let seller = env::predecessor_account_id();
        self.sales.insert(
            &sale_id,
            &Sale {
                seller,
                item,
                price: price.into(),
                amount,
            }
            .into(), // added .into() when using Sale
        );
        self.next_sale_id += 1;
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
        if sale.amount == 0 {
            self.sales.remove(&sale_id);
        } else {
            self.sales.insert(&sale_id, &sale.into());
        }
        Promise::new(seller).transfer(price)
    }

    pub fn get_sale(self, sale_id: SaleId) -> Option<SaleJson> {
        self.sales.get(&sale_id).map(|sale| {
            let Sale {
                seller,
                item,
                price,
                amount,
            } = sale.into();
            SaleJson {
                seller,
                item,
                price: price.into(),
                amount: amount.into(),
            }
        })
    }
}
