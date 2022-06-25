use super::general_information_tab::*;
use super::UIMessage;
use gtk;
use gtk::prelude::*;
use gtk::Widget;

pub struct Notebook {
    pub notebook: gtk::Notebook,
    pub general_information_tab: GeneralInformationTab,
}

#[derive(Debug)]
pub enum NotebookError {
    Error(&'static str),
    ErrorString(String),
}

impl std::convert::From<GeneralInformationTabError> for NotebookError {
    fn from(error: GeneralInformationTabError) -> Self {
        NotebookError::ErrorString(format!("{:?}", error))
    }
}

impl std::convert::From<gtk::Widget> for NotebookError {
    fn from(widget: gtk::Widget) -> Self {
        NotebookError::ErrorString(format!("could not get widget {}", widget))
    }
}

impl Notebook {
    pub fn new(window: &gtk::ApplicationWindow) -> Notebook {
        let general_information_tab = GeneralInformationTab::new(window);
        let notebook = Notebook {
            notebook: gtk::Notebook::new(),
            general_information_tab,
        };

        let label = gtk::Label::new(Some("General Information"));
        notebook.notebook.append_page(
            &notebook
                .general_information_tab
                .container
                .clone()
                .upcast::<Widget>(),
            Some(&label),
        );
        notebook
    }

    pub fn update(&mut self, message: UIMessage) -> Result<(), NotebookError> {
        self.general_information_tab.update(&message)?;

        Ok(())
    }

    pub fn create_tab(&mut self, title: &str, widget: &gtk::Widget) -> u32 {
        let label = gtk::Label::new(Some(title));
        self.notebook.append_page(widget, Some(&label))
    }
}
