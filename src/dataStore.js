
const defaultLimit = 210;

class dataStore {

    constructor() {
    }

    restart(user, id) {
        this.currentAmount = 0;
    }

    getLimit(user) {
        return this.limit;
    }
    
    setLimit(user, newLimit) {
        this.limit = newLimit;
    }

    getAmount(user) {
        return this.currentAmount;
    }

    //TODO: Make it void and use exception
    addAmount(user, amount) {
        if (this.currentAmount+amount > this.limit) {
            return -1;
        }
        this.currentAmount += amount;
        return this.currentAmount;
    }

}

module.exports.dataStore = dataStore;
module.exports.defaultLimit = defaultLimit;