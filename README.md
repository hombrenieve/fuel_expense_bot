# Fuel expense bot

A simple telegram bot to record the expenses in fuel per month and know the information of what is left

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
        pingInterval: 60000 //Interval to ping the db server (milliseconds)
    }
};
module.exports = config;
```
5. Launch the app with `node src/bot.js`

## DB Architecture

![ER Model for DB](http://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/hombrenieve/fuel_expense_bot/main/diagrams/febER.puml)

The database stores each fuel operation as a new entry in the `counts` table. The primary key is the transaction date so two fuel operations in the same date are not allowed (TODO: Allow more than one operation per date).
The resulting value of the getAmount operation will be the sum of all operations within the month (TODO: check different months or even different periods).
