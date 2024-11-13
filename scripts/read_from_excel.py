import pandas as pd
import json
import paho.mqtt.client as mqtt
from datetime import datetime
import time
import os

def read_value_from_file(file_path):
    current_month = datetime.now().month

    row_index = current_month - 1
    column_index = 5
    df = pd.read_excel(file_path, header=None)
    return df.iloc[row_index, column_index]

def on_connect(client, userdata, flags, reason_code, properties):
    print(f"Connected with result code {reason_code}")
    client.publish(userdata[0], userdata[1])
    print("Message published")
    client.disconnect()
    print("Disconnected")

def send_mqtt_message(value):
    limit = 210
    message = {
        "limit":limit,
        "left":limit-value,
        "amount": value
    }
    json_message = json.dumps(message)
    broker_address = "localhost"  # Your MQTT broker address
    mqtt_topic = "car/bmw/fuel_load"  # Your MQTT topic

    mqttc = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    mqttc.on_connect = on_connect
    mqttc.user_data_set([mqtt_topic, json_message])
    mqttc.connect(broker_address)
    print("Connecting...")
    mqttc.loop_forever()
    
def main():
    home_dir = os.path.expanduser("~")
    file_path = home_dir + '/OneDrive/Gasolina.xlsx'
    print(file_path)
    try:
        value = read_value_from_file(file_path)
        print("Read: ", value)
        send_mqtt_message(value)
    except KeyboardInterrupt:
        print("Process interrupted by user")

if __name__ == "__main__":
    main()
