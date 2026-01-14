use crate::state::{Counter, SCHEMA_VERSION, STORAGE_KEY, Store, StoredData};
use dioxus::prelude::*;
use js_sys::Date;
use uuid::Uuid;

const LONG_PRESS_THRESHOLD_MS: f64 = 520.0;
const MIN_ROW_HEIGHT_PX: f32 = 96.0;
const PALETTE: [&str; 8] = [
    "#0f172a", "#1e3a8a", "#047857", "#9d174d", "#7c3aed", "#ea580c", "#2563eb", "#0f766e",
];

#[derive(Clone, Debug, PartialEq)]
enum DialogState {
    Closed,
    Add(CounterDraft),
    Edit(CounterDraft),
}

#[derive(Clone, Debug, PartialEq)]
struct CounterDraft {
    id: Option<String>,
    name: String,
    score: i32,
    color: String,
}

impl CounterDraft {
    fn new(color: String, existing_total: usize) -> Self {
        Self {
            id: None,
            name: format!("Player {}", existing_total + 1),
            score: 0,
            color,
        }
    }

    fn from_counter(counter: &Counter) -> Self {
        Self {
            id: Some(counter.id.clone()),
            name: counter.name.clone(),
            score: counter.score,
            color: counter.color.clone(),
        }
    }

    fn into_counter(self) -> Counter {
        Counter {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: self.name.trim().to_string(),
            score: self.score,
            color: self.color,
        }
    }
}

#[component]
pub fn App() -> Element {
    let store = use_signal(Store::default);
    let loaded = use_signal(|| false);
    let mut dialog = use_signal(|| DialogState::Closed);
    let next_palette = use_signal(|| 0usize);

    use_effect({
        let store = store.clone();
        let loaded = loaded.clone();
        move || {
            load_from_storage(store.clone(), loaded.clone());
        }
    });

    use_effect({
        let store = store.clone();
        let loaded = loaded.clone();
        move || {
            if !loaded() {
                return;
            }
            let _ = persist_store(&store());
        }
    });

    let counters = store().counters;
    let total_rows = counters.len();
    let row_height = row_height(total_rows);

    let mut open_add = {
        let mut dialog = dialog.clone();
        let next_palette = next_palette.clone();
        let total_rows = total_rows;
        move || {
            dialog.set(DialogState::Add(CounterDraft::new(
                next_color(&next_palette),
                total_rows,
            )));
        }
    };

    let mut on_save = {
        let mut store = store.clone();
        let mut dialog = dialog.clone();
        let mut next_palette = next_palette.clone();
        move |draft: CounterDraft| {
            let is_new = draft.id.is_none();
            store.with_mut(|state| state.upsert(draft.into_counter()));
            if is_new {
                next_palette.with_mut(|idx| *idx = (*idx + 1) % PALETTE.len());
            }
            dialog.set(DialogState::Closed);
        }
    };

    let mut handle_delete = {
        let mut store = store.clone();
        let mut dialog = dialog.clone();
        move |id: String| {
            store.with_mut(|state| state.remove(&id));
            dialog.set(DialogState::Closed);
        }
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        document::Link { rel: "manifest", href: asset!("/assets/manifest.webmanifest") }
        document::Script { src: asset!("/assets/sw-register.js") }
        main {
            class: "app",
            div { class: "hud",
                div { class: "branding",
                    span { class: "dot" }
                    h1 { "Score Counter" }
                    p { "Press to adjust, hold for quick +5/-5." }
                }
                div { class: "summary",
                    span { "{total_rows} counters" }
                }
            }
            if counters.is_empty() {
                EmptyState { on_add: move |_| open_add() }
            } else {
                div { class: "rows",
                    style: "height: 100vh;",
                    for counter in counters {
                        ScoreRow {
                            counter,
                            total: total_rows,
                            row_height: row_height.clone(),
                            on_edit: move |id| {
                                let state = store.read();
                                if let Some(current) = state.counters.iter().find(|c| c.id == id) {
                                    dialog.set(DialogState::Edit(CounterDraft::from_counter(current)));
                                }
                            },
                            on_delete: move |id| handle_delete(id),
                            on_adjust: {
                                let mut store = store.clone();
                                move |(id, delta): (String, i32)| {
                                    let _ = store.with_mut(|s| s.adjust_score(&id, delta));
                                }
                            },
                        }
                    }
                }
            }
            AddButton { on_click: move |_| open_add() }
            if let DialogState::Add(form) = dialog() {
                RowDialog {
                    title: "Add counter".to_string(),
                    form,
                    on_close: move |_| dialog.set(DialogState::Closed),
                    on_save: move |draft| on_save(draft),
                    on_delete: move |_| {},
                    show_delete: false,
                }
            } else if let DialogState::Edit(form) = dialog() {
                RowDialog {
                    title: "Edit counter".to_string(),
                    on_close: move |_| dialog.set(DialogState::Closed),
                    on_save: move |draft| on_save(draft),
                    show_delete: form.id.is_some(),
                    on_delete: {
                        let delete_id = form.id.clone();
                        let mut handle_delete = handle_delete.clone();
                        move |_| {
                            if let Some(id) = delete_id.clone() {
                                handle_delete(id);
                            }
                        }
                    },
                    form,
                }
            }
        }
    }
}

fn next_color(next_palette: &Signal<usize>) -> String {
    let idx = next_palette();
    PALETTE[idx % PALETTE.len()].to_string()
}

fn row_height(total_rows: usize) -> String {
    if total_rows == 0 {
        return "100vh".to_string();
    }
    let vh_height = 100.0 / total_rows as f32;
    format!("max({vh_height:.3}vh, {MIN_ROW_HEIGHT_PX}px)")
}

fn load_from_storage(mut store: Signal<Store>, mut loaded: Signal<bool>) {
    let window = web_sys::window();
    let storage = window.and_then(|w| w.local_storage().ok().flatten());
    if let Some(storage) = storage {
        if let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) {
            if let Ok(payload) = serde_json::from_str::<StoredData>(&raw) {
                let hydrated = if payload.schema_version == SCHEMA_VERSION {
                    Store::from_storage(payload)
                } else {
                    Store {
                        counters: payload.counters,
                    }
                };
                store.set(hydrated);
            }
        }
    }
    loaded.set(true);
}

fn persist_store(store: &Store) -> Result<(), String> {
    let payload = serde_json::to_string(&store.to_storage()).map_err(|e| e.to_string())?;
    let window = web_sys::window().ok_or("Missing window")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Local storage unavailable".to_string())?
        .ok_or("Local storage unavailable".to_string())?;
    storage
        .set_item(STORAGE_KEY, &payload)
        .map_err(|_| "Failed to persist".to_string())
}

#[component]
fn ScoreRow(
    counter: Counter,
    total: usize,
    row_height: String,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
    on_adjust: EventHandler<(String, i32)>,
) -> Element {
    let press_start = use_signal(|| None::<f64>);
    let counter_id_for_edit = counter.id.clone();
    let counter_id_for_delete = counter.id.clone();
    let counter_id_for_adjust = counter.id.clone();

    rsx! {
        div {
            class: "row",
            style: format!("background: {}; height: {row_height};", counter.color),
            div { class: "row-actions",
                button {
                    class: "icon-button ghost",
                    r#type: "button",
                    onclick: move |_| on_edit.call(counter_id_for_edit.clone()),
                    "Edit"
                }
                button {
                    class: "icon-button ghost",
                    r#type: "button",
                    onclick: move |_| on_delete.call(counter_id_for_delete.clone()),
                    "Delete"
                }
            }
            div { class: "row-content",
                button {
                    class: "pill minus",
                    r#type: "button",
                    onpointerdown: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(Some(Date::now()))
                    },
                    onpointerup: {
                        let mut press_start = press_start.clone();
                        let id = counter_id_for_adjust.clone();
                        let on_adjust = on_adjust.clone();
                        move |_| {
                            if let Some(start) = press_start() {
                                let elapsed = Date::now() - start;
                                let delta = if elapsed >= LONG_PRESS_THRESHOLD_MS { -5 } else { -1 };
                                on_adjust.call((id.clone(), delta));
                            }
                            press_start.set(None);
                        }
                    },
                    onpointerleave: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(None)
                    },
                    onpointercancel: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(None)
                    },
                    "-"
                }
                div { class: "center",
                    span { class: "name", "{counter.name}" }
                    span { class: "score", class: if counter.score < 0 { "negative" } else { "" }, "{counter.score}" }
                    span { class: "hint", "{total} rows • hold for ±5" }
                }
                button {
                    class: "pill plus",
                    r#type: "button",
                    onpointerdown: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(Some(Date::now()))
                    },
                    onpointerup: {
                        let mut press_start = press_start.clone();
                        let id = counter_id_for_adjust.clone();
                        let on_adjust = on_adjust.clone();
                        move |_| {
                            if let Some(start) = press_start() {
                                let elapsed = Date::now() - start;
                                let delta = if elapsed >= LONG_PRESS_THRESHOLD_MS { 5 } else { 1 };
                                on_adjust.call((id.clone(), delta));
                            }
                            press_start.set(None);
                        }
                    },
                    onpointerleave: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(None)
                    },
                    onpointercancel: {
                        let mut press_start = press_start.clone();
                        move |_| press_start.set(None)
                    },
                    "+"
                }
            }
        }
    }
}

#[component]
fn RowDialog(
    title: String,
    mut form: CounterDraft,
    on_close: EventHandler<()>,
    on_save: EventHandler<CounterDraft>,
    show_delete: bool,
    on_delete: EventHandler<()>,
) -> Element {
    let mut name_value = use_signal(|| form.name.clone());
    let mut score_value = use_signal(|| form.score);
    let mut color_value = use_signal(|| form.color.clone());

    use_effect({
        let form = form.clone();
        let mut name_value = name_value.clone();
        let mut score_value = score_value.clone();
        let mut color_value = color_value.clone();
        move || {
            name_value.set(form.name.clone());
            score_value.set(form.score);
            color_value.set(form.color.clone());
        }
    });

    rsx! {
        div { class: "dialog-backdrop",
            onclick: move |_| on_close.call(()),
            div { class: "dialog", onclick: move |evt| evt.stop_propagation(),
                h2 { "{title}" }
                label { "Name" }
                input {
                    r#type: "text",
                    value: "{name_value}",
                    oninput: move |evt| name_value.set(evt.value()),
                    placeholder: "Player name",
                }
                label { "Score" }
                input {
                    r#type: "number",
                    value: "{score_value}",
                    oninput: move |evt| {
                        if let Ok(parsed) = evt.value().parse::<i32>() {
                            score_value.set(parsed);
                        }
                    },
                }
                label { "Color" }
                div { class: "palette",
                    for swatch in PALETTE {
                        button {
                            class: "swatch",
                            r#type: "button",
                            style: format!("background: {swatch};"),
                            onclick: move |_| color_value.set(swatch.to_string()),
                            aria_selected: (swatch == color_value()).then_some("true"),
                        }
                    }
                    input {
                        class: "color-picker",
                        r#type: "color",
                        value: "{color_value}",
                        oninput: move |evt| color_value.set(evt.value()),
                    }
                }
                div { class: "dialog-actions",
                    if show_delete {
                        button {
                            class: "ghost danger",
                            r#type: "button",
                            onclick: {
                                let delete_handler = on_delete.clone();
                                move |_| delete_handler.call(())
                            },
                            "Delete"
                        }
                    }
                    div { class: "spacer" }
                    button {
                        class: "ghost",
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "primary",
                        r#type: "button",
                        onclick: move |_| {
                            form.name = name_value();
                            form.score = score_value();
                            form.color = color_value();
                            on_save.call(form.clone());
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyState(on_add: EventHandler<()>) -> Element {
    rsx! {
        div { class: "empty",
            h2 { "No counters yet" }
            p { "Create your first row to start tracking scores." }
            button {
                class: "primary",
                r#type: "button",
                onclick: move |_| on_add.call(()),
                "+ Add counter"
            }
        }
    }
}

#[component]
fn AddButton(on_click: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "fab",
            r#type: "button",
            onclick: move |_| on_click.call(()),
            "+ Add"
        }
    }
}
