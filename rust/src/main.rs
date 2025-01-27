use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use gpio_cdev::{Chip, LineRequestFlags};

// Define default MIDI hex values
#[derive(Clone)]
struct MidiConfig {
    next: String,
    previous: String,
    custom: String,
}

impl MidiConfig {
    fn new() -> Self {
        Self {
            next: "C0 01".to_string(),
            previous: "C0 00".to_string(),
            custom: "C0 7F".to_string(),
        }
    }
}

// Send a MIDI message (dummy function for this example)
fn send_midi_message(hex: &str) {
    println!("MIDI Message Sent: {}", hex);
}

// GPIO Button Thread
fn gpio_thread(config: Arc<Mutex<MidiConfig>>) {
    let mut chip = Chip::new("/dev/gpiochip0").expect("Failed to open GPIO chip");

    let next_button = chip
        .get_line(17)
        .expect("Failed to get GPIO line 17")
        .request(LineRequestFlags::INPUT, 0, "next_button")
        .expect("Failed to request GPIO line 17");

    let previous_button = chip
        .get_line(27)
        .expect("Failed to get GPIO line 27")
        .request(LineRequestFlags::INPUT, 0, "previous_button")
        .expect("Failed to request GPIO line 27");

    let custom_button = chip
        .get_line(22)
        .expect("Failed to get GPIO line 22")
        .request(LineRequestFlags::INPUT, 0, "custom_button")
        .expect("Failed to request GPIO line 22");

    loop {
        let config = config.lock().unwrap();

        if next_button.get_value().unwrap() == 0 {
            send_midi_message(&config.next);
            thread::sleep(Duration::from_millis(200));
        }

        if previous_button.get_value().unwrap() == 0 {
            send_midi_message(&config.previous);
            thread::sleep(Duration::from_millis(200));
        }

        if custom_button.get_value().unwrap() == 0 {
            send_midi_message(&config.custom);
            thread::sleep(Duration::from_millis(200));
        }

        thread::sleep(Duration::from_millis(50));
    }
}

// HTTP Handlers
async fn index(config: web::Data<Arc<Mutex<MidiConfig>>>) -> impl Responder {
    let config = config.lock().unwrap();
    let html = format!(
        r#"
        <html>
            <head>
                <title>MIDI Button Config</title>
            </head>
            <body>
                <h1>MIDI Button Configuration</h1>
                <form action="/update" method="post">
                    <label for="button">Select Button:</label>
                    <select name="button">
                        <option value="next">Next</option>
                        <option value="previous">Previous</option>
                        <option value="custom">Custom</option>
                    </select>
                    <br><br>
                    <label for="hex_value">Hexadecimal Value:</label>
                    <input type="text" name="hex_value" placeholder="e.g., C0 01" required />
                    <br><br>
                    <button type="submit">Update</button>
                </form>
                <h2>Current Configurations</h2>
                <ul>
                    <li>Next: {}</li>
                    <li>Previous: {}</li>
                    <li>Custom: {}</li>
                </ul>
            </body>
        </html>
        "#,
        config.next, config.previous, config.custom
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn update(
    config: web::Data<Arc<Mutex<MidiConfig>>>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let button = form.get("button").cloned();
    let hex_value = form.get("hex_value").cloned();

    if let (Some(button), Some(hex_value)) = (button, hex_value) {
        let mut config = config.lock().unwrap();
        match button.as_str() {
            "next" => config.next = hex_value,
            "previous" => config.previous = hex_value,
            "custom" => config.custom = hex_value,
            _ => (),
        }
    }

    HttpResponse::Found().header("Location", "/").finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(Mutex::new(MidiConfig::new()));
    let config_clone = Arc::clone(&config);

    // Start GPIO thread
    thread::spawn(move || gpio_thread(config_clone));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&config)))
            .route("/", web::get().to(index))
            .route("/update", web::post().to(update))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
