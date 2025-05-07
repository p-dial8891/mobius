// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use clap::Parser;
use futures::{future, prelude::*};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use service::{Mobius, init_tracing};
use std::io::Error;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tarpc::{
    context,
    server::{self, Channel, incoming::Incoming},
    tokio_serde::formats::Json,
};
use tokio::time;

use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use simple_crypt::decrypt;
use std::path::Path;

use serde_json::{Result, Value};
use std::error::Error as OtherError;
use std::io::BufReader;

fn get_key_codes(c: char) -> (bool, u8) {
    let capital = c.is_ascii_uppercase();
    match c.to_ascii_lowercase() {
        'a' => (capital, 4),

        'b' => (capital, 5),

        'c' => (capital, 6),

        'd' => (capital, 7),

        'e' => (capital, 8),

        'f' => (capital, 9),

        'g' => (capital, 10),

        'h' => (capital, 11),

        'i' => (capital, 12),

        'j' => (capital, 13),

        'k' => (capital, 14),

        'l' => (capital, 15),

        'm' => (capital, 16),

        'n' => (capital, 17),

        'o' => (capital, 18),

        'p' => (capital, 19),

        'q' => (capital, 20),

        'r' => (capital, 21),

        's' => (capital, 22),

        't' => (capital, 23),

        'u' => (capital, 24),

        'v' => (capital, 25),

        'w' => (capital, 26),

        'x' => (capital, 27),

        'y' => (capital, 28),

        'z' => (capital, 29),

        '0' => (capital, 30),

        ')' => (true, 39),

        '1' => (false, 30),

        '!' => (true, 30),

        '2' => (false, 31),

        '@' => (true, 31),

        '3' => (false, 32),

        '#' => (true, 32),

        '4' => (false, 33),

        '$' => (true, 33),

        '5' => (false, 34),

        '%' => (true, 34),

        '6' => (false, 35),

        '^' => (true, 35),

        '7' => (false, 36),

        '&' => (true, 36),

        '8' => (false, 37),

        '*' => (true, 37),

        '9' => (false, 38),

        '(' => (true, 38),

        '-' => (false, 45),

        '_' => (true, 45),

        '=' => (false, 46),

        '+' => (true, 46),

        '[' => (false, 47),

        '{' => (true, 47),

        ']' => (false, 48),

        '}' => (true, 48),

        '\'' => (false, 49),

        '|' => (true, 49),

        ';' => (false, 51),

        ':' => (true, 51),

        '\'' => (false, 52),

        '"' => (true, 52),

        '`' => (false, 53),

        '~' => (true, 53),

        ',' => (false, 54),

        '<' => (true, 54),

        '.' => (false, 55),

        '>' => (true, 55),

        '/' => (false, 56),

        '?' => (true, 56),

        _ => (true, 56),
    }
}

#[derive(Parser)]
struct Flags {
    /// Sets the secret key to use
    #[clap(long)]
    secret: String,
    /// Sets the port number to listen on.
    #[clap(long)]
    port: u16,
}

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
struct MobiusServer(SocketAddr, Value, String);

impl Mobius for MobiusServer {
    async fn username(self, _: context::Context, secret: String, id: String) -> String {
        let sleep_time =
            Duration::from_millis(Uniform::new_inclusive(1, 10).sample(&mut thread_rng()));
        time::sleep(sleep_time).await;
        let mut gadgetFIle = File::create("/dev/hidg0").await.unwrap();
        if secret == self.2 {
            for c in self.1[id][0].as_str().expect("Unknown JSON id").chars() {
                let cx = get_key_codes(c);
                gadgetFIle
                    .write_all(vec![if cx.0 { 2 } else { 0 }, 0, 0, cx.1, 0, 0, 0, 0].as_slice())
                    .await;
                gadgetFIle
                    .write_all(vec![0, 0, 0, 0, 0, 0, 0, 0].as_slice())
                    .await;
            }
        } else {
            tracing::info!("Wrong secret key.");
        }
        String::new()
    }
    async fn password(self, _: context::Context, secret: String, id: String) -> String {
        let sleep_time =
            Duration::from_millis(Uniform::new_inclusive(1, 10).sample(&mut thread_rng()));
        time::sleep(sleep_time).await;
        let mut gadgetFIle = File::create("/dev/hidg0").await.unwrap();
        if secret == self.2 {
            for c in self.1[id][2].as_str().expect("Unknown JSON id").chars() {
                let cx = get_key_codes(c);
                gadgetFIle
                    .write_all(vec![if cx.0 { 2 } else { 0 }, 0, 0, cx.1, 0, 0, 0, 0].as_slice())
                    .await;
                gadgetFIle
                    .write_all(vec![0, 0, 0, 0, 0, 0, 0, 0].as_slice())
                    .await;
            }
        } else {
            tracing::info!("Wrong secret key.");
        }
        String::new()
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let flags = Flags::parse();
    init_tracing("Tarpc Example Server")?;
    println!("Tarpc Example Server");

    // Open the file in read-only mode with buffer.
    let contents = fs::read_to_string("input.json")
        .await
        .expect("Unable to open JSON file.");
    //let contents = decrypt(&encrypted, flags.secret.as_bytes())
    //.expect("Failed to decrypt the file");
    // Read the JSON contents of the file as an instance of `User`.
    let server_addr = (IpAddr::V4(Ipv4Addr::new(169, 254, 24, 24)), flags.port);

    // JSON transport is provided by the json_transport tarpc module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    println!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let v: Value = serde_json::from_str(contents.as_str()).expect("Unable to parse JSON");
            let server = MobiusServer(
                channel.transport().peer_addr().unwrap(),
                v.clone(),
                flags.secret.clone(),
            );
            channel.execute(server.serve()).for_each(spawn)
            //fs::remove_file("input.json")
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
