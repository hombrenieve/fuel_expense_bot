
const defaultLimit = 210;

class dataStore {

    constructor() {
        this.data = {}
    }

    start(user, id) {
        this.data[user] = {
            currentAmount: 0,
            limit: defaultLimit,
            id: id
        }
    }

    getLimit(user) {
        return this.data[user].limit;
    }
    
    setLimit(user, newLimit) {
        this.data[user].limit = newLimit;
    }

    getAmount(user) {
        return this.data[user].currentAmount;
    }

    //TODO: Make it void and use exception
    addAmount(user, amount) {
        if (this.data[user].currentAmount+amount > this.data[user].limit) {
            return -1;
        }
        this.data[user].currentAmount += amount;
        return this.data[user].currentAmount;
    }

}

module.exports.dataStore = dataStore;
module.exports.defaultLimit = defaultLimit;