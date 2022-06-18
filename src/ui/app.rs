use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use log::*;
// import channels library
use crate::metainfo::Metainfo;
use glib::{clone, Continue, PRIORITY_DEFAULT};
use gtk::{self, glib};
use std::sync::mpsc::Sender;

pub struct UIMessageSender {
    tx: Option<glib::Sender<UIMessage>>,
}

impl UIMessageSender {
    pub fn no_ui() -> Self {
        UIMessageSender { tx: None }
    }

    pub fn with_ui(tx: glib::Sender<UIMessage>) -> Self {
        UIMessageSender { tx: Some(tx) }
    }

    pub fn send_metadata(&self, metainfo: Metainfo) {
        self.send_message_to_ui(UIMessage::Metainfo(metainfo))
    }

    fn send_message_to_ui(&self, message: UIMessage) {
        if let Some(tx) = &self.tx {
            if tx.send(message).is_err() {
                error!("Failed to send message to UI");
            }
        }
    }
}

pub fn run_ui(client_sender: Sender<glib::Sender<UIMessage>>) {
    let app = Application::builder()
        .application_id("org.gtk-rs.example")
        .build();

    app.connect_activate(move |app| {
        build_ui(app, &client_sender);
    });

    let args: Vec<String> = vec![]; // necessary to not use main program args
    app.run_with_args(&args);
}

fn build_ui(app: &Application, client_sender: &Sender<glib::Sender<UIMessage>>) {
    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let (tx_messages, rx_messages) = glib::MainContext::channel(PRIORITY_DEFAULT);

    client_sender.send(tx_messages).unwrap();

    rx_messages.attach(
        None,
        clone!(@weak button => @default-return Continue(false),
                    move |msg| {
                        match msg {
                            UIMessage::Metainfo(metainfo) => {
                                button.set_label(&metainfo.info.name);
                            }
                        }
                        Continue(true)
                    }
        ),
    );
    let mut notebook = Notebook::new();
    notebook.create_tab("General Information", button.upcast());
    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    gtk_box.append(&notebook.notebook);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .default_width(800)
        .default_height(800)
        .child(&gtk_box)
        .build();

    // Present window
    window.present();
}

pub enum UIMessage {
    Metainfo(Metainfo),
}

pub struct Notebook {
    pub notebook: gtk::Notebook,
    pub tabs: Vec<gtk::Box>,
}

impl Notebook {
    pub fn new() -> Notebook {
        Notebook {
            notebook: gtk::Notebook::new(),
            tabs: Vec::new(),
        }
    }

    pub fn create_tab(&mut self, title: &str, widget: gtk::Widget) -> u32 {
        let label = gtk::Label::new(Some(title));
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let index = self.notebook.append_page(&widget, Some(&label));

        self.tabs.push(tab);

        index
    }
}
