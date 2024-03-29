const TeleBot = require('telebot');
const Db = require('./db.js');
const config = require('./config.js');

const bot = new TeleBot(config.api);

const data = new Db.Db();

bot.on('/start', (msg) => {
    data.start(msg.from.username, msg.chat.id)
    .then(() => sendData(msg))
    .catch(err => console.log("Error starting", err));
});

bot.on('/check', (msg) => {
    sendData(msg);
});

bot.on(/^\d+\.*\d*$/, (msg) => {
    data.addAmount(msg.from.username, new Date(), parseFloat(msg.text))
        .then(added => {
            if (added == -1) {
                bot.sendMessage(msg.chat.id, "Expense exceeds limit!");
            }
            sendData(msg);
        })
        .catch(err => console.log("Error adding amount", err));
});

bot.on(/^\/config (.+)$/, (msg, props) => {
    const propsText = props.match[1].split(' ');
    if(propsText[0] == 'limit') {
        console.log("Configuring limit for "+msg.from.username+" to: "+propsText[1]);
        data.setLimit(msg.from.username, parseFloat(propsText[1]))
            .then(() => sendData(msg))
            .catch(err => console.log("Error configuring limit for "+msg.from.username+" "+err));
    } else {
        console.log("Unknown config: "+ propsText[0]);
    }
});

function round(value, decimals) {
    return Number(Math.round(value +'e'+ decimals) +'e-'+ decimals).toFixed(decimals);
}

function sendData(msg) {
    data.getAmount(msg.from.username, new Date())
        .then(res => {
            var rounded = round(res[0].monthlyTotal, 2);
            bot.sendMessage(msg.chat.id,
                "Spent: " + rounded.toString() + "\n" +
                "Left: " + round(res[0].payLimit - res[0].monthlyTotal, 2))
        })
        .catch(err => console.log("Error getting amount", err));
}

process.on('SIGINT', function() {
    console.log("Caught interrupt signal");

    data.close();
    bot.stop(); //Seems it takes enough time for the DB to close
});

bot.start();