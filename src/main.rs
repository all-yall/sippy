#![allow(dead_code)]

use std::{fs, path::PathBuf};
mod api_types;
use serde::{de::DeserializeOwned, ser::Serialize};
use reqwest::{blocking::RequestBuilder, Method};
use api_types::*;

/* Commands I want to implement
* login                --  gives prompt to enter login
* list menu            --  tries to list all menu items in json
* order                --  takes a list of food items and makes order
*  --location          --  specify location
*  --kitchen-message   --  specify location
*  --prepared-for      --  specify location
*/

const UNION_ID : i32 = 203162;
const PAN_BASE : &str = "https://services-mob.panerabread.com";
const SETTINGS_FILE: &str = "panera_cli.json";


fn get_settings_path() -> PathBuf {
    let mut conf_dir = dirs::config_dir()
        .expect("Fatal Error: Cannot get configuration directory.");
    conf_dir.push(SETTINGS_FILE);
    conf_dir
}

struct PaneraClient {
    client : reqwest::blocking::Client,
    settings: Option<Settings>,
}

impl PaneraClient {
    fn new() -> Self {
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
		headers.insert("User-Agent", "Panera/4.69.9 (iPhone; iOS 16.2; Scale/3.00)".parse().unwrap());
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

    fn get<R: DeserializeOwned>(&self, path: &str) -> R {
        let req = self.request(Method::GET, path);
        let resp = req.send().expect("Error while sending request");

        resp.error_for_status()
            .expect("Error in API response")
            .json::<R>()
            .expect("Error parsing json sent from API")
    }

    fn post<S: Serialize, R: DeserializeOwned>(&self, path: &str, obj: S) -> R {
        let req = self.request(Method::POST, path);
        let req = req.json(&obj);
        let resp = req.send().expect("Error while sending request");

        resp.error_for_status()
            .expect("Error in API response")
            .json::<R>()
            .expect("Error parsing json sent from API")
    }

    fn get_menu(&self) -> Vec<Optset> {
        let mv: MenuVersion = self.get(&format!("/{}/menu/version", UNION_ID)[..]);
        let menu: Menu = self.get(&format!("/en-US/{}/menu/v2/{}", UNION_ID, mv.aggregateVersion)[..]);

        let ret = menu.placards
            .into_values()
            .into_iter()
            .flat_map(|placard| placard.optSets.into_iter())
            .flat_map(|optsets| optsets.into_iter())
            .collect();

        ret
    }

    fn load_creds(&mut self) -> Result<(), String> {
        let path = get_settings_path();
        let data = fs::read_to_string(path)
            .map_err(|e| format!("while reading file; {}", e))?;
        let settings: Settings = serde_json::from_str(&data[..])
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

    fn login(&mut self, login_packet: &str) -> Result<(), String> {
        let login_resp: Credentials = serde_json::from_str(login_packet)
            .map_err(|e| format!("Problem parsing JSON login response; {}", e))?;
        let settings = Settings {
            credentials: login_resp,
        };
        
        self.settings = Some(settings);

        self.save_creds()
    }

    fn create_cart(&self) -> String {
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
                    id: UNION_ID,
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

    fn add_item(&self, item_id: i32, cart_id: &str)  {
        let item = FoodItem {
            msgKitchen: "".to_string(),
            msgPreparedFor: "".to_string(),
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

    fn apply_sip_club(&self, cart_id: String) {
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

    fn checkout(&self, cart_id: &str) {
        let req_url = format!("/pyment/v2/slot-submit/{}", cart_id);
        let checkout_req = CheckoutReq {
            payment: Payment {
                giftCards: vec![],
                creditCards: vec![],
                campusCards: vec![],
            }
        };

        let _: Empty = self.post(&req_url, checkout_req);
    }
}


fn login(client: &mut PaneraClient) {
    let login_resp = 
    r#"
        {
          "accessToken": "GET-YOUR-OWN-API-KEY",
          "loginDate": "2023-03-03T19:43:36.477+0000",
          "expirationDate": "2023-04-02T19:43:36.477+0000",
          "customerId": 65486285,
          "username": "enemy@purplish.blue",
          "firstName": "Enemy",
          "lastName": "Ally",
          "rewardsCount": 0,
          "isSecure": true,
          "isPersistentSession": true,
          "secureExpiration": "2023-03-03T20:13:36.477Z"
        }
    "#;
    if let Err(msg) = client.login(login_resp){
        eprintln!("Problem parsing provided login response: {}", msg);
    };
}

fn menu(client: &mut PaneraClient) {
    client.get_menu().iter().for_each({|optset|
        println!("{:8} {:6} | {} - {}", optset.itemId, optset.price, optset.i18nName, optset.logicalName)
    });
}

fn order_item(client: &mut PaneraClient, item_id: i32) {
    let cart_id = client.create_cart();
    client.add_item(item_id, &cart_id[..]);
    client.apply_sip_club(cart_id)
}


/*
* Login to panera, find the json request that appears when you search
* for 'details' and copy the response.
*/
fn main() {
    let mut client : PaneraClient = PaneraClient::new();
    if let Err(msg) = client.load_creds(){
        eprintln!("Problem loading credentials: {}", msg);         
        eprintln!("Make sure that you've run 'login' before.");         
    }
    //menu(&mut client);
    order_item(&mut client, 646472)
}

