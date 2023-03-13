pub mod error;
pub mod api_types;
pub mod panera_client;

use std::process::exit;

use panera_client::Sippy;
use error::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {

    ///Save your panera credentials for use with the order command
    Login {
        login_packet: String,
        /// Customer rewards number. required to allow sip club to be applied successfully
        loyalty_num: String,
    },
    ///Get and print the menu for the specified panera location
    Menu {
        /// The panera store ID. can be found on the panera website
        location: i32,
    },

    ///Order the given food item (check menu) at the given location using sip club.
    Order {
        /// The panera store ID. can be found on the panera website
        location: i32, 
        /// ID of the requested food. Check menu
        food: i32,

        /// The message to be passed to the kitchen, such as 'milk please'
        #[arg(short, long, default_value = "")]
        kitchen_message: String,

        /// Name to be written on order
        #[arg(short, long, default_value = "")]
        prepared_for_message: String,
    },
}

fn run() -> Result<()> {
    let mut client : Sippy = Sippy::new();
    let args = Args::parse();

    match args.action {
        Action::Login { login_packet, loyalty_num } => {
            if let Err(msg) = client.login(&login_packet, loyalty_num){
                eprintln!("Problem parsing provided login response: {}", msg);
            };
        }

        Action::Menu { location } => {
            client.get_menu(location)?.iter().for_each({|optset|
                println!("{:8} {:6} | {} - {}", optset.itemId, optset.price, optset.i18nName, optset.logicalName)
            });
        }

        Action::Order { location, food, kitchen_message, prepared_for_message } => {
            if let Err(msg) = client.load_creds(){
                eprintln!("Problem loading credentials: {}", msg);         
                eprintln!("Make sure that you've run 'login' before.");         
                panic!();
            }
            let cart_id = client.create_cart(location)?;
            client.add_item(food, &cart_id, &kitchen_message, &prepared_for_message)?;
            client.apply_sip_club(&cart_id)?;
            client.checkout(&cart_id, location)?;
            println!("Item ordered successfully.");
        }
    }

    Ok(())
}

fn main() {
    match run() {
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        },
        Ok(()) => {
            exit(0);
        }
    }
}
