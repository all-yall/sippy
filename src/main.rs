use reqwest::blocking::{Response, RequestBuilder};
/* Commands I want to implement
* login         -- gives prompt to enter login
* list menu     -- tries to list all menu items in json
* order         -- takes a list of food items and makes order
* --sip-club -- makes order apply sip club discount
* --location -- specify location
*/
const UNION_ID : u32 = 203162;
const PAN_BASE : &str = "https://services-mob.panerabread.com";

struct PaneraClient {
    client : reqwest::blocking::Client,
}

impl PaneraClient {
    fn new() -> Self {
       Self { client: reqwest::blocking::Client::new() }
    }

    fn add_headers(&self, req: RequestBuilder) -> RequestBuilder {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("api_token", "test".parse().unwrap());
        headers.insert("api_token", "bcf0be75-0de6-4af0-be05-13d7470a85f2".parse().unwrap());
        headers.insert("appVersion", "4.71.0".parse().unwrap());
		headers.insert("Content-Type", "application/json".parse().unwrap());
		headers.insert("User-Agent", "Panera/4.69.9 (iPhone; iOS 16.2; Scale/3.00)".parse().unwrap());
        req.headers(headers)
    }

    fn get(&self, path: &str) -> Response {
        let req_url = format!("{base}/{path}", base = PAN_BASE, path = path);
        let req = self.client.get(req_url);
        let req = self.add_headers(req);

        match req.send() {
            Err(err) => {
                panic!("Error while sending request: {:?}", err);
            },
            Ok(ret) => {
                ret
            }
        }
    }
}

fn main() {
    let client : PaneraClient = PaneraClient::new();
    let response = client.get(&format!("/{}/menu/version", UNION_ID)[..]);

    println!("{}", response.text().unwrap());
}
