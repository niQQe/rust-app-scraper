use dotenv::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use redis::{Commands, RedisError};
use std::collections::HashMap;
use std::env;
mod getter;

fn db() -> redis::Connection {
    let redis_addr = env::var("REDIS_ADDR").unwrap();
    redis::Client::open(
        redis_addr)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

fn main() {
    let (changes_on, available) = get_available_apps();
    if available {
        println!("changes found");
        send_email(changes_on)
    } else {
        println!("No changes found");
    }
}

fn get_available_apps() -> (String, bool) {
    let mut db = db();

    let finfast = get_finfast_apps();
    let lundbergs = get_lundbergs_apps();

    let scraped_data = HashMap::from([
        ("finfast".to_string(), finfast.to_string()),
        ("lundbergs".to_string(), lundbergs.to_string()),
    ]);

    /* Get keys from database */

    let mut changes_on = String::from("");
    let mut new_changes_found = false;

    for (key, value) in &scraped_data {
        let db_data: String = db.get(key).unwrap();
        /* Set newly scraped data to database if scraped hashmap values are not the same as db hashmap values */
        if !db_data.eq(&value.to_string()) {
            new_changes_found = true;
            changes_on.push_str(key);
            changes_on.push(' ');
            let set_response: Result<(), RedisError> = db.set(&key, &value.to_string());
            match set_response {
                Ok(_) => println!("Set complete"),
                Err(e) => println!("Set error {}", e),
            }
        }
    }
    (changes_on, new_changes_found)
}

fn get_lundbergs_apps() -> String {
    let getter = getter::Getter::new(
        format!("https://www.lundbergsfastigheter.se/bostad/lediga-lagenheter/orebro"),
        format!(".closed"),
    )
    .expect("Fetching faileds");

    let apps = getter.get_apps();

    let mut db_value = String::from("");

    apps.iter().for_each(|app_string| {
        if app_string.contains("2 rum och kök") {
            db_value.push_str(app_string);
        }
    });

    db_value
}

fn get_finfast_apps() -> String {
    let getter = getter::Getter::new(
        format!("https://finfast.se/lediga-objekt"),
        format!(".title a strong"),
    )
    .expect("Fetching failed");

    let apps = getter.get_apps();

    let mut db_value = String::from("");

    apps.iter().for_each(|app_string| {
        let available = app_string.split(" ").collect::<Vec<&str>>()[0];
        if available.parse::<i32>().is_ok() {
            if available.parse::<i32>().unwrap() > 3 {
                db_value.push_str(app_string);
            }
        } else {
            if available.replace(",", ".").parse::<f32>().unwrap() > 2.0 {
                db_value.push_str(app_string);
            }
        }
    });

    db_value
}

fn send_email(changes: String) {
    dotenv().ok();
    let email_username = env::var("EMAIL_USERNAME").unwrap();
    let email_password = env::var("EMAIL_PASSWORD").unwrap();

    let mut body = "Det finns nya lägenheter på: ".to_string();
    body += &changes;
    let email = Message::builder()
        .from(email_username.parse().unwrap())
        .reply_to(email_username.parse().unwrap())
        .to(email_username.parse().unwrap())
        .subject("Nya lägenheter!")
        .body(body.to_string())
        .unwrap();

    let creds = Credentials::new(email_username, email_password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
