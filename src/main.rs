pub mod api_types;
pub mod panera_client;

use std::{process::exit, collections::BTreeSet};

use panera_client::Sippy;
use clap::Parser;
use anyhow::{Result, Context};

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

        /// Do not filter the menu to only sip club elligible items.
        /// if this isn't included, you can only view the menu while 
        /// you have a sip club coupon available
        #[arg(short, long)]
        no_filter: bool,
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
    let args = Args::parse();

    match args.action {
        Action::Login { login_packet, loyalty_num } => {
            panera_client::login(&login_packet, loyalty_num)
                .context("While Logging in")?;
        }

        Action::Menu { location, no_filter} => {
            let client = Sippy::try_new()
                .context("While creating client")?;

            let mut items = client.get_menu(location)
                .context("While fetching menu items")?;
            
            if !no_filter {
                let sip_club_items : BTreeSet<i32> = client
                    .get_sip_club_items()?
                    .into_iter()
                    .collect();

                items = items
                    .into_iter()
                    .filter(move |item| sip_club_items.contains(&item.itemId))
                    .collect();
            }
            
            items.into_iter().for_each({|optset|
                println!("{:8} {:6} | {} - {}", optset.itemId, optset.price, optset.i18nName, optset.logicalName)
            });
        }

        Action::Order { location, food, kitchen_message, prepared_for_message } => {
            let client = Sippy::try_new()
                .context("While creating client")?;

            // Needs to be called to update panera that another Sip club is ready
            client.get_sip_club_items()?; 

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
            eprintln!("Error in sippy\n\n{:?}", e);
            exit(1);
        },
        Ok(()) => {
            exit(0);
        }
    }
}
