const ds = require("../src/dataStore");

describe('dataStore tests', () => {
    
    beforeEach(() => {
        dsSut = new ds.dataStore;
        dsSut.start('whoever', 12345);
    })

    test('empty dataon user should return 0', () => {
        expect(dsSut.getAmount('whoever')).toBe(0);
    })

    test('limit on unset limit should be default', () => {
        expect(dsSut.getLimit('whoever')).toBe(ds.defaultLimit);
    })

    test('limit changed for user should return changed value', () => {
        dsSut.setLimit('whoever', 312);
        expect(dsSut.getLimit('whoever')).toBe(312);
    })

    test('add amount and surpass limit', () => {
        expect(dsSut.addAmount('whoever', ds.defaultLimit+1)).toBeLessThan(0);
    })

    test('normal addition returns added value', () => {
        expect(dsSut.addAmount('whoever', 3)).toBe(3);
        expect(dsSut.addAmount('whoever', 3)).toBe(6);
        expect(dsSut.addAmount('whoever', 100)).toBe(106);
    })

})