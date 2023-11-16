const ds = require("../src/dataStore");

describe('dataStore tests', () => {
    
    beforeEach(() => {
        dsSut = new ds.dataStore;
    })

    test('empty dataStore should return 0', () => {
        expect(dsSut.getAmount('whoever')).toBe(0);
    })

    test('limit on empty dataStore should be default', () => {
        expect(dsSut.getLimit('whoever')).toBe(ds.defaultLimit);
    })
})