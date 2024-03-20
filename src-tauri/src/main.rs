use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::{fs::File, io::Write};
use std::{thread, time::Duration};
use tauri::api::notification::Notification;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MuteDef {
    recur: String,
    start: String,
    end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    notify: bool,
    interval: u64,
    dars_start_date: String,
    dars_end_date: String,
    mute_for: i32,
    mute_def: Vec<MuteDef>,
}

#[tauri::command]
fn get_dars() -> String {
    // serde_json::to_string_pretty(&fetch_dars_data()).expect("Ustadji: error parsing to JSON")
    let dars = {
        let file_path = "/Applications/iustadji-mac.app/Contents/Resources/dars.json";
        let file_content = fs::read_to_string(file_path).expect("Ustadji: error reading file");
        serde_json::from_str::<Vec<Dars>>(&file_content)
            .expect("Ustadji: error serializing to JSON")
    };
    serde_json::to_string_pretty(&dars).expect("Ustadji: error parsing to JSON")
}

#[tauri::command]
fn get_settings() -> Settings {
    let settings = {
        let file_path = "/Applications/iustadji-mac.app/Contents/Resources/settings.json";

        println!("Pathe {}", Path::new(file_path).exists());

        if !Path::new(file_path).exists() {
            // If the file doesn't exist, create it with default settings
            let default_settings = vec![Settings {
                notify: true,
                interval: 1,
                dars_start_date: "01.01.2024".to_string(),
                dars_end_date: "01.01.2025".to_string(),
                mute_for: 30,
                mute_def: vec![
                    MuteDef {
                        recur: "daily".to_string(),
                        start: "10:00".to_string(),
                        end: "11:00".to_string(),
                    },
                    MuteDef {
                        recur: "daily".to_string(),
                        start: "16:00".to_string(),
                        end: "16:30".to_string(),
                    },
                ],
            }];

            // Serialize the default settings to JSON
            let default_settings_json = serde_json::to_string_pretty(&default_settings)
                .expect("Failed to serialize default settings to JSON");
            // Create the settings file
            fs::write(file_path, default_settings_json).expect("Failed to create settings file");
        }

        let file_content = fs::read_to_string(file_path).expect("Ustadji: error reading file");

        serde_json::from_str::<Vec<Settings>>(&file_content)
            .expect("Ustadji: error serializing to JSON")
            .first()
            .unwrap()
            .clone()
    };
    settings
}

// Define the Notification struct
#[derive(Debug, Serialize, Deserialize)]
struct NotificationData {
    id: i32,
    title: String,
    description: String,
}

// Define the Dars struct
#[derive(Debug, Serialize, Deserialize)]
struct Dars {
    date: String,
    notifications: Vec<NotificationData>,
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let open: CustomMenuItem = CustomMenuItem::new("open".to_string(), "Open");
    let unmute: CustomMenuItem = CustomMenuItem::new("unmute".to_string(), "Unmute");
    let mute_30: CustomMenuItem = CustomMenuItem::new("mute_30".to_string(), "Mute for 30 mins");
    let mute_60: CustomMenuItem = CustomMenuItem::new("mute_60".to_string(), "Mute for 1 hr");
    let mute_restart: CustomMenuItem = CustomMenuItem::new("mute_restart".to_string(), "Mute until restart");

    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(unmute)
        .add_item(mute_30)
        .add_item(mute_60)
        .add_item(mute_restart)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_dars, get_settings])
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // #[cfg(not(target_os = "macos"))] {
                //     event.window().hide().unwrap();
                //   }
                #[cfg(target_os = "macos")]
                {
                    tauri::AppHandle::hide(&event.window().app_handle()).unwrap();
                }
                api.prevent_close();
            }
            _ => {}
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "open" => {
                    let window = app.get_window("main").unwrap();
                    window.set_focus().unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while building tauri application");
    let app_config = app.config().tauri.bundle.identifier.clone();

    init_notification(app_config);

    // Run the app
    app.run(|_app_handle, _event| {});
}

fn fetch_dars_data() -> Vec<Dars> {
    loop {
        match reqwest::blocking::get(
            "https://raw.githubusercontent.com/nazmul-pro/iustadji/data/dars.json",
        ) {
            Ok(response) => match response.text() {
                Ok(body) => match serde_json::from_str::<Vec<Dars>>(&body) {
                    Ok(all_dars) => return all_dars,
                    Err(err) => {
                        eprintln!("Failed to parse JSON: {}", err);
                    }
                },
                Err(err) => {
                    eprintln!("Failed to read response body: {}", err);
                }
            },
            Err(err) => {
                eprintln!("Failed to fetch JSON: {}", err);
            }
        }
        // Retry after 30 seconds
        std::thread::sleep(Duration::from_secs(30));
    }
}

fn init_notification(app_config: String) {
    // let settings = get_settings();
    // Spawn a new thread to handle notifications
    thread::spawn(move || {
        let settings = get_settings();
        let all_dars = fetch_dars_data();
        let file_path = "/Applications/iustadji-mac.app/Contents/Resources/dars.json";
        let json_content =
            serde_json::to_string_pretty(&all_dars).expect("Failed to serialize data to JSON");
        let mut file = File::create(file_path).unwrap();
        file.write_all(json_content.as_bytes())
            .expect("Failed to create settings file");
        // if !Path::new(file_path).exists() {
        //     let mut file = File::create(file_path).unwrap();
        //     file.write_all(json_content.as_bytes()).expect("Failed to create settings file");
        //     // fs::write("../public/dars.json", json_content).expect("Failed to create settings file");
        // }
        // Write the JSON content to the file
        // let mut file = File::create("../public/dars.json")
        //     .expect("Failed to create dars.json file");
        // file.write_all(json_content.as_bytes())
        //     .expect("Failed to write data to dars.json file");
        for dars in all_dars {
            for notification in dars.notifications {
                // Shows a notification with the given title and body
                Notification::new(&app_config)
                    .title(&notification.title)
                    .body(&notification.description)
                    .show()
                    .unwrap();

                thread::sleep(Duration::from_secs(settings.interval * 60));
            }
        }
    });
}

// Call this function to fetch dars data
