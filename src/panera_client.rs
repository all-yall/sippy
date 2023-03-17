use std::{fs, path::PathBuf};
use serde::{de::DeserializeOwned, ser::Serialize};
use reqwest::{blocking::RequestBuilder, Method};
use crate::api_types::*;
use anyhow::{Result, Context};

const PAN_BASE : &str = "https://services-mob.panerabread.com";
const SETTINGS_FILE: &str = "sippy.json";

fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir()
        .expect("config dir should exist on all supported platforms");
    path.push(SETTINGS_FILE);
    path
}

pub fn login(login_packet: &str, loyalty_num: String) -> Result<()> {
    let login_resp: Credentials = serde_json::from_str(login_packet)
        .context("While parsing JSON login response (Did you select the right packet?)")?;

    let settings = Settings {
        credentials: login_resp,
        loyalty_num,
    };

    let path = get_settings_path();
    let contents = serde_json::to_string(&settings)
        .expect("settings object should serialize");
    fs::write(path, contents)
        .context("Problem writing credentials to file (Do you have permissions?)")?;

    Ok(())
}

fn load_creds() -> Result<Settings> {
    let path = get_settings_path();
    let data = fs::read_to_string(&path)
        .context("While reading settings file (Make sure you've logged in before running other commands)")?;

    let settings: Settings = serde_json::from_str(&data)
        .context("While loading JSON in settings file (Try re-logging in)")?;

    Ok(settings)
}

pub struct Sippy {
    client : reqwest::blocking::Client,
    settings: Settings,
}


impl Sippy {
    pub fn try_new() -> Result<Self> {
        let settings = load_creds()?;
        let client = reqwest::blocking::Client::new();
        Ok(Self{client, settings})
    }

    fn add_headers(&self, req: RequestBuilder) -> RequestBuilder {
        let mut headers = reqwest::header::HeaderMap::new();
        // This is not a private API token; it is embedded in all Panera apps
        headers.insert("api_token", "bcf0be75-0de6-4af0-be05-13d7470a85f2".parse().unwrap());
        headers.insert("appVersion", "4.71.0".parse().unwrap());
		headers.insert("Content-Type", "application/json".parse().unwrap());
		headers.insert("User-Agent", "Panera/4.73.1 (iPhone; iOS 16.2; Scale/3.00)".parse().unwrap());
        headers.insert("auth_token", self.settings.credentials.accessToken.parse().unwrap());
        headers.insert("deviceId", self.settings.credentials.accessToken.parse().unwrap());
        req.headers(headers)
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let req_url = format!("{base}{path}", base = PAN_BASE, path = path);
        let req = self.client.request(method, req_url);
        let req = self.add_headers(req);
        req
    }

    fn send_and_marshal<R:DeserializeOwned>(&self, req: RequestBuilder) -> Result<R> {
        let resp = req.send().context("Error while sending request")?;

        resp.error_for_status()
            .context("Bad response code from API")?
            .json::<R>()
            .context("Error parsing json sent from API")
    }

    fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let req = self.request(Method::GET, path);
        self.send_and_marshal(req)
    }

    fn post<S: Serialize, R: DeserializeOwned>(&self, path: &str, obj: S) -> Result<R> {
        let req = self.request(Method::POST, path).json(&obj);
        self.send_and_marshal(req)
    }

    fn put<S: Serialize, R: DeserializeOwned>(&self, path: &str, obj: S) -> Result<R> {
        let req = self.request(Method::PUT, path).json(&obj);
        self.send_and_marshal(req)
    }


    pub fn get_menu(&self, location_id: i32) -> Result<Vec<Optset>> {
        let mv: MenuVersion = self.get(&format!("/{}/menu/version", location_id))?;
        let menu: Menu = self.get(&format!("/en-US/{}/menu/v2/{}", location_id, mv.aggregateVersion))?;

        let ret = menu.placards
            .into_values()
            .into_iter()
            .filter_map(|placard| placard.optSets)
            .flat_map(|optsets| optsets.into_iter())
            .collect();

        Ok(ret)
    }

    pub fn create_cart(&self, location_id: i32) -> Result<String> {
        let creds = &self.settings.credentials;
        let cart = Cart {
            createGroupOrder: false,
            customer: Customer { 
                email: creds.username.clone(),
                firstName: creds.firstName.clone(),
                lastName: creds.lastName.clone(),
                identityProvider: "PANERA".to_string(),
                id: creds.customerId,
            },
            cafes: vec![
                Cafe {
                    id: location_id,
                }
            ],
            serviceFeeSupported: true,
            applyDynamicPricing: true,
            cartSummary: CartSummary {
                destination: "RPU".to_string(),
                priority: "ASAP".to_string(),
                clientType: "MOBILE_IOS".to_string(),
                deliveryFee: 0.0,
                leadTime: 10.0,
                languageCode: "en-US".to_string(),
            }
        };

        let cart_resp: CartResp = self.post("/cart", cart)
            .context("Creating cart")?;

        Ok(cart_resp.cartId)
    }

    pub fn add_item(&self, item_id: i32, cart_id: &str,  kitchen_message: &str, prepared_for: &str) -> Result<()>  {
        let item = FoodItem {
            msgKitchen: kitchen_message.to_string(),
            msgPreparedFor: prepared_for.to_string(),
            isNoSideOption: false,
            parentId: 0,
            itemId: item_id,
            quantity: 1,
            foodType: "PRODUCT".to_string(),
            promotional: false,
        };

        let add_item = ItemAdd {
            items: vec![ item ],
        };

        let req_path = format!("/v2/cart/{}/item", cart_id);

        let _ : Empty = self.post(&req_path, add_item)
            .context("Adding Item")?;

        Ok(())
    }

    pub fn apply_sip_club(&self, cart_id: &str) -> Result<()> {
        let req_path = format!("/cart/{}/discount", cart_id);
        let sip_club_discount = DiscountReq {
            discounts: vec![
                Discount {
                    discountType: "WALLET_CODE".to_string(),
                    promoCode: "1238".to_string(),
                }
            ]
        };
        let _ : Empty = self.post(&req_path, sip_club_discount)
            .context("Applying Discount Code")?;
        Ok(())
    }

    pub fn checkout(&self, cart_id: &str, location_id: i32) -> Result<()> {

        let req_url = format!("/cart/{}/checkout?summary=true", cart_id);
        let data = serde_json::json!({"summary" : true});
        let _ : Empty = self.post(&req_url, data)
            .context("Checking Out")?;

        let settings = &self.settings;
        let creds = &settings.credentials;

        let data = serde_json::json!({
            "cafes": [
                {
                    "id": location_id,
                    "pagerNum": 0
                }
            ],
            "cartSummary": {
                "clientType": "MOBILE_IOS",
                "deliveryFee": "0.00",
                "destination": "RPU",
                "goGreen": true,
                "languageCode": "en-US",
                "leadTime": 10,
                "priority": "ASAP",
                "specialInstructions": "",
                "tip": "0.00"
            },
            "customer": {
                "email": creds.username,
                "firstName": creds.firstName,
                "lastName": creds.lastName,
                "id": creds.customerId,
                "identityProvider": "PANERA",
                "loyaltyNum": &settings.loyalty_num,
            },
            "serviceFeeSupported": true
        });

        let req_url = format!("/cart/{}", cart_id);
        let _ : Empty = self.put(&req_url, data)?;

        let req_url = format!("/payment/v2/slot-submit/{}", cart_id);
        let checkout_req = CheckoutReq {
            customer: CustomerSMS {
                smsOptIn: false
            },
            payment: Payment {
                giftCards: vec![],
                creditCards: vec![],
                campusCards: vec![],
            }
        };

        let _: Empty = self.post(&req_url, checkout_req)
            .context("Paying For Order")?;

        Ok(())
    }
}

