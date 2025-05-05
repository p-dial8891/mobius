use std::ptr::null_mut;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use windows::{
    core::PCSTR,
    Win32::Foundation::{LPARAM, LRESULT, WPARAM, BOOL},
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::{SetWindowsHookExA, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN},
    Win32::UI::WindowsAndMessaging::{CallNextHookEx, GetMessageA, PeekMessageA, MSG, HHOOK, PEEK_MESSAGE_REMOVE_TYPE},
};

static mut user_key: u32 = 0;
static mut HOOK_HANDLE: HHOOK = HHOOK(0);

// Callback function for the hook
unsafe extern "system" fn keyboard_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 && w_param.0 as u32 == WM_KEYDOWN {
        let kb_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let key_code = kb_struct.vkCode;

        // Print the key code or check for specific keys
        println!("Key Pressed: {}", key_code);

        // For example, check if the 'ESC' key (key code 27) was pressed
		if key_code == 115 || key_code == 116 || key_code == 117 {
			user_key = key_code;
		}
        else if key_code == 27 {
            println!("Escape key pressed! Exiting...");
            // Exit the hook
            UnhookWindowsHookEx(HOOK_HANDLE as HHOOK);
            //std::process::exit(0);
        }
    }

    CallNextHookEx(HOOK_HANDLE as HHOOK, code, w_param, l_param)
}

fn main() {
    unsafe {
        // Get the handle to the current module (required for the hook)
        let h_instance = GetModuleHandleA(PCSTR(null_mut())).unwrap();

        // Set the low-level keyboard hook
        HOOK_HANDLE = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook_proc), h_instance, 0).unwrap();
        if HOOK_HANDLE == HHOOK(0) {
            eprintln!("Failed to set hook!");
            return;
        }

        println!("Hook set. Press 'ESC' to exit...");

        // Keep the program running to listen for key presses
        let mut msg = MSG::default();
		while !<BOOL as Into<bool>>::into((PeekMessageA(&mut msg, None, 0, 0, 
		PEEK_MESSAGE_REMOVE_TYPE(0))))  {
			if user_key != 0 {
				let key = user_key;
				println!("Key {key} received.");
				user_key = 0;
			}
		}
		println!("End");
    }

}
