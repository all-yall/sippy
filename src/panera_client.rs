use std::{fs, path::PathBuf};
use serde::{de::DeserializeOwned, ser::Serialize};
use reqwest::{blocking::RequestBuilder, Method};
use crate::api_types::*;

const PAN_BASE : &str = "https://services-mob.panerabread.com";
const SETTINGS_FILE: &str = "sippy.json";

fn get_settings_path() -> PathBuf {
    let mut conf_dir = dirs::config_dir()
        .expect("Fatal Error: Cannot get configuration directory.");
    conf_dir.push(SETTINGS_FILE);
    conf_dir
}

pub struct Sippy {
    client : reqwest::blocking::Client,
    settings: Option<Settings>,
}

impl Sippy {
    pub fn new() -> Self {
        let settings = None;
        let client = reqwest::blocking::Client::new();
        Self{client, settings}
    }

    fn add_headers(&self, req: RequestBuilder) -> RequestBuilder {
        let mut headers = reqwest::header::HeaderMap::new();
        // This is not a private API token; it is embedded in all Panera Apps
        headers.insert("api_token", "bcf0be75-0de6-4af0-be05-13d7470a85f2".parse().unwrap());
        headers.insert("appVersion", "4.71.0".parse().unwrap());
		headers.insert("Content-Type", "application/json".parse().unwrap());
		headers.insert("User-Agent", "Panera/4.73.1 (iPhone; iOS 16.2; Scale/3.00)".parse().unwrap());
        if let Some(settings) = &self.settings {
            headers.insert("auth_token", settings.credentials.accessToken.parse().unwrap());
            headers.insert("deviceId", settings.credentials.accessToken.parse().unwrap());
        }
        req.headers(headers)
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let req_url = format!("{base}{path}", base = PAN_BASE, path = path);
        let req = self.client.request(method, req_url);
        let req = self.add_headers(req);
        req
    }

    fn send_and_marshal<R:DeserializeOwned>(&self, req: RequestBuilder) -> R {
        let resp = req.send().expect("Error while sending request");

        resp.error_for_status()
            .expect("Error in API response")
            .json::<R>()
            .expect("Error parsing json sent from API")
    }

    fn get<R: DeserializeOwned>(&self, path: &str) -> R {
        let req = self.request(Method::GET, path);
        self.send_and_marshal(req)
    }

    fn post<S: Serialize, R: DeserializeOwned>(&self, path: &str, obj: S) -> R {
        let req = self.request(Method::POST, path);
        let req = req.json(&obj);
        self.send_and_marshal(req)
    }

    fn put<S: Serialize, R: DeserializeOwned>(&self, path: &str, obj: S) -> R {
        let req = self.request(Method::PUT, path);
        let req = req.json(&obj);
        self.send_and_marshal(req)
    }

    pub fn get_menu(&self, location_id: i32) -> Vec<Optset> {
        let mv: MenuVersion = self.get(&format!("/{}/menu/version", location_id));
        let menu: Menu = self.get(&format!("/en-US/{}/menu/v2/{}", location_id, mv.aggregateVersion));

        let ret = menu.placards
            .into_values()
            .into_iter()
            .filter_map(|placard| placard.optSets)
            .flat_map(|optsets| optsets.into_iter())
            .collect();

        ret
    }

    pub fn load_creds(&mut self) -> Result<(), String> {
        let path = get_settings_path();
        let data = fs::read_to_string(path)
            .map_err(|e| format!("while reading file; {}", e))?;
        let settings: Settings = serde_json::from_str(&data)
            .map_err(|e| format!("while loading JSON; {}", e))?;

        self.settings = Some(settings); 

        Ok(())
    }

    fn save_creds(&mut self) -> Result<(), String> {
        let path = get_settings_path();
        let settings = self.settings.as_ref()
            .ok_or("Can't save credentials when they were never loaded.")?;
        let contents = serde_json::to_string(settings)
            .map_err(|e| format!("Problem serializing credentials to JSON; {}", e))?;
        fs::write(path, contents)
            .map_err(|e| format!("Problem writing credentials to file; {}", e))?;

        Ok(())
    }

    pub fn login(&mut self, login_packet: &str) -> Result<(), String> {
        let login_resp: Credentials = serde_json::from_str(login_packet)
            .map_err(|e| format!("Problem parsing JSON login response; {}", e))?;
        let settings = Settings {
            credentials: login_resp,
        };
        
        self.settings = Some(settings);

        self.save_creds()
    }

    pub fn create_cart(&self, location_id: i32) -> String {
        let creds = &self.settings.as_ref().expect("Can't create cart when not logged in").credentials;
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

        let cart_resp: CartResp = self.post("/cart", cart);
        cart_resp.cartId
    }

    pub fn add_item(&self, item_id: i32, cart_id: &str,  kitchen_message: &str, prepared_for: &str)  {
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

        let _ : Empty = self.post(&req_path, add_item);
    }

    pub fn apply_sip_club(&self, cart_id: &str) {
        let req_path = format!("/cart/{}/discount", cart_id);
        let sip_club_discount = DiscountReq {
            discounts: vec![
                Discount {
                    discountType: "WALLET_CODE".to_string(),
                    promoCode: "1238".to_string(),
                }
            ]
        };
        let _ : Empty = self.post(&req_path, sip_club_discount);
    }

    pub fn checkout(&self, cart_id: &str) {

        let req_url = format!("/cart/{}/checkout?summary=true", cart_id);
        let data = serde_json::json!({"summary" : true});
        let _ : Empty = self.post(&req_url, data);

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

        let _: Empty = self.post(&req_url, checkout_req);
    }
}


