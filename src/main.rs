// #![allow(unused)]
use glib::prelude::*;
use gstreamer::prelude::*;
use gtk::prelude::*;
use gtk::{Inhibit, Window, WindowType};
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Debug, Msg)]
enum MainAppMessage {
    Quit,
}

struct MainAppModel {
    video_source: Option<String>,
    pipeline: gstreamer::Pipeline,
}

struct MainAppWindow {
    model: MainAppModel,
    window: Window,
}

impl Update for MainAppWindow {
    type Model = MainAppModel;
    type ModelParam = ();
    type Msg = MainAppMessage;

    // Return the initial model.
    fn model(_: &Relm<Self>, _: ()) -> Self::Model {
        gstreamer::init().expect("Failed to initialize gstreamer!");

        Self::Model {
            video_source: Some("v4l2src".into()),
            pipeline: gstreamer::Pipeline::new(Some("Video Pipeline".into())),
        }
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Self::Msg) {
        match dbg!(event) {
            Self::Msg::Quit => {
                gtk::main_quit();
            }
        }
    }
}

impl Widget for MainAppWindow {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    // Create the widgets.
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // Create video source from Video 4 Linux webcam driver or default if not available
        let src = gstreamer::ElementFactory::make(
            &model
                .video_source
                .clone()
                .unwrap_or("videotestsrc".to_string()),
            None,
        )
        .unwrap();

        // create video sink widget
        let (sink, video_widget) =
            if let Ok(gtkglsink) = gstreamer::ElementFactory::make("gtkglsink", None) {
                println!("Using OpenGL acceleration");
                let glsinkbin = gstreamer::ElementFactory::make("glsinkbin", None).unwrap();
                glsinkbin.set_property("sink", &gtkglsink);
                let video_widget = gtkglsink.property::<gtk::Widget>("widget");
                (glsinkbin, video_widget)
            } else {
                println!("Using software fallback");
                let sink = gstreamer::ElementFactory::make("gtksink", None).unwrap();
                let video_widget = sink.property::<gtk::Widget>("widget");
                (sink, video_widget)
            };

        model.pipeline.add_many(&[&src, &sink]).unwrap();
        src.link(&sink).unwrap();

        // GTK+ widgets are used normally within a `Widget`.
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(320, 240);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add(&video_widget);

        let button = gtk::Button::new();

        vbox.add(&button);

        window.add(&vbox);
        window.show_all();

        if let Err(err) = model.pipeline.set_state(gstreamer::State::Playing) {
            println!("{:?}", err);
        }

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Self::Msg::Quit), Inhibit(false))
        );

        MainAppWindow { model, window }
    }
}

fn main() {
    MainAppWindow::run(()).unwrap();
}
