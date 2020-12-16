#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;
extern crate glib;
#[macro_use] extern crate conrod_derive;
// Async
use std::sync::{Arc, Mutex};
// other imports
//extern crate raw_window_handle;

mod media_player;
mod support;

fn main() {
    //let mut application_state: Arc<Mutex<AppWindow>> = Arc::new(Mutex::new(AppWindow::new()));
    let mut application_state: AppWindow = AppWindow::new();
    media_player::media_player::main(application_state);
}

        
#[derive(Copy, Clone)]
pub struct AppWindow {
    app_font_id: Option<conrod_core::text::font::Id>,
    slider_indicator_loop_set: bool,
}

impl AppWindow {
    fn new () -> AppWindow {
        AppWindow {
            app_font_id: None,
            slider_indicator_loop_set: false
        }
    }
}
/*
struct Mechanical {
    state: State,
}

struct State {
    text_slider_space: canvas::Cache,
    system_cache: canvas::Cache,
    cursor_position: Point,
    media_start_time: f64,
    current_media_time: f64,
    beat_map: BeatMap,
}

impl State {
    pub fn new() -> State {
        let now: Instant = Instant::now();
        let (width, height) : (u32, u32)= window::Settings::default().size;
        State {
            text_slider_space: Default::default(),
            system_cache: Default::default(),
            cursor_position: Point::ORIGIN,
            media_start_time: 0.0,
            // TODO
            current_media_time: 0.0,
            // In progress, maybe rename TODO
            beat_map: Self::generate_letters(letter)
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }

    fn 
}

// A BeatMap is a tuple of chars and f64.
// The slider will show the characters (the char) to press at a certain time (the f64).
// They will be ruled by the time of the current media playing
struct BeatMap {
    character: Vec<char>,
    time: Vec<f64>
}

*/