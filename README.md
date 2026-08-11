# BambuLab-notifier

Listen to BambuLab printers through MQTT and send messages to Telegram at start and end of printing.

## Configuration

Requires a `.env` file to load all the required variables, for example:

```plain
HOST="<bambulab_ip>"
ACCESS_CODE="<bambulab_access_code>"
SERIAL="<bambula_serial_number>"
TELEGRAM_USER="<telegram_user_id_to_send_messages>"
TELEGRAM_BOT_TOKEN="<telegram_bot_token>"
```
