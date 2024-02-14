use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use chrono::{Datelike, Local};
use colored::Colorize;
use std::io;

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aio_key = "aio_Vnba62v0VRwvEmx1LckJH9niwEsl";
    let username = "Tuhil";

    // get the value of feed, which is the name of the user
    println!("{}", "Enter username:".yellow());
    let mut feed: String = String::new();
    io::stdin()
        .read_line(&mut feed)
        .expect("couldn't read the username!!");
    let feed = feed.trim().to_ascii_lowercase();
    //println!("{}", feed);
    let feed = feed.as_str();

    let client = make_client(aio_key);

    let valid_user = get_values_and_print_them(username, feed, &client).await;
    println!("\n");
    if !valid_user {
        create_feed(&client, username, feed)
            .await
            .expect("Couldn't create new feed!!!!!!!!!!");
        let now = Local::now();
        send_data(
            &client,
            format!(
                "created account!~0~{}{:02}{:02}",
                now.year(),
                now.month(),
                now.day()
            )
            .as_str(),
            username,
            feed,
        )
        .await
        .expect("error 24513");
        println!("{}", "created a feed!".magenta());
    }

    loop {
        // get the choice
        println!(
            "{}",
            "Enter choice:\n1) See others' data\n2) Update your data\n3) View your data\n4) exit"
                .yellow()
        );
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("couldn't read the choice!!");
        println!("");
        let choice: i32 = choice.trim().parse().expect("Please enter a number!");

        if choice == 1 {
            let value: String = get_value_from_feed(username, feed, &client)
                .await
                .expect("Could not get value!!!");
            //println!("{}", value);
            let value: Vec<&str> = value.split("~").collect();
            let mut others = "";
            if value.len() > 3 {
                others = value[3];
            }
            if others == "" {
                println!("{}", "Enter the username of the person:".yellow());
                let mut other_username = String::new();
                io::stdin()
                    .read_line(&mut other_username)
                    .expect("couldn't read the choice!!");
                let other_username = other_username.trim();
                get_values_and_print_them(username, other_username, &client).await;

                let data_to_send =
                    format!("{}~{}~{}~{}", value[0], value[1], value[2], other_username);
                println!("{}", data_to_send);
                send_data(&client, &data_to_send, username, feed)
                    .await
                    .expect("coulfndsjf");
            } else {
                let mut others: Vec<&str> = others.split(" ").collect();
                for other_username in others.iter() {
                    if other_username.len() != 0 {
                        get_values_and_print_them(username, other_username, &client).await;
                    }
                }
                let mut choice2 = String::new();
                println!(
                    "{}",
                    "Would you like to add some more friends?(y/n)".yellow()
                );
                io::stdin()
                    .read_line(&mut choice2)
                    .expect("couldn't read the choice!!");
                match choice2.as_str().trim() {
                    "y" => {
                        let mut ppl_to_add = String::new();
                        println!(
                            "{}",
                            "Enter the username(s) separated with spaces:".yellow()
                        );
                        io::stdin()
                            .read_line(&mut ppl_to_add)
                            .expect("couldn't read the choice!!");
                        let mut ppl_to_add: Vec<&str> = ppl_to_add.trim().split(" ").collect();
                        others.append(&mut ppl_to_add);

                        let mut others_str = String::new();
                        for person in others.iter() {
                            if person.len() != 0 {
                                //get_values_and_print_them(username, person, &client).await;
                                others_str.push_str(format!("{} ", person).as_str());
                                //println!("{} in for loop", others_str);
                            }
                        }
                        //println!("{} in for loop", others_str.as_str());
                        let data_to_send = format!(
                            "{}~{}~{}~{}",
                            value[0],
                            value[1],
                            value[2],
                            others_str.as_str()
                        );
                        send_data(&client, &data_to_send, username, feed)
                            .await
                            .expect("coulfndsjf");
                    }
                    _ => println!(""),
                };
            }
        } else if choice == 2 {
            println!("{}", "Enter a brief description of your work:".yellow());
            let mut data = String::new();
            io::stdin()
                .read_line(&mut data)
                .expect("Couldn't read data!!");
            let data = data.trim();

            let value: String = get_value_from_feed(username, feed, &client)
                .await
                .expect("Could not get value!!!");
            let value: Vec<&str> = value.split("~").collect();

            let now = Local::now();
            let prev_timestamp: i32 = value[2].parse().expect("Error parsing prev_date");
            let timestamp: i32 = format!("{}{:02}{:02}", now.year(), now.month(), now.day())
                .parse()
                .expect("Error parsing date");
            let mut streak: i32 = value[1].parse().expect("Couldn't parse streak");
            if are_consecutive(prev_timestamp, timestamp) {
                streak += 1;
            } else if timestamp - prev_timestamp > 1 {
                streak = 0;
            }

            let mut others = "";
            if value.len() > 3 {
                others = value[3];
            }

            let data_to_send = format!("{}~{}~{}~{}", data, streak, timestamp, others);

            send_data(&client, &data_to_send, username, feed)
                .await
                .expect("Couldn't send the data to feed!!!!!!!");
        } else if choice == 3 {
            get_values_and_print_them(username, feed, &client).await;
        } else if choice == 4 {
            break;
        }
        println!("\n");
    }
    Ok(())
}

async fn get_values_and_print_them(username: &str, feed: &str, client: &Client) -> bool {
    println!("");
    let res = match get_value_from_feed(username, feed, &client).await {
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

async fn get_value_from_feed(
    username: &str,
    feed: &str,
    client: &Client,
) -> Result<String, String> {
    let url = format!(
        "https://io.adafruit.com/api/v2/{}/feeds/{}/data/last",
        username, feed
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

async fn send_data(
    client: &Client,
    data: &str,
    username: &str,
    feed_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({ "value": data });

    client
        .post(&format!(
            "https://io.adafruit.com/api/v2/{}/feeds/{}/data",
            username, feed_key
        ))
        .json(&data)
        .send()
        .await?;

    //println!("{}", res.text().await?);

    Ok(())
}

async fn create_feed(
    client: &Client,
    username: &str,
    feed_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = json!({ "feed": {"name":feed_key}});

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").expect("could not make header"),
    );

    client
        .post(&format!(
            "https://io.adafruit.com/api/v2/{}/feeds",
            username
        ))
        .json(&data)
        .headers(headers)
        .send()
        .await
        .expect("couldnst srgo");

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
