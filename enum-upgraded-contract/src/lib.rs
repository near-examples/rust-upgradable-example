use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::Serialize;
use near_sdk::{env, AccountId, Promise};
use near_sdk::{near_bindgen, require};

pub type SaleId = u64;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SaleV1 {
    item: String,
    price: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
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
    legacy_sales: LookupMap<SaleId, SaleV1>,
    sales: LookupMap<SaleId, UpgradableSale>,
    next_sale_id: SaleId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            legacy_sales: LookupMap::new(b"s"),
            sales: LookupMap::new(b"n"),
            next_sale_id: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct OldContract {
            sales: LookupMap<SaleId, SaleV1>,
            next_sale_id: SaleId,
        }
        let old_state: OldContract = env::state_read().expect("failed");
        Contract {
            legacy_sales: old_state.sales,
            sales: LookupMap::new(b"n"),
            next_sale_id: old_state.next_sale_id,
        }
    }

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
            .get_sale_internal(sale_id)
            .expect("No sale with this id");
        require!(sale.amount > 0, "Sale already sold");
        let price = sale.price;
        require!(
            env::attached_deposit() == price,
            format!("Attached deposit is not equal to price({})", sale.price)
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
        let sale: Option<Sale> = self
            .legacy_sales
            .get(&sale_id)
            .map(UpgradableSale::V1)
            .or_else(|| self.sales.get(&sale_id))
            .map(|sale| sale.into());
        sale.map(|s| SaleJson {
            seller: s.seller,
            item: s.item,
            price: s.price.into(),
            amount: s.amount.into(),
        })
    }

    // When the legacy_sales field will be empty you can remove field from the struct
    fn get_sale_internal(&mut self, sale_id: SaleId) -> Option<Sale> {
        if let Some(legacy_sale) = self.legacy_sales.remove(&sale_id) {
            self.sales
                .insert(&sale_id, &UpgradableSale::V1(legacy_sale));
        }
        self.sales.get(&sale_id).map(|sale| sale.into())
    }
}
