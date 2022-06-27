use crate::client::ClientInfo;
use crate::ui::{UIMessage, UIMessageSender};
use gtk::{self, glib};

pub fn init_ui(
    ui_message_sender: Option<glib::Sender<UIMessage>>,
    client_info: &mut ClientInfo,
) -> UIMessageSender {
    let ui_message_sender = match ui_message_sender {
        Some(sender) => UIMessageSender::with_ui(&client_info.metainfo.info.name, sender),
        None => UIMessageSender::no_ui(),
    };
    ui_message_sender.send_metadata(client_info.metainfo.clone());
    ui_message_sender
}
