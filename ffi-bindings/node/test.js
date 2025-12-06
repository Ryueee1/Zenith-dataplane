const { expect } = require('chai');
const { ZenithClient, ZenithError } = require('./zenith');

describe('ZenithClient', function () {
    this.timeout(5000);

    it('should create a client', function () {
        const client = new ZenithClient(1024);
        expect(client).to.exist;
        expect(client.closed).to.be.false;
        client.close();
    });

    it('should close properly', function () {
        const client = new ZenithClient(512);
        client.close();
        expect(client.closed).to.be.true;

        // Double close should be safe
        client.close();
    });

    it('should reject operations after close', function () {
        const client = new ZenithClient();
        client.close();

        expect(() => client.getStats()).to.throw(ZenithError);
    });

    it('should get stats', function () {
        const client = new ZenithClient();
        const stats = client.getStats();
        expect(stats).to.be.an('object');
        client.close();
    });
});
