use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use chrono::{Datelike, Local};
use colored::Colorize;

use serde_json::json;

#[allow(unused_macros)]
macro_rules! read {
    ($out:ident as $type:ty) => {
        let mut inner = String::new();
        std::io::stdin().read_line(&mut inner).expect("A String");
        let $out = inner
            .trim()
            .parse::<$type>()
            .expect("could not parsable the input");
    };
}

#[allow(unused_macros)]
macro_rules! read_str {
    ($out:ident) => {
        let mut inner = String::new();
        std::io::stdin().read_line(&mut inner).expect("A String");
        let $out = inner.trim();
    };
}

#[allow(unused_macros)]
macro_rules! read_vec {
    ($out:ident as $type:ty) => {
        let mut inner = String::new();
        std::io::stdin().read_line(&mut inner).unwrap();
        let $out = inner
            .trim()
            .split_whitespace()
            .map(|s| s.parse::<$type>().unwrap())
            .collect::<Vec<$type>>();
    };
}

struct UserData {
    work_done: String,
    streak: i32,
    timestamp: i32,
    others: Vec<String>,
}

impl UserData {
    fn new(work_done: String, streak: i32, timestamp: i32, others: Vec<String>) -> UserData {
        UserData {
            work_done,
            streak,
            timestamp,
            others,
        }
    }
    fn from(value: &str) -> UserData {
        let value: Vec<&str> = value.split("~").collect();
        let work_done = value[0].to_string();
        let streak = value[1].parse().unwrap();
        let timestamp = value[2].parse().unwrap();
        if value.len() > 3 {
            let others = value[3].split(" ");
            let mut user_data = UserData::new(work_done, streak, timestamp, Vec::new());
            for other in others {
                if other != "" {
                    user_data.others.push(other.to_string());
                }
            }
            user_data
        } else {
            UserData::new(work_done, streak, timestamp, Vec::new())
        }
    }

    fn in_transfer_format(&self) -> String {
        let mut others_str = String::new();
        for person in self.others.iter() {
            if person.len() != 0 {
                //get_values_and_print_them(username, person, &client).await;
                others_str.push_str(format!("{} ", person).as_str());
                //println!("{} in for loop", others_str);
            }
        }
        format!(
            "{}~{}~{}~{}",
            self.work_done, self.streak, self.timestamp, others_str
        )
    }
    fn add_to_others_from_string(&mut self, others: &str) {
        let others = others.trim().split(" ");
        for person in others {
            if person != "" {
                self.others.push(person.to_string());
            }
        }
    }
}

const USERNAME: &str = "Tuhil";
const AIO_KEY: &str = "aio_Vnba62v0VRwvEmx1LckJH9niwEsl";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get the value of feed, which is the name of the user
    println!("{}", "Enter username:".yellow());

    read_str!(feed);
    let feed = feed.trim().to_ascii_lowercase();
    //println!("{}", feed);
    let feed = feed.as_str();

    let client = make_client(AIO_KEY);

    let valid_user = get_values_and_print_them(feed, &client).await;
    println!("\n");
    if !valid_user {
        create_feed_and_send_initial_data(&client, feed).await;
        println!("{}", "created a feed!".magenta());
    }

    loop {
        // get the choice
        println!(
            "{}",
            "Enter choice:\n1) See others' data\n2) Update your data\n3) View your data\n4) exit"
                .yellow()
        );
        read!(choice as i32);
        println!("");
        //let choice: i32 = choice.trim().parse().expect("Please enter a number!");

        if choice == 1 {
            let value: String = get_value_from_feed(feed, &client)
                .await
                .expect("Could not get value!!!");
            //println!("{}", value);
            let mut user_data = UserData::from(&value);
            if user_data.others.len() == 0 {
                println!("{}", "Enter the username of the person:".yellow());
                read!(other_username as String);
                let other_username = other_username.trim();
                get_values_and_print_them(other_username, &client).await;
                user_data.others.push(other_username.to_string());

                send_formatted_data(&user_data, &client, feed).await;
            } else {
                for other_username in user_data.others.iter() {
                    get_values_and_print_them(other_username, &client).await;
                }
                println!(
                    "{}",
                    "Would you like to add some more friends?(y/n)".yellow()
                );
                read!(choice2 as String);
                if choice2.contains("y") {
                    println!(
                        "{}",
                        "Enter the username(s) separated with spaces:".yellow()
                    );
                    read_str!(ppl_to_add);
                    user_data.add_to_others_from_string(ppl_to_add);
                    send_formatted_data(&user_data, &client, feed).await;
                } else {
                    println!("");
                }
            }
        } else if choice == 2 {
            println!("{}", "Enter a brief description of your work:".yellow());
            read!(data as String);
            let data = data.trim();

            let value: String = get_value_from_feed(feed, &client)
                .await
                .expect("Could not get value!!!");
            let mut user_data = UserData::from(value.as_str());
            user_data.work_done = data.to_string();

            let now = Local::now();
            let timestamp_now: i32 = format!("{}{:02}{:02}", now.year(), now.month(), now.day())
                .parse()
                .expect("Error parsing date");
            if are_consecutive(user_data.timestamp, timestamp_now) {
                user_data.streak += 1;
            } else if timestamp_now - user_data.timestamp > 1 {
                user_data.streak = 0;
            }

            user_data.timestamp = timestamp_now;

            send_formatted_data(&user_data, &client, feed).await;
        } else if choice == 3 {
            get_values_and_print_them(feed, &client).await;
        } else if choice == 4 {
            break;
        } else {
            println!("There are literally only 4 otions how did you mess this up..")
        }
        println!("\n");
    }
    Ok(())
}

async fn get_values_and_print_them(feed: &str, client: &Client) -> bool {
    println!("");
    let res = match get_value_from_feed(feed, &client).await {
        Ok(value) => {
            let value: Vec<&str> = value.split("~").collect();

            println!("{}", format!("Name: {}", feed).cyan());
            println!("{}", format!("Streak: {}", value[1]).cyan());
            println!("{}", format!("Work done: {}", value[0]).cyan());
            println!("{}", format!("Timestamp: {}", value[2]).cyan());
            true
        }
        Err(_) => {
            println!("{}", format!("Username {} not found.", feed).red());
            false
        }
    };
    println!("");
    res
}

fn make_client(key: &str) -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-AIO-Key",
        HeaderValue::from_str(key).expect("could not make header"),
    );
    let client = reqwest::Client::builder().default_headers(headers).build();
    client.expect("Failed to connect")
}

async fn get_value_from_feed(feed: &str, client: &Client) -> Result<String, String> {
    let url = format!(
        "https://io.adafruit.com/api/v2/{}/feeds/{}/data/last",
        USERNAME, feed
    );
    let res = client.get(&url).send().await.expect("Couldn't get");

    if res.status().is_success() {
        let text = res.text().await.expect("Couldn't get value");
        let parts: Vec<&str> = text.split(",").collect();
        for part in parts.iter() {
            let parts2: Vec<&str> = part.split(":").collect();
            //println!("{}", part);
            if parts2[0] == "\"value\"" {
                let out = parts2[1]
                    .strip_prefix("\"")
                    .expect("No prefix")
                    .strip_suffix("\"")
                    .expect("No suffix");
                return Ok(out.to_string());
            }
        }
        Err(String::from("dfjslfsl"))
    } else {
        //println!("Failed to retrieve data: {:?}", res);
        Err(String::new() + "sjfl")
    }
}

async fn send_formatted_data(user_data: &UserData, client: &Client, feed: &str) {
    let data_to_send = user_data.in_transfer_format();
    send_data(client, &data_to_send, feed).await.unwrap();
}

async fn send_data(
    client: &Client,
    data: &str,
    feed_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({ "value": data });

    client
        .post(&format!(
            "https://io.adafruit.com/api/v2/{}/feeds/{}/data",
            USERNAME, feed_key
        ))
        .json(&data)
        .send()
        .await?;

    //println!("{}", res.text().await?);

    Ok(())
}

async fn create_feed_and_send_initial_data(client: &Client, feed: &str) {
    create_feed(client, feed)
        .await
        .expect("Couldn't create new feed!!!!!!!!!!");
    let now = Local::now();
    send_data(
        client,
        format!(
            "created account!~0~{}{:02}{:02}",
            now.year(),
            now.month(),
            now.day()
        )
        .as_str(),
        feed,
    )
    .await
    .expect("error sending initial data");
}

async fn create_feed(client: &Client, feed_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({ "feed": {"name":feed_key}});

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );

    client
        .post(&format!(
            "https://io.adafruit.com/api/v2/{}/feeds",
            USERNAME
        ))
        .json(&data)
        .headers(headers)
        .send()
        .await
        .expect("Post request failed!!!");

    //println!("{}", res.text().await?);

    Ok(())
}

fn are_consecutive(timestamp1: i32, timestamp2: i32) -> bool {
    let day1 = timestamp1 % 100;
    let month1 = timestamp1 % 10000 / 100;
    let year1 = timestamp1 / 10000;

    let day2 = timestamp2 % 100;
    let month2 = timestamp2 % 10000 / 100;
    let year2 = timestamp2 / 10000;

    if year2 - year1 == 0 {
        if month2 - month1 == 0 {
            return day2 - day1 == 1;
        } else if month2 - month1 == 1 && day2 == 1 {
            if day1 == number_of_days(month1, year1) {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else if year2 - year1 == 1
        && month2 == 1
        && month1 == 12
        && day2 == 1
        && day1 == number_of_days(12, year1)
    {
        return true;
    } else {
        return false;
    }
}

fn number_of_days(month: i32, year: i32) -> i32 {
    match month {
        1 => 31,
        2 => {
            if year % 4 == 0 {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 0,
    }
}
