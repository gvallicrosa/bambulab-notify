use bambulab::{Message, client::Client, command::Command};
use dotenvy::dotenv;
use std::env;

const TEMPERATURE_THRESHOLD: f64 = 40.0;

async fn send_telegram(client: &str, token: &str, text: &str) {
    let form = reqwest::multipart::Form::new()
        .text("chat_id", client.to_string())
        .text("text", text.to_string());
    let resp = reqwest::Client::new()
        .post(format!("https://api.telegram.org/bot{}/sendMessage", token))
        .multipart(form)
        .send()
        .await;
    log::info!("response: {:?}", resp);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info) // TODO
        .env()
        .init()
        .unwrap();
    log::info!("starting...");

    // load variables from .env file
    dotenv().expect("missing .env file");
    let host = env::var("HOST").expect("missing HOST in .env");
    let access_code = env::var("ACCESS_CODE").expect("missing ACCESS_CODE in .env");
    let serial = env::var("SERIAL").expect("missing SERIAL in .env");
    let telegram_user = env::var("TELEGRAM_USER").expect("missing TELEGRAM_USER in .env");
    let telegram_bot_token =
        env::var("TELEGRAM_BOT_TOKEN").expect("missing TELEGRAM_BOT_TOKEN in .env");

    // channel to receive messages from MQTT
    let (tx, mut rx) = tokio::sync::broadcast::channel::<Message>(25);

    // MQTT client with channel writer
    let mut client = Client::new(host, access_code, serial, tx);
    let client_clone = client.clone();

    // main loop to react to messages
    tokio::try_join!(
        // client task
        tokio::spawn(async move {
            client.run().await.unwrap();
        }),
        // message parsing task
        tokio::spawn(async move {
            let mut print_started = false;
            let mut filename = None;
            loop {
                let message = rx.recv().await.unwrap();

                // on connect request status
                if message == Message::Connected {
                    log::info!("connected");
                    client_clone.publish(Command::PushAll).await.unwrap();
                }

                // on status data, run the print finish logic
                if let Message::Print(data) = message {
                    // log::debug!("{:?}", data);
                    // log::info!(
                    //     "gcode: {:?}, bed_temp: {:?}, percent: {:?}",
                    //     data.print.gcode_file,
                    //     data.print.bed_temper,
                    //     data.print.mc_percent
                    // );
                    let f = data.print.gcode_file;
                    let bed_temp = data.print.bed_temper;
                    let percent = data.print.mc_percent;

                    // filename update
                    if let Some(file) = f
                        && !file.is_empty()
                    {
                        filename = Some(file);
                    }

                    // print start/running (some percent)
                    if !print_started
                        && let Some(val) = percent
                        && val > 0
                        && val < 100
                    {
                        print_started = true;
                        let msg = format!(
                            "print started {:?}",
                            filename.clone().unwrap_or("UNKNOWN".to_string())
                        );
                        log::info!("{}", msg);
                        send_telegram(&telegram_user, &telegram_bot_token, &msg).await;
                    }

                    // print end (bed temp below threshold)
                    if print_started
                        && let Some(val) = bed_temp
                        && val < TEMPERATURE_THRESHOLD
                    {
                        let msg = format!(
                            "print ended {:.1}ºC for {}",
                            val,
                            filename.clone().unwrap_or("UNKNOWN".to_string())
                        );
                        log::info!("{}", msg);
                        send_telegram(&telegram_user, &telegram_bot_token, &msg).await;
                        print_started = false;
                    }
                }
            }
        }),
    )?;

    Ok(())
}
