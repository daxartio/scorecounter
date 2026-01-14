mod app;
mod state;

use app::App;

fn main() {
    dioxus::launch(App);
}
