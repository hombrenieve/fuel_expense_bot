const TeleBot = require('telebot');
const ds = require('./dataStore.js');
const config = require('./config.js');

const bot = new TeleBot(config.api);

const data = new ds.DataStore;

bot.on('/start', (msg) => {
    data.start(msg.from.username, msg.chat.id);
    sendData(msg);
});

bot.on('/check', (msg) => {
    sendData(msg);
});

bot.on(/^\d+\.*\d*$/, (msg) => {
    let ret = data.addAmount(msg.from.username, parseFloat(msg.text));
    if (ret == -1) {
        bot.sendMessage(msg.chat.id, "Expense exceeds limit!");
    } else {
        sendData(msg);
    }
});

bot.on(/^\/config (.+)$/, (msg, props) => {
    const propsText = props.match[1].split(' ');
    if(propsText[0] == 'limit') {
        console.log("Configuring limit for "+msg.from.username+" to: "+propsText[1]);
        data.setLimit(msg.from.username, parseFloat(propsText[1]));
        sendData(msg);
    } else {
        console.log("Unknown config: "+ propsText[0]);
    }
});

function round(value, decimals) {
    return Number(Math.round(value +'e'+ decimals) +'e-'+ decimals).toFixed(decimals);
}

function sendData(msg) {
    let rounded = round(data.getAmount(msg.from.username), 2);
    bot.sendMessage(msg.chat.id,
        "Spent: " + rounded.toString() + "\n" +
        "Left: " + round(data.getLimit(msg.from.username) - rounded, 2))
}

process.on('SIGINT', function() {
    console.log("Caught interrupt signal");
    bot.stop();
});

bot.start();