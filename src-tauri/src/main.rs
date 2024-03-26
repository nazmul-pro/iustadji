use chrono::{Local, NaiveDate};
use rand::seq::SliceRandom;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tauri::api::notification::Notification;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

#[macro_use]
extern crate lazy_static;

const DATA_URL: &str = "https://raw.githubusercontent.com/nazmul-pro/iustadji/data/dars.json";
const DARS_FILE_PATH: &str = "/Applications/iUstadji.app/Contents/Resources/data/dars.json";
const SETTINGS_FILE_PATH: &str = "/Applications/iUstadji.app/Contents/Resources/data/settings.json";

lazy_static! {
    static ref THREAD_IDS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    static ref NOTIFICATIONS: Arc<Mutex<Vec<NotificationData>>> = Arc::new(Mutex::new(vec![]));
    static ref MUTE_FOR: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    static ref APP_CONFIG: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref SETTINGS_UPDATED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MuteDef {
    recur: String,
    start: String,
    end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    data_url: String,
    interval: u64,
    dars_start_date: String,
    dars_end_date: String,
    mute_for: i32,
    mute_def: Vec<MuteDef>,
    pick_random: bool,
    skip_ids: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            data_url: DATA_URL.into(),
            interval: 10,
            dars_start_date: String::from("01.01.2024"),
            dars_end_date: String::from("31.12.2025"),
            mute_for: 0,
            mute_def: Vec::new(),
            pick_random: false,
            skip_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationData {
    id: String,
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Dars {
    date: String,
    notifications: Vec<NotificationData>,
}

impl Default for Dars {
    fn default() -> Self {
        Self {
            date: "01.01.2024".to_string(),
            notifications: vec![NotificationData {
                id: "start_id".to_string(),
                title: "تسمية".to_string(),
                description: "بِسْمِ ٱللَّٰهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
            }],
        }
    }
}

#[tauri::command]
fn get_dars() -> String {
    let dars = fetch_dars_data();
    serde_json::to_string_pretty(&dars).expect("Ustadji: error parsing to JSON")
}

#[tauri::command]
fn get_settings_str() -> String {
    serde_json::to_string_pretty(&get_settings()).expect("Ustadji: error parsing to JSON")
}

#[tauri::command]
fn set_settings_str(data: String) -> String {
    if let Ok(setting) = serde_json::from_str::<Settings>(&data) {
        if let Ok(settings_json) = serde_json::to_string_pretty(&vec![setting]) {
            if let Err(err) = fs::write(SETTINGS_FILE_PATH, settings_json) {
                return format!("Failed to write settings file: {}", err);
            } else {
                *SETTINGS_UPDATED.lock().unwrap() = true;
                thread::spawn(move || {
                    populate_notifications();
                    *SETTINGS_UPDATED.lock().unwrap() = false;
                    init_notification(APP_CONFIG.lock().unwrap().clone());
                });

                return String::from("Settings successfully updated");
            }
        } else {
            return String::from("Failed to serialize default settings to JSON");
        }
    } else {
        return String::from("Error parsing settings data");
    }
}

#[tauri::command]
fn get_settings() -> Settings {
    let settings = {
        let file_path = SETTINGS_FILE_PATH;

        if !Path::new(file_path).exists() {
            // If the file doesn't exist, create it with default settings
            let default_settings = vec![Settings::default()];

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

fn main() {
    fetch_dars_data();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let open: CustomMenuItem = CustomMenuItem::new("open".to_string(), "Open");
    let unmute: CustomMenuItem = CustomMenuItem::new("unmute".to_string(), "Unmute");
    let mute_30: CustomMenuItem = CustomMenuItem::new("mute_30".to_string(), "Mute for 30 mins");
    let mute_60: CustomMenuItem = CustomMenuItem::new("mute_60".to_string(), "Mute for 1 hr");
    let mute_restart: CustomMenuItem =
        CustomMenuItem::new("mute_restart".to_string(), "Mute until unmute/restart");

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
        .invoke_handler(tauri::generate_handler![
            get_dars,
            get_settings_str,
            set_settings_str
        ])
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
                "unmute" => {
                    *MUTE_FOR.lock().unwrap() = 0; // change to real value self, below and sleep time
                }
                "mute_30" => {
                    *MUTE_FOR.lock().unwrap() = 30;
                }
                "mute_60" => {
                    *MUTE_FOR.lock().unwrap() = 60;
                }
                "mute_restart" => {
                    *MUTE_FOR.lock().unwrap() = 1440; // 1d
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while building tauri application");
    *APP_CONFIG.lock().unwrap() = app.config().tauri.bundle.identifier.clone();

    populate_notifications();

    init_notification(APP_CONFIG.lock().unwrap().clone());

    // Run the app
    app.run(|_app_handle, _event| {});
}

fn populate_notifications() {
    let all_dars = fetch_dars_data();
    let settings = get_settings();
    let mut rng = rand::thread_rng();
    let mut all_notif: Vec<NotificationData> = vec![];

    for dars in all_dars {
        if let Ok(start_date) = NaiveDate::parse_from_str(&settings.dars_start_date, "%d.%m.%Y") {
            if let Ok(end_date) = NaiveDate::parse_from_str(&settings.dars_end_date, "%d.%m.%Y") {
                let notifications = dars.notifications.clone();

                for notification in notifications {
                    if let Ok(notification_date) = NaiveDate::parse_from_str(&dars.date, "%d.%m.%Y")
                    {
                        if notification_date >= start_date && notification_date <= end_date {
                            let notif = NotificationData {
                                id: String::from(&dars.date) + &notification.id,
                                title: notification.title,
                                description: notification.description,
                            };
                            all_notif.push(notif);
                        }
                    }
                }
            }
        }
    }
    if settings.pick_random {
        all_notif.shuffle(&mut rng);
    }
    *NOTIFICATIONS.lock().unwrap() = all_notif;
}

fn fetch_dars_data() -> Vec<Dars> {
    let settings = get_settings();
    let mut tried = 0;
    loop {
        tried += 1;
        match reqwest::blocking::get(&settings.data_url) {
            Ok(response) => match response.text() {
                Ok(body) => match serde_json::from_str::<Vec<Dars>>(&body) {
                    Ok(all_dars) => {
                        return all_dars;
                    }
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
        if tried > 3 {
            if let Ok(file_content) = fs::read_to_string(DARS_FILE_PATH) {
                // will serve data from local json file if not resolve api after 30 sec
                return serde_json::from_str::<Vec<Dars>>(&file_content).unwrap();
            }
        } else {
            // Retry after 10 seconds
            std::thread::sleep(Duration::from_secs(10));
        }
    }
}

fn init_notification(app_config: String) {
    thread::spawn(move || {
        let t_id = Local::now().to_string();
        THREAD_IDS.lock().unwrap().push(t_id.clone());
        let settings = get_settings();
        let mut count = NOTIFICATIONS.lock().unwrap().len();

        for notification in NOTIFICATIONS.lock().unwrap().iter() {
            // close this slept thread if init from anywhere
            if *THREAD_IDS.lock().unwrap().last().unwrap() != t_id {
                break;
            }

            if settings.interval > *MUTE_FOR.lock().unwrap() {
                println!(
                    "msg {} mute for = {} interval = {}",
                    &notification.description,
                    *MUTE_FOR.lock().unwrap(),
                    settings.interval
                );
                // Shows a notification with the given title and body
                Notification::new(&app_config)
                    .title(&notification.title)
                    .body(&notification.description)
                    .show()
                    .unwrap();

                // break per min for smooth transition btwn two save settings of diff interval
                for i in 0..settings.interval {
                    // wip for quick transition while save settings
                    println!("loop {} interval {}", i, settings.interval);
                    if *SETTINGS_UPDATED.lock().unwrap() {
                        break;
                    }
                    thread::sleep(Duration::from_secs(60)); // 60
                }
            } else {
                let mute_for = *MUTE_FOR.lock().unwrap();
                // break per min for smooth transition btwn mute/unmute
                for _ in 0..mute_for {
                    thread::sleep(Duration::from_secs(60)); // 60
                    if *MUTE_FOR.lock().unwrap() == 0 {
                        break;
                    }
                }
                *MUTE_FOR.lock().unwrap() = 0;
            };

            count = count - 1;
            if count == 0 {
                // notify from start
                init_notification(app_config.clone());
            }
        }
    });
}
