const mqtt = require('mqtt');

class Publisher {

    constructor(config) {
        this.topic = config.topic;
        this.url = config.url;
    }

    start() {
        this.client = mqtt.connect(this.url);
        this.publish(0, 0, 0);
    }

    publish(currentTS, currentLimit, currentValue) {
        this.client.publish(this.topic, JSON.stringify(
            {
                limit: currentLimit,
                timestamp: currentTS,
                value: currentValue,
                left: currentLimit-currentValue
            }
        ));
    }
}

module.exports.Publisher = Publisher;