# Fuel expense bot

A simple telegram bot to reord the expenses in fuel per month and know the information of what is left

## Getting started

1. Install pre-requisites:
    * node
    * npm
    * A working [mariadb](https://mariadb.org/) database
2. Create tables in database with [scripts/initdb.sql](scripts/initdb.sql)
3. Issue command `npm install` in the root of this repo
4. Configure the personal information. For that you need to create a file `src/config.js`. Recommended content:
```js
const config = {
    db: {
        host: "<host_with_mariadb>",
        user: "<database_user>",
        password: "<database_password>",
        database: "<database_name>"
    },
    api: {
        //TeleBot configuration as in the call to new TeleBot(...)
    },
    app: {
        fuelExpenseLimit: 100.00 //The limit to apply
    }
};
module.exports = config;
```
5. Launch the app with `node src/bot.js`