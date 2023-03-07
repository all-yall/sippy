#![allow(non_snake_case)]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MenuVersion {
    pub aggregateVersion : String,
}


#[derive(Deserialize)]
pub struct Menu {
    pub placards: BTreeMap<String, Placard>
}

#[derive(Deserialize)]
pub struct Placard {
    pub optSets: Option<Vec<Optset>>
}


#[derive(Deserialize)]
pub struct Optset {
    pub itemId: i32,
    pub i18nName: String,
    pub logicalName: String,
    pub price: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub credentials: Credentials,
}

#[derive(Deserialize)]
pub struct LoginResp {
    pub token :String,
    pub emails : Vec<Email>,
    pub phones : Vec<Phone>,
    pub firstName: String,
    pub lastName: String,
    pub phoneNumber: String,
    pub customerId: i32,
    pub loyalty: Loyalty,
}

#[derive(Deserialize)]
pub struct Email {
    pub emailAddress: String,
    pub isDefault: bool,
}


#[derive(Deserialize)]
pub struct Phone {
    pub phoneNumber: String,
    pub isDefault: bool,
}

#[derive(Deserialize)]
pub struct Loyalty {
    pub cardNumber: String,
}

#[derive(Deserialize, Serialize)]
pub struct Credentials {
    pub accessToken     :String,
    pub username        :String,
    pub lastName        :String,
    pub firstName       :String,
    pub customerId      :i32,
}

#[derive(Serialize)]
pub struct Cart {
    pub createGroupOrder: bool,
    pub customer:   Customer,
    pub cafes:      Vec<Cafe>,
    pub serviceFeeSupported: bool,
    pub applyDynamicPricing: bool,
    pub cartSummary: CartSummary,
}

#[derive(Serialize)]
pub struct Customer {
    pub email: String,
    pub id: i32,
    pub lastName: String,
    pub firstName: String,
    pub identityProvider: String,
}

#[derive(Serialize)]
pub struct Cafe {
    pub id: i32,
}

#[derive(Serialize)]
pub struct CartSummary {
    pub destination :String,
    pub priority :String,
    pub clientType :String,
    pub deliveryFee :f32,
    pub leadTime :f64,
    pub languageCode :String,
}

#[derive(Deserialize)]
pub struct CartResp {
    pub cartId:     String,
}

#[derive(Serialize)]
pub struct DiscountReq {
    pub discounts :Vec<Discount>
}

#[derive(Serialize)]
pub struct Discount {
    #[serde(rename = "type")]
    pub discountType: String,
    pub promoCode: String,
}

#[derive(Serialize)]
pub struct ItemAdd {
    pub items: Vec<FoodItem>
}

#[derive(Serialize)]
pub struct FoodItem {
    pub msgKitchen: String,
    pub isNoSideOption: bool,
    pub itemId: i32,
    pub parentId: i32,
    pub msgPreparedFor: String,
    #[serde(rename = "type")]
    pub foodType: String,
    pub promotional: bool,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct CheckoutReq {
    pub payment: Payment,
}

#[derive(Serialize)]
pub struct Payment {
    pub giftCards: Vec<Empty>,
    pub creditCards: Vec<Empty>,
    pub campusCards: Vec<Empty>,
}

#[derive(Deserialize, Serialize)]
pub struct Empty {}


