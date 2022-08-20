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

function fmtDate(date) {
    var month = '' + (date.getMonth() + 1),
        day = '' + date.getDate(),
        year = date.getFullYear();

    if (month.length < 2) 
        month = '0' + month;
    if(day.length < 2)
        day = '0' + day;
    

    return [year, month, day].join('-');
}

function nextMonth(d) {
    if(!d) {
        d = new Date();
    }
    return new Date(d.setMonth(d.getMonth() + 1));
}

function currentMonth() {
    return new Date();
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
    console.log("Data: ", [user, user, fmtMonth(date), fmtMonth(nextMonth(date))]);
        const rows = await this.conn.query("SELECT config.payLimit AS payLimit, sum(counts.quantity) AS monthlyTotal \
                    FROM config,counts \
                    WHERE config.username = ?\
                        AND counts.username = ?\
                        AND txDate > ? \
                        AND txDate < ?",
                    [user, user, fmtMonth(date), fmtMonth(nextMonth(date))]);
    console.log(rows);
        var current = rows[0]['monthlyTotal'];
        if (!current) current = 0;
        return [current, rows[0]['payLimit']];
    }

    async getLimit(user) {
        const rows = await this.conn.query("SELECT payLimit FROM config WHERE username = ?", [user]);
        return rows[0]['payLimit'];
    }

    setLimit(user, newLimit) {
        return this.conn.query("UPDATE config SET payLimit = ? WHERE username = ?", [newLimit, user]);
    }

    async addAmount(user, date, amount) { //TODO: Needs transaction
    console.log("Adding amount ", user, " ", date, " ", amount);
        const [current, limit] = await this.getAmount(user, date);
    console.log("Current ", current, " amount: ", amount)
        if (current + amount > limit) {
            return -1;
        }
        await this.conn.query("INSERT INTO counts VALUES (?, ?, ?)", [fmtDate(date), user, amount]);
        return current + amount;
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