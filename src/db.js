const config = require("./config.js");
const mariadb = require('mariadb');
require('log-timestamp');

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
            .catch(err => {
                console.log("DB Connection error:", err);
                var that = this;
                this.check = setTimeout(function() { that.checkConnection() }, config.app.pingInterval);
            });
    }

    checkConnection() {
        if(this.conn) {
            this.conn.ping()
                .then(() => {
                    var that = this;
                    this.check = setTimeout(function() { that.checkConnection() }, config.app.pingInterval);
                })
                .catch(err => {
                    console.log("DB connection lost:", err);
                    this.loadConnection();
                })
        } else {
            console.log("DB connection not available");
            this.loadConnection();
        }
    }

    start(user, id) {
        return this.conn.query("INSERT INTO counts(username, chatId, paid) VALUES (?, ?, ?)", [user, id, 0]);
    }
    
    async getAmount(user) {
        const rows = await this.conn.query("SELECT paid FROM counts WHERE username = ?", [user]);
        return rows[0]['paid'];
    }

    async getLimit(user) {
        const rows = await this.conn.query("SELECT payLimit FROM counts WHERE username = ?", [user]);
        return rows[0]['payLimit'];
    }

    setLimit(user, newLimit) {
        return this.conn.query("UPDATE counts SET payLimit = ? WHERE username = ?", [newLimit, user]);
    }

    async addAmount(user, amount) {
        const current = await this.getAmount(user);
        if(current + amount > await this.getLimit(user)) {
            return -1;
        }
        await this.conn.query("UPDATE counts SET paid = ? WHERE username = ?", [current + amount, user]);
        return current + amount;
    }

    reset(user) {
        return this.conn.query("UPDATE counts SET paid = ? WHERE username = ?", [0, user]);
    }

    close() {
        console.log("DB connection is closing...");
        clearTimeout(this.check);
        this.conn.end()
        .then(() => console.log("DB Connection succesfully closed!"))
        .catch(err => console.log("DB connection error closing:", err));
    }
}

module.exports.Db = Db;