const config = require("./config.js");
const mariadb = require('mariadb');

class Db {
    constructor() {
        this.loadConnection();    
    }

    loadConnection() {
        mariadb.createConnection(config.db)
            .then(conn => {
                console.log("DB Connection established!");
                this.conn = conn;
                this.checkConnection();
            })
            .catch(err => console.log("Connecting to db:", err));
    }

    checkConnection() {
        if(this.conn) {
            this.conn.ping()
                .then(() => setTimeout(this.checkConnection, 60000))
                .catch(err => {
                    console.log("DB connection lost:", err);
                    this.loadConnection();
                })
        }
    }
    
    async getAmount(user) {
        var num = -1;
        const rows = await this.conn.query("SELECT paid FROM counts WHERE username = ?", [user]);
        if(rows.length == 1) {
            num = rows[0]['paid'];
        }
        return num;
    }

    getLimit(user) {
        return config.app.fuelExpenseLimit;
    }

    async addAmount(user, amount) {
        const current = await this.getAmount(user);
        if(current >= 0) {
            if(current + amount > config.app.fuelExpenseLimit) {
                return -1;
            }
            await this.conn.query("UPDATE counts SET paid = ? WHERE username = ?", [current + amount, user]);
            return current + amount;
        } else {
            await this.conn.query("INSERT INTO counts VALUES (?, ?)", [user, amount]);
            return amount;
        }
    }

    reset(user) {
        return this.conn.query("UPDATE counts SET paid = ? WHERE username = ?", [0, user]);
    }

    close() {
        console.log("Closing DB Connection");
        this.conn.end()
        .then(() => console.log("DB Connection succesfully closed!"))
        .catch(err => console.log("Error closing DB connection:", err));
    }
}

module.exports.Db = Db;