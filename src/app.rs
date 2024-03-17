use chrono::NaiveDate;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use thaw::DatePicker;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationData {
    id: i32,
    title: String,
    description: String,
}

// Define the Dars struct
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dars {
    date: String,
    notifications: Vec<NotificationData>,
}

#[component]
pub fn App() -> impl IntoView {
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
                        <div class="flex">
                            <p class="border text-center w-16 h-7 rounded-2xl font-bold bg-gray-800 text-white pt-1 mr-5">"Dars"</p>
                            <div class="flex pr-4">
                                <p class="pr-2 pt-2">Start Date</p>
                                <DatePicker/>
                            </div>
                            <div class="flex">
                                <p class="pr-2 pt-2">End Date</p>
                                <DatePicker/>
                            </div>
                        </div>
                    </div>
                    <div class="overflow-auto text-xs">
                        <StaticList/>
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
    view! {
        <div class="sticky top-0 bg-gray-100 p-3 text-sm">
            <div class="flex">
                <p class="border text-center w-20 h-7 rounded-2xl font-bold bg-gray-800 text-white pt-1 mr-5">"Settings"</p>
            </div>
        </div>
        <div class="overflow-auto text-sm">
            "Settings----"
        </div>
    }
}

#[component]
fn StaticList(
) -> impl IntoView {
    // create counter signals that start at incrementing numbers
    let (dars, set_dars) = create_signal(vec![]);
    let get_data = move || {
        spawn_local(async move {
            let args = to_value(&DarsArg {
                date: "10.10.2023".to_string(),
            })
            .unwrap();
            // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
            let new_msg = invoke("get_dars", args).await.as_string().unwrap();
            // let dars = serde_json::from_str::<_>(&new_msg).unwrap();
            let data: Vec<Dars> = serde_json::from_str(&new_msg).unwrap();
            set_dars.set(data);
        })
    };

    get_data();

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
