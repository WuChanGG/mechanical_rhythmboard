
pub mod media_player {
    use glib;
    // gstreamer
    use std::os::raw::c_void;
    use std::process;
    use gstreamer::prelude::*;
    use gstreamer_video::prelude::*;
    use std::ops;
    // conrod
    use glium::Surface;
    use glium::glutin;
    use conrod_core::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget, position::Dimension, Point};
    use conrod_core;
    use crate::support;
    // mechanical
    use crate::AppWindow;
    // sync
    use std::sync::{Arc, Mutex};
    // other imports
    use raw_window_handle;
    use raw_window_handle::{HasRawWindowHandle};

    #[derive(WidgetCommon)]
    pub struct VideoSlider {
        #[conrod(common_builder)]
        common: widget::CommonBuilder,
    }
    
    pub fn main(mut application_state: AppWindow) {
        //let application_state = Arc::clone(&application_state);
        //let mut application_state_lock = application_state.lock().unwrap();
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;
        let APPLICATION_NAME: String = String::from("Mechanical Rhythmboard");
        let path_to_font: String = String::from("Menlo-Regular.ttf");
        // Build the window
        // EventLoop can be considered a "context" for retrieving events
        // from the system and registered windows
        let event_loop: glium::glutin::event_loop::EventLoop<()> = 
            glium::glutin::event_loop::EventLoop::new();
        let window = glium::glutin::window::WindowBuilder::new()
            .with_title(APPLICATION_NAME)
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window
            , context, &event_loop).unwrap();
        // Hook the video streamer to the window
        start_gstreamer(&display);
        // Construct the UI
        let mut ui = conrod_core::UiBuilder::new([WIDTH as f64
            , HEIGHT as f64]).build();

        // Get the assets folder path
        let assets = find_folder::Search::KidsThenParents(1, 1)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path_to_font);
        let app_font_id = ui.fonts.insert_from_file(font_path).unwrap();
        application_state.app_font_id = Some(app_font_id);

        // A type used for converting 'conrod_core::render::Primitives' into 'Command'
        // that can be used for drawing to the glium 'Surface'
        let mut renderer =
            conrod_glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none)
        let image_map =
            conrod_core::image::Map::<glium::texture::Texture2d>::new();


        // Instantiate the generated list of widget identifiers
        let mut ids = Ids::new(ui.widget_id_generator());

        // Poll events from the window.
        support::run_loop(display, event_loop, move |request, display| {
            match request {
                support::Request::Event {
                    event,
                    should_update_ui,
                    should_exit,
                } => {
                    if let Some(event) = support::convert_event(
                        &event, &display.gl_window().window()) {
                        // global_input handles aggregation of events and provides these
                        // events to widgets (which widgets are capturing input)
                        ui.handle_event(event);
                        *should_update_ui = true;
                    }

                    match event {
                        glium::glutin::event::Event::WindowEvent {
                            event, ..} => match event {
                                glutin::event::WindowEvent::CloseRequested
                                | glutin::event::WindowEvent::KeyboardInput {
                                    input:
                                        glium::glutin::event::KeyboardInput {
                                            virtual_keycode:
                                                Some(glium::glutin::event::VirtualKeyCode::Escape),
                                                ..
                                        },
                                    ..
                                } => *should_exit = true,
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                support::Request::SetUi { needs_redraw } => {
                    set_widgets(ui.set_widgets(), &mut ids, application_state, display);
                    *needs_redraw = ui.has_changed();
                }
                support::Request::Redraw => {
                    let primitives = ui.draw();
                    renderer.fill(display, primitives, &image_map);
                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 0.0, 1.0);
                    renderer.draw(display, &mut target, &image_map).unwrap();
                    target.finish().unwrap();
                }
            }
        });

    }

    // GUI Section
    fn set_widgets(ref mut ui: conrod_core::UiCell, ids: &mut Ids, mut application_state: AppWindow, display: &glium::Display) {
        //let mut application_state_lock = application_state.lock().unwrap();
        let video_controls_length: f64 = 50.0;
        widget::Canvas::new().flow_down(&[
            // Video area
            (
                ids.video_area,
                widget::Canvas::new().color(color::BLACK)
                    .pad_bottom(20.0)
            ),
            // Game area
            (
                ids.game_area,
                widget::Canvas::new().color(color::CHARCOAL)
                    .pad_bottom(20.0)
            ),
            // Video controls area
            (
                ids.video_controls,
                widget::Canvas::new().color(color::BLACK)
                    //.pad_bottom(20.0)
                    .length(video_controls_length)
                    .flow_down(&[
                        (
                            ids.video_slider_canvas,
                            widget::Canvas::new().color(color::BLACK)
                            .length(10.0)
                        ),
                        (
                            ids.video_buttons_canvas,
                            widget::Canvas::new().color(color::BLACK)
                            .flow_right(&[
                                // Play button area
                                (
                                    ids.play_button_canvas,
                                    widget::Canvas::new().color(color::BLACK)
                                ),
                                // Pause Button Area
                                (
                                    ids.pause_button_canvas,
                                    widget::Canvas::new().color(color::BLACK)
                                )
                            ])
                        ),
                    ])
            ),
        ])
        .set(ids.master, ui);

        // Play button
        for _click in widget::Button::new()
            .color(color::CHARCOAL)
            .label("Play")
            .label_color(color::WHITE)
            .label_font_id(application_state.app_font_id.unwrap())
            .wh_of(ids.play_button_canvas)
            .mid_right_of(ids.play_button_canvas)
            .set(ids.play_button, ui)
        {
        }
        // Pause button
        for _click in widget::Button::new()
            .color(color::CHARCOAL)
            .label("Pause")
            .label_color(color::WHITE)
            .label_font_id(application_state.app_font_id.unwrap())
            .wh_of(ids.pause_button_canvas)
            .mid_right_of(ids.pause_button_canvas)
            .set(ids.pause_button, ui)
        {
        }

        // Slider indicator
        // TODO move this circle in glib task and also in the previous loop
        let video_slider_canvas_rect = ui.rect_of(ids.video_slider_canvas).unwrap();
        widget::Circle::fill(video_slider_canvas_rect.h() / 2.0)
            .color(color::BLACK)
            .xy(video_slider_canvas_rect.xy())
            .set(ids.video_slider_indicator, ui);

        // Video Slider Controls
        widget::Rectangle::fill([0.0, 0.0])
            .color(color::DARK_RED)
            .wh_of(ids.video_slider_canvas)
            .mid_right_of(ids.video_slider_canvas)
            .set(ids.video_slider, ui);

        let playbin: gstreamer::Element;
        let slider_input = ui.widget_input(ids.video_slider);
        // For each click
        // for slider_input_presses in slider_input.clicks() {

        //     let click_point = slider_input_presses.xy;
        //     let width = ui.rect_of(ids.video_slider).unwrap().w();            
        //     // Percentage of the bar at clicked location
        //     let value = (click_point.get(0).unwrap() / width) as f64;
        //     // Duration of the video
        //     if let Some(dur) = playbin.query_duration::<gstreamer::ClockTime>() {
        //         let seconds = dur / gstreamer::SECOND;
        //         let float_seconds = seconds.map(|v| v as f64).unwrap_or(0.0);
        //         println!("Video slider changed with a percentage of {} and a duration of {}", value, float_seconds);
        //         let seek_time = float_seconds * value;
        //         let seek_clock_time = gstreamer::ClockTime::from_seconds(seek_time as u64);
        //         playbin.seek_simple(
        //             gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
        //             seek_clock_time
        //         );
        //     }

        // }
        
    
        // let pipeline = playbin.clone();
        // let indicator_rect = ui.rect_of(ids.video_slider_indicator).unwrap();
        // if application_state.slider_indicator_loop_set == false {
        //     glib::timeout_add_seconds_local(1, move || {
        //         let pipeline = &pipeline;
                
        //         let video_location_percentage = get_video_location_as_percent(&pipeline);
        //         let new_circle_x_position = video_location_percentage * video_slider_canvas_rect.w();
        //         indicator_rect.shift_x(new_circle_x_position);
        //         return Continue(true);
        //     });

        //     application_state.slider_indicator_loop_set = true;
        // }

        
        // let video_overlay = playbin
        //     .clone()
        //     .dynamic_cast::<gstreamer_video::VideoOverlay>()
        //     .unwrap();

        // unsafe {
        //     let rwh = display.gl_window().window().raw_window_handle();
        //     let hwnd = match rwh {
        //         raw_window_handle::RawWindowHandle::Windows(windows_handle) => Some(windows_handle.hwnd),
        //         _ => None
        //     };
        //     video_overlay.set_window_handle(hwnd.unwrap() as usize);
        // }
    }

    fn get_video_location_as_percent(playbin: &gstreamer::Element) 
        -> f64 {
        if let Some(pos) = playbin.query_position::<gstreamer::ClockTime>() {
            let position_seconds = pos / gstreamer::SECOND;
            if let Some(duration) = playbin.query_duration::<gstreamer::ClockTime>() {
                let duration_seconds = duration / gstreamer::SECOND;
                let percentage = position_seconds / duration_seconds;
                return percentage.map(|v| v as f64).unwrap();
            }
        }
        return 0.0;
    }

    fn start_gstreamer(display: &glium::Display) {
        gstreamer::init().unwrap();
        //let uri = "file:///c:/Videos/1280.mp4";
        let uri = "https://www.freedesktop.org/software/gstreamer-sdk/\
                   data/media/sintel_trailer-480p.webm";

        let playbin = 
            gstreamer::ElementFactory::make("playbin", None).unwrap_or_else(|err| {
                println!("{:?}", err);
                panic!();
            });
        playbin.set_property("uri", &uri).unwrap_or_else(|err| {
            println!("{:?}", err);
        });

        // playbin
        //     .connect("video-tags-changed", false, |args| {
        //         let pipeline = args[0]
        //             .get::<gstreamer::Element>()
        //             .expect("playbin \"video-tags-changed\" args[0]")
        //             .unwrap();
        //         post_app_message(&pipeline);
        //         None
        //     })
        //     .unwrap();

        // playbin
        //     .connect("audio-tags-changed", false, |args| {
        //         let pipeline = args[0]
        //             .get::<gstreamer::Element>()
        //             .expect("playbin \"audio-tags-changed\" args[0]")
        //             .unwrap();
        //         post_app_message(&pipeline);
        //         None
        //     })
        //     .unwrap();

        // playbin
        //     .connect("text-tags-changed", false, move |args| {
        //         let pipeline = args[0]
        //             .get::<gstreamer::Element>()
        //             .expect("playbin \"text-tags-changed\" args[0]")
        //             .unwrap();
        //         post_app_message(&pipeline);
        //         None
        //     })
        //     .unwrap();

        // let video_overlay = playbin
        //     .clone()
        //     .dynamic_cast::<gstreamer_video::VideoOverlay>()
        //     .unwrap();

        // unsafe {
        //     let rwh = display.gl_window().window().raw_window_handle();
        //     let hwnd = match rwh {
        //         raw_window_handle::RawWindowHandle::Windows(windows_handle) => Some(windows_handle.hwnd),
        //         _ => None
        //     };
        //     video_overlay.set_window_handle(hwnd.unwrap() as usize);
        // }
        
        // let bus = playbin.get_bus().unwrap();
        // bus.add_signal_watch();

        // let pipeline_weak = playbin.downgrade();
        // bus.connect_message(move |_, msg| {
        //     let pipeline = match pipeline_weak.upgrade() {
        //             Some(pipeline) => pipeline,
        //             None => return,
        //         };

        //         match msg.view() {
        //             //  This is called when an End-Of-Stream message is posted on the bus.
        //             // We just set the pipeline to READY (which stops playback).
        //             gstreamer::MessageView::Eos(..) => {
        //                 println!("End-Of-Stream reached.");
        //                 pipeline
        //                     .set_state(gstreamer::State::Ready)
        //                     .expect("Unable to set the pipeline to the `Ready` state");
        //             }

        //             // This is called when an error message is posted on the bus
        //             gstreamer::MessageView::Error(err) => {
        //                 println!(
        //                     "Error from {:?}: {} ({:?})",
        //                     err.get_src().map(|s| s.get_path_string()),
        //                     err.get_error(),
        //                     err.get_debug()
        //                 );
        //             }
        //             // This is called when the pipeline changes states. We use it to
        //             // keep track of the current state.
        //             gstreamer::MessageView::StateChanged(state_changed) => {
        //                 if state_changed
        //                     .get_src()
        //                     .map(|s| s == pipeline)
        //                     .unwrap_or(false)
        //                 {
        //                     println!("State set to {:?}", state_changed.get_current());
        //                 }
        //             }
        //             _ => (),
        //         }
        // });

        // playbin
        //     .set_state(gstreamer::State::Playing)
        //     .expect("Unable to set the playbin to the 'Playing State'");
        
        // bus.remove_signal_watch();
    }

    // We are possibly in a GStreamer working thread, so we notify the main
    // thread of this event through a message in the bus
    fn post_app_message(playbin: &gstreamer::Element) {
        let _ = playbin.post_message(gstreamer::message::Application::new(gstreamer::Structure::new_empty(
            "tags-changed",
        )));
    }

    // Generate a unique `WidgetId` for each widget.
    widget_ids! {
        struct Ids {
            master,
            header,
            body,
            left_column,
            middle_column,
            right_column,
            footer,
            footer_scrollbar,
            floating_a,
            floating_b,
            tabs,
            tab_foo,
            tab_bar,
            tab_baz,

            title,
            subtitle,
            top_left,
            bottom_right,
            foo_label,
            bar_label,
            baz_label,
            button_matrix,
            bing,
            bong,
            // Video Player UI
            video_area,
            game_area,
            video_controls,
            // Video Player Controls
            play_button,
            play_button_canvas,
            pause_button,
            pause_button_canvas,
            video_slider,
            video_slider_canvas,
            video_buttons_canvas,
            video_slider_indicator,
            circle,
            text

        }
    }

    // Video Player Section
    fn add_streams_info(playbin: &gstreamer::Element, stype: &str)
    {
        let propname: &str = &format!("n-{}", stype);
        let signame: &str = &format!("get-{}-tags", stype);

        match playbin.get_property(propname).unwrap().get() {
            Ok(Some(x)) => {
                for i in 0..x {
                    let tags =
                        playbin.emit(signame, &[&i]).unwrap().unwrap();
                    
                    if let Ok(Some(tags)) = tags.get::<gstreamer::TagList>() {
                        println!("{} stream {}:", stype, i);

                        if let Some(codec) = tags.get::<gstreamer::tags::VideoCodec>() {
                            println!("  codec: {}", codec.get().unwrap());
                        }

                        if let Some(codec) = tags.get::<gstreamer::tags::AudioCodec>() {
                            println!("  codec: {}", codec.get().unwrap());
                        }

                        if let Some(lang) = tags.get::<gstreamer::tags::LanguageCode>() {
                            println!("  language: {}", lang.get().unwrap());
                        }

                        if let Some(bitrate) = tags.get::<gstreamer::tags::Bitrate>() {
                            println!("  language: {}", bitrate.get().unwrap());
                        }
                    }
                }
            }
            _ => {
                eprintln!("Could not get {}!", propname);
            }
        }
    }

    fn analize_streams(playbin: &gstreamer::Element)
    {
        add_streams_info(playbin, "video");
        add_streams_info(playbin, "audio");
        add_streams_info(playbin, "text");
    }

    fn create_ui(playbin: &gstreamer::Element) {
        let pipeline = playbin.clone();

    }

    // Buttons Logic
    fn play_button_logic (playbin: &gstreamer::Element, ref mut ui: conrod_core::UiCell, ids: &mut Ids) {
        playbin.set_state(gstreamer::State::Playing)
            .expect("Unable to set pipeline to the 'Playing' state.");
    }

    fn pause_button_logic (playbin: &gstreamer::Element) {
        playbin.set_state(gstreamer::State::Paused)
            .expect("Unable to set pipeline to the 'Paused' state.");
    }

    fn stop_button_logic (playbin: &gstreamer::Element) {
        playbin.set_state(gstreamer::State::Ready)
            .expect("Unable to set pipeline to the 'Stop' state.");
    }


    /// The type upon which we'll implement the `Widget` trait.
    #[derive(WidgetCommon)]
    pub struct CircularButton<'a> {
        /// An object that handles some of the dirty work of rendering a GUI. We don't
        /// really have to worry about it.
        #[conrod(common_builder)]
        common: widget::CommonBuilder,
        /// Optional label string for the button.
        maybe_label: Option<&'a str>,
        /// See the Style struct below.
        style: Style,
        /// Whether the button is currently enabled, i.e. whether it responds to
        /// user input.
        enabled: bool,
    }

    #[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
    pub struct Style {
        /// Color of the button.
        #[conrod(default = "theme.shape_color")]
        pub color: Option<conrod_core::Color>,
        /// Color of the button's label.
        #[conrod(default = "theme.label_color")]
        pub label_color: Option<conrod_core::Color>,
        /// Font size of the button's label.
        #[conrod(default = "theme.font_size_medium")]
        pub label_font_size: Option<conrod_core::FontSize>,
        /// Specify a unique font for the label.
        #[conrod(default = "theme.font_id")]
        pub label_font_id: Option<Option<conrod_core::text::font::Id>>,
    }

    
    impl<'a> CircularButton<'a> {
        /// Create a button context to be built upon.
        pub fn new() -> Self {
            CircularButton {
                common: widget::CommonBuilder::default(),
                style: Style::default(),
                maybe_label: None,
                enabled: true,
            }
        }

        /// Specify the font used for displaying the label.
        pub fn label_font_id(mut self, font_id: conrod_core::text::font::Id) -> Self {
            self.style.label_font_id = Some(Some(font_id));
            self
        }

        /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
        /// other Conrod configs, this returns self for chainability. Allow dead code
        /// because we never call this in the example.
        #[allow(dead_code)]
        pub fn enabled(mut self, flag: bool) -> Self {
            self.enabled = flag;
            self
        }
    }

    pub struct State {
        ids: Ids,
    }

    impl<'a> Widget for CircularButton<'a> {
        /// The State struct that we defined above.
        type State = State;
        /// The Style struct that we defined using the `widget_style!` macro.
        type Style = Style;
        /// The event produced by instantiating the widget.
        ///
        /// `Some` when clicked, otherwise `None`.
        type Event = Option<()>;

        fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
            State {
                ids: Ids::new(id_gen),
            }
        }

        fn style(&self) -> Self::Style {
            self.style.clone()
        }

        /// Optionally specify a function to use for determining whether or not a point is over a
        /// widget, or if some other widget's function should be used to represent this widget.
        ///
        /// This method is optional to implement. By default, the bounding rectangle of the widget
        /// is used.
        fn is_over(&self) -> widget::IsOverFn {
            use conrod_core::graph::Container;
            use conrod_core::Theme;
            fn is_over_widget(widget: &Container, _: Point, _: &Theme) -> widget::IsOver {
                let unique = widget.state_and_style::<State, Style>().unwrap();
                unique.state.ids.circle.into()
            }
            is_over_widget
        }

        /// Update the state of the button by handling any input that has occurred since the last
        /// update.
        fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
            let widget::UpdateArgs {
                id,
                state,
                rect,
                ui,
                style,
                ..
            } = args;

            let (color, event) = {
                let input = ui.widget_input(id);

                // If the button was clicked, produce `Some` event.
                let event = input.clicks().left().next().map(|_| ());
                for ev in input.presses() {
                }
                

                let color = style.color(&ui.theme);
                let color = input.mouse().map_or(color, |mouse| {
                    if mouse.buttons.left().is_down() {
                        color.clicked()
                    } else {
                        color.highlighted()
                    }
                });

                (color, event)
            };

            // Finally, we'll describe how we want our widget drawn by simply instantiating the
            // necessary primitive graphics widgets.
            //
            // Conrod will automatically determine whether or not any changes have occurred and
            // whether or not any widgets need to be re-drawn.
            //
            // The primitive graphics widgets are special in that their unique state is used within
            // conrod's backend to do the actual drawing. This allows us to build up more complex
            // widgets by using these simple primitives with our familiar layout, coloring, etc
            // methods.
            //
            // If you notice that conrod is missing some sort of primitive graphics that you
            // require, please file an issue or open a PR so we can add it! :)

            // First, we'll draw the **Circle** with a radius that is half our given width.
            let radius = rect.w() / 2.0;
            widget::Circle::fill(radius)
                .middle_of(id)
                .graphics_for(id)
                .color(color)
                .set(state.ids.circle, ui);

            // Now we'll instantiate our label using the **Text** widget.
            if let Some(ref label) = self.maybe_label {
                let label_color = style.label_color(&ui.theme);
                let font_size = style.label_font_size(&ui.theme);
                let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
                widget::Text::new(label)
                    .and_then(font_id, widget::Text::font_id)
                    .middle_of(id)
                    .font_size(font_size)
                    .graphics_for(id)
                    .color(label_color)
                    .set(state.ids.text, ui);
            }

            event
        }
    }


    impl<'a> Labelable<'a> for CircularButton<'a> {
        fn label(mut self, text: &'a str) -> Self {
            self.maybe_label = Some(text);
            self
        }
        fn label_color(mut self, color: conrod_core::Color) -> Self {
            self.style.label_color = Some(color);
            self
        }
        fn label_font_size(mut self, size: conrod_core::FontSize) -> Self {
            self.style.label_font_size = Some(size);
            self
        }
    }

}


