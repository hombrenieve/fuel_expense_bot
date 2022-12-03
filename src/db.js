const config = require("./config.js");
const mariadb = require('mariadb');
require('log-timestamp');

function fmtMonth(date) {
    var month = '' + (date.getMonth() + 1),
        year = date.getFullYear();

    if (month.length < 2) 
        month = '0' + month;

    return [year, month, '00'].join('-');
}

function nextMonth(d) {
    if(!d) {
        d = new Date();
    }
    var nmd = new Date(d);
    return new Date(nmd.setMonth(nmd.getMonth() + 1));
}

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
        return this.conn.query("INSERT INTO config(username, chatId) VALUES (?, ?)", [user, id]);
    }
    
    async getAmount(user, date) {
        const curM = fmtMonth(date), nextM = fmtMonth(nextMonth(date));
        return this.conn.query(
            "SELECT config.payLimit AS payLimit, sum(counts.quantity) AS monthlyTotal \
            FROM config,counts \
            WHERE config.username = ?\
                AND counts.username = ?\
                AND counts.txDate > ?\
                AND counts.txDate < ?",
        [user, user, curM, nextM]);
    }

    async getLimit(user) {
        const rows = await this.conn.query("SELECT payLimit FROM config WHERE username = ?", [user]);
        return rows[0]['payLimit'];
    }

    setLimit(user, newLimit) {
        return this.conn.query("UPDATE config SET payLimit = ? WHERE username = ?", [newLimit, user]);
    }

    async addAmount(user, date, amount) { //TODO: Needs transaction
        const [res,] = await this.getAmount(user, date);
        if (res.monthlyTotal + amount > res.payLimit) {
            return -1;
        }
        await this.conn.query("INSERT INTO counts(txDate, username, quantity) VALUES (?, ?, ?)", [date, user, amount]);
        return res.monthlyTotal + amount;
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