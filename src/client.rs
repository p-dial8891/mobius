// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
use std::ptr::null_mut;
use std::thread;
use tokio::sync::mpsc::{self, Receiver, Sender};
use windows::{
    Win32::Foundation::{BOOL, LPARAM, LRESULT, WPARAM},
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageA, HHOOK, MSG, PEEK_MESSAGE_REMOVE_TYPE, PeekMessageA,
    },
    Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, SetWindowsHookExA, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
    },
    core::PCSTR,
};

static mut user_key: u32 = 0;
static mut HOOK_HANDLE: HHOOK = HHOOK(0);

// Callback function for the hook
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 && w_param.0 as u32 == WM_KEYDOWN {
        let kb_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let key_code = kb_struct.vkCode;

        // Print the key code or check for specific keys
        println!("Key Pressed: {}", key_code);
        // For example, check if the 'ESC' key (key code 27) was pressed
        if key_code == 119 || key_code == 121 {
            user_key = key_code;
        } else if key_code == 27 {
            println!("Escape key pressed! Exiting...");
            // Exit the hook
            UnhookWindowsHookEx(HOOK_HANDLE as HHOOK);
            //std::process::exit(0);
        }
    }

    CallNextHookEx(HOOK_HANDLE as HHOOK, code, w_param, l_param)
}

async fn capture_key(tx: mpsc::Sender<u32>) {
    unsafe {
        // Get the handle to the current module (required for the hook)
        let h_instance = GetModuleHandleA(PCSTR(null_mut())).unwrap();

        // Set the low-level keyboard hook
        HOOK_HANDLE =
            SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook_proc), h_instance, 0).unwrap();
        if HOOK_HANDLE == HHOOK(0) {
            eprintln!("Failed to set hook!");
            return;
        }

        println!("Hook set. Press 'ESC' to exit...");

        // Keep the program running to listen for key presses
        let mut msg = MSG::default();
        while !<BOOL as Into<bool>>::into(
            (PeekMessageA(&mut msg, None, 0, 0, PEEK_MESSAGE_REMOVE_TYPE(0))),
        ) {
            if user_key != 0 {
                let key = user_key;
                if let Err(_) = tx.send(user_key).await {
                    println!("Receiver dropped.");
                }
                println!("Key {key} received.");
                user_key = 0;
            }
        }
        println!("End");
    }
}

use clap::Parser;
use service::{MobiusClient, init_tracing};
use std::fmt;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::{net::SocketAddr, time::Duration};
use tarpc::{client, context, tokio_serde::formats::Json};
use tokio::time::sleep;
use tracing::Instrument;

#[derive(Parser)]
struct Flags {
    /// Sets the server address to connect to.
    #[arg(long)]
    server_addr: Option<SocketAddrV4>,
    /// Sets the name to say hello to.
    #[arg(long)]
    name: Option<String>,
    /// Sets the secret key to send
    #[arg(long)]
    secret: String,
    /// Sets the id to lookup
    #[arg(long)]
    id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel(1);
    let flags = Flags::parse();
    init_tracing("Tarpc Example Client")?;
    println!("Tarpc Example client");

    let mut transport = tarpc::serde_transport::tcp::connect(
        flags
            .server_addr
            .unwrap_or(SocketAddrV4::new(Ipv4Addr::new(169, 254, 24, 24), 50051)),
        Json::default,
    );
    transport.config_mut().max_frame_length(usize::MAX);

    let client = MobiusClient::new(client::Config::default(), transport.await?).spawn();
    let winKeyT = tokio::spawn(capture_key(tx));

    // Let the background span processor finish.
    while let Some(cmd) = rx.recv().await {
        println!("Command Received {cmd}");
        match cmd {
            119 => {
                let uname = client
                    .username(context::current(), flags.secret.clone(), flags.id.clone())
                    .await?;
                println!("{uname}");
            }
            121 => {
                let pwd = client
                    .password(context::current(), flags.secret.clone(), flags.id.clone())
                    .await?;
                println!("{pwd}");
            }
            _ => {}
        }
    }
    sleep(Duration::from_millis(10)).await;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
