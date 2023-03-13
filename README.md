# Introduction
This is a small client used to order sip club drinks from Panera
it is not 'full featured' as you can't do things such as order 
specific variants for tea, or add pumps of syrup to drinks, but 
it can order a Pepsi, a coffee, or other simple drinks.

## Logging In
Before using `sippy`, account credentials need to be provided. To do this, 
go to the Panera website [here](https://www.panera.ca/en-us/home.html). 
Next, open the developer tools with f12 and click on the Network tab of 
the newly opened window. Then, log into Panera using the Sign in button.

Search for the POST request to 'panera' endpoint.
Right click the request and click 'Copy' value followed
by 'Copy Response'. This will be the first argument to the 
login command.

Next search for the GET request to 'rewards?cafeId=' and look
in the response packet for the 'cardNumber'. This will be the 
second argument to the login command.

Finally, run `sippy login '<VALUE_COPIED>' '<CARD NUMBER>'` 
(remember the single quotes around the values). 
The command should look something like:
```bash
sippy login '{"accessToken":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", ...' '123412..'
```

The credentials will be stored in your config directory as `sippy.json`

## Usage
After logging in, you can order!

First, find the Panera location that you'd like to order from.
Check [here](https://delivery.panera.ca/cafeLocations/),
search for the Panera location you would like, and then look for 
the number after the hashtag (this is the location's ID).
For fellow RPI'ers, the Union's Panera location ID is 203162.

Next, grab the menu with `sippy menu <LOCATION-ID>`. Find the row
with the item you want, and then note the item's ID in the first
column. 

Finally, order the item with `sippy order <LOCATION-ID> <ITEM-ID>`

Enjoy your drink. 


## Notes 
Additional things that users should keep in mind.

* One can also provide a message to the kitchen (such as 'please add cream and sugar') 
or an alternative name for the order. Check `sippy order -h` for information.

* If you order again before the 2 hour cool down, you will get a 422 error.

* You cannot order every item that menu provides. This is because the cli only
works for items that the sip club discount applies to.

* Additionally, the cli cannot place an order that costs money, as it is
hard codded to A) not record payment methods and B) not provide them at
checkout.

## Installation
Clone this repository and from the directory run `cargo install --path=.`
