use std::time::Duration;
use chrono::{Local, NaiveDate};
use leptos::logging::log;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use thaw::mobile::{show_toast, ToastOptions};
use thaw::{Button, ButtonColor, Divider, Input, TimePicker};
use thaw::{DatePicker, InputNumber, SignalWatch, Switch};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DarsArg {
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsArg {
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationData {
    id: String,
    title: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dars {
    date: String,
    notifications: Vec<NotificationData>,
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
            data_url: "https://raw.githubusercontent.com/nazmul-pro/iustadji/data/dars.json".to_string(),
            interval: 10,
            dars_start_date: String::new(),
            dars_end_date: String::new(),
            mute_for: 0,
            mute_def: Vec::new(),
            pick_random: false,
            skip_ids: Vec::new(),
        }
    }
}

#[derive(Copy, Clone)]
struct DarsContext(ReadSignal<Vec<Dars>>, WriteSignal<Vec<Dars>>);

#[derive(Copy, Clone)]
struct AllDarsContext(ReadSignal<Vec<Dars>>, WriteSignal<Vec<Dars>>);

#[derive(Copy, Clone)]
struct SettingsContext(ReadSignal<Settings>, WriteSignal<Settings>);

#[component]
pub fn App() -> impl IntoView {
    let (all_dars, set_all_dars) = create_signal(vec![]);
    let (dars, set_dars) = create_signal(vec![]);
    provide_context(DarsContext(dars, set_dars));
    provide_context(AllDarsContext(all_dars, set_all_dars));

    let get_data = move || {
        spawn_local(async move {
            let args = to_value(&DarsArg {
                date: "10.10.2023".to_string(),
            })
            .unwrap();

            let dars_str = invoke("get_dars", args).await.as_string().unwrap();
            let data: Vec<Dars> = serde_json::from_str(&dars_str).unwrap();
            set_dars.set(data.clone());
            set_all_dars.set(data);
        });
        
    };

    get_data();

    view! {
        <Router>
            <div class="flex h-screen">
                <div class="bg-gray-800 text-white w-30">
                    <div class="p-4">
                        <h1 class="text-xl font-semibold">iUstadji</h1>
                        <ul class="mt-4">
                            <li><a href="/"><img src="public/1.png" class="logo tauri" /></a></li>
                            <li><a href="/settings"><img src="public/2.png" class="logo leptos" /></a></li>
                        </ul>
                    </div>
                </div>
                <div class="flex-grow overflow-auto">
            <Routes>
                <Route path="/" view=|| view! {
                    <div class="sticky top-0 bg-gray-100 p-3 text-xs">
                        <Header/>
                    </div>
                    <div class="overflow-auto text-xs">
                        <DarsList/>
                    </div>
                }/>
                <Route
                    path="/settings"
                    view=Settings
                >
                </Route>
            </Routes>
        </div>
            </div>
        </Router>
    }
}

#[component]
fn Settings() -> impl IntoView {

    let data_url = create_rw_signal(String::new());
    let interval = create_rw_signal(1);
    
    let sd = NaiveDate::parse_from_str("01.01.2023", "%d.%m.%Y").unwrap();
    let dars_start_date = create_rw_signal(Some(sd));

    let ed = NaiveDate::parse_from_str("12.12.2025", "%d.%m.%Y").unwrap();
    let dars_end_date = create_rw_signal(Some(ed));

    let mute_for = create_rw_signal(0);
    let pick_random = create_rw_signal(false);
    
    let (settings, set_settings) = create_signal(Settings::default());
    provide_context(SettingsContext(settings, set_settings));
    spawn_local(async move {
        let args = to_value(&DarsArg {
            date: "10.10.2023".to_string(),
        })
        .unwrap();
        let sett_str = invoke("get_settings_str", args).await.as_string().unwrap();
        let data: Settings = serde_json::from_str(&sett_str).unwrap();
        set_settings.set(data.clone());

        data_url.set(settings.get().data_url);
        interval.set(settings.get().interval);
        let sd = NaiveDate::parse_from_str(&settings.get().dars_start_date, "%d.%m.%Y").unwrap();
        dars_start_date.set(Some(sd));
        let ed = NaiveDate::parse_from_str(&settings.get().dars_end_date, "%d.%m.%Y").unwrap();
        dars_end_date.set(Some(ed));
        log!("{}", settings.get().pick_random);
        pick_random.set(settings.get().pick_random);
    });

    view! {
        <div class="sticky top-0 bg-gray-100 p-3 text-sm">
            <div class="flex">
                <p class="border text-center w-20 h-7 rounded-2xl font-bold bg-gray-800 text-white pt-1 mr-5">"Settings"</p>
            </div>
        </div>
        <div class="overflow-auto text-sm p-5">
            <div class="flex-col">
                <div class="flex items-center gap-2.5 mb-5">
                    <div>Data location</div>
                    <div class="flex-1"><Input value=data_url/></div>
                </div>
                <div class="flex items-center gap-2.5 mb-5">
                    <div>"Notification interval"</div>
                    <div><InputNumber value=interval step=5/></div> min
                </div>
                <div class="items-center gap-2.5 mb-5">
                    <p class="font-bold mb-2">Notify dars between:</p>
                    <div class="flex mb-2">
                        <p class="pr-2 pt-2">Start date</p>
                        <DatePicker value=dars_start_date/>
                    </div>
                    <div class="flex">
                        <p class="pr-2 pt-2">End date</p>
                        <DatePicker value=dars_end_date/>
                    </div>
                </div>
                <div class="flex items-center gap-2.5 mb-5">
                    <div>Notify random</div>
                    <div><Switch value=pick_random /></div>
                </div>
                // <div class="flex items-center gap-2.5 mb-5">
                //     <div>Mute for next</div>
                //     <div><InputNumber value=mute_for step=5/></div> min
                // </div>
                <div class="font-bold">Mute daily *(TBD)*</div>
                <Divider class="m-2"/>
                <div class="flex items-center gap-2.5 mb-5">
                    from <div><TimePicker /></div> to <div><TimePicker /></div>
                </div>
                <div class="flex items-center gap-2.5 mb-5">
                    from <div><TimePicker /></div> to <div><TimePicker /></div>
                </div>
                <div class="flex items-center gap-2.5 mb-5">
                    from <div><TimePicker /></div> to <div><TimePicker /></div>
                </div>
            </div>

            <Button on:click=move |_| {
                if data_url.get().is_empty() || 
                    interval.get() < 1 || 
                    dars_start_date.get().is_none() || 
                    dars_end_date.get().is_none() {
                        show_toast(ToastOptions {
                            message: format!("{}", "Invalid settings"),
                            duration: Duration::from_millis(3000),
                        });
                        return;
                }
                let new_settings: Settings = Settings {
                    data_url: data_url.get(),
                    interval: interval.get(),
                    dars_start_date: dars_start_date.get().unwrap().format("%d.%m.%Y").to_string(),
                    dars_end_date: dars_end_date.get().unwrap().format("%d.%m.%Y").to_string(),
                    pick_random: pick_random.get(),
                    mute_for: settings.get().mute_for,
                    mute_def: settings.get().mute_def,
                    skip_ids: settings.get().skip_ids,
                };
                spawn_local(async move {
                    let args = to_value(&SettingsArg {
                        data: serde_json::to_string_pretty(&new_settings).expect("msg")
                    })
                    .unwrap();
                    let msg = invoke("set_settings_str", args).await.as_string().unwrap();
                    show_toast(ToastOptions {
                        message: format!("{}", msg),
                        duration: Duration::from_millis(3000),
                    });
                });

            } class="mt-5" color=ButtonColor::Success>Save</Button>
        </div>
    }
}

#[component]
fn Header() -> impl IntoView {
    let start = create_rw_signal(Some(Local::now().date_naive()));
    let end = create_rw_signal(Some(Local::now().date_naive()));
    let _ = start.watch(move |_| {
        filter_dars(start.get().unwrap(), end.get().unwrap());
    });

    let _ = end.watch(move |_| {
        filter_dars(start.get().unwrap(), end.get().unwrap());
    });
    view! {
        <div class="flex">
            <p class="border text-center w-16 h-7 rounded-2xl font-bold bg-gray-800 text-white pt-1 mr-5">"Dars"</p>
            <div class="flex pr-4">
                <p class="pr-2 pt-2">Start Date</p>
                <DatePicker value=start/>
            </div>
            <div class="flex">
                <p class="pr-2 pt-2">End Date</p>
                <DatePicker value=end/>
            </div>
        </div>
    }
}

#[component]
fn DarsList() -> impl IntoView {
    let dars = use_context::<DarsContext>().unwrap().0;

    view! {
        <For
            each= move || dars.get()
            key=|state| state.date.clone()
            let:child
        >
            <div class="flex justify-center"><p class="border rounded-2xl font-bold text-center m-4 p-1 bg-green-600 text-white w-40">{format_date(&child.date)}</p></div>
            <For
                each= move || child.notifications.clone()
                key=|state| state.id.clone()
                let:child
            >

                <div href="#" class="block p-6 m-2 bg-white border border-gray-200 rounded-lg shadow hover:bg-gray-100">
                    <h6 class="mb-2 font-bold tracking-tight text-gray-900">{child.title}</h6>
                    <p class="font-normal text-gray-700 dark:text-gray-700">{child.description}</p>
                </div>

            </For>
        </For>
    }
}

fn format_date(date: &str) -> String {
    let date = NaiveDate::parse_from_str(date, "%d.%m.%Y").expect("Invalid date format");

    let formatted_date = date.format("%a, %-d %b %Y").to_string();
    formatted_date
}

fn filter_dars(start: NaiveDate, end: NaiveDate) {
    let all_dars = use_context::<AllDarsContext>().unwrap().0;
    let set_dars = use_context::<DarsContext>().unwrap().1;

    let filtered = all_dars
        .get_untracked()
        .iter()
        .filter(|d| {
            let dars_date = NaiveDate::parse_from_str(&d.date, "%d.%m.%Y").unwrap();
            dars_date >= start && dars_date <= end
        })
        .cloned()
        .collect::<Vec<_>>();
    set_dars.set(filtered);
}
