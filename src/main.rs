use bambulab::{Message, client::Client, command::Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let (tx, mut rx) = tokio::sync::broadcast::channel::<Message>(25);

    let mut client = Client::new(host, access_code, serial, tx);
    let client_clone = client.clone();

    tokio::try_join!(
        tokio::spawn(async move {
            client.run().await.unwrap();
        }),
        tokio::spawn(async move {
            let mut print_started = false;
            let mut filename = None;
            loop {
                let message = rx.recv().await.unwrap();
                // println!("received: {message:?}");

                if message == Message::Connected {
                    client_clone.publish(Command::PushAll).await.unwrap();
                }

                match message {
                    Message::Print(data) => {
                        let f = data.print.gcode_file;
                        let bed_temp = data.print.bed_temper;
                        let percent = data.print.mc_percent;
                        if let Some(val) = percent {
                            if val > 0 && !print_started {
                                println!("print started");
                                print_started = true;
                                filename = f;
                            }
                        }
                        if let Some(val) = bed_temp {
                            if print_started && val < 35.0 {
                                println!("{}º finished print of {:?}", val, filename);
                                print_started = false;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }),
    )?;

    Ok(())
}
