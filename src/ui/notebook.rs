use super::download_statistics_tab::*;
use super::general_information_tab::*;
use super::UIMessage;
use gtk;
use gtk::prelude::*;
use gtk::Widget;

pub struct Notebook {
    pub notebook: gtk::Notebook,
    pub general_information_tab: GeneralInformationTab,
    pub download_statistics_tab: DownloadStatisticsTab,
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

impl std::convert::From<DownloadStatisticsTabError> for NotebookError {
    fn from(error: DownloadStatisticsTabError) -> Self {
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
        let notebook = Notebook {
            notebook: gtk::Notebook::new(),
            general_information_tab: GeneralInformationTab::new(window),
            download_statistics_tab: DownloadStatisticsTab::new(window),
        };

        Self::create_tab(
            "General Information",
            &notebook.general_information_tab.container,
            &notebook.notebook,
        );
        Self::create_tab(
            "Download Statistics",
            &notebook.download_statistics_tab.container,
            &notebook.notebook,
        );
        notebook
    }

    pub fn update(&mut self, message: UIMessage) -> Result<(), NotebookError> {
        self.general_information_tab.update(&message)?;
        self.download_statistics_tab.update(&message)?;
        Ok(())
    }

    pub fn create_tab(title: &str, container: &gtk::Box, notebook: &gtk::Notebook) -> u32 {
        let label = gtk::Label::new(Some(title));
        notebook.append_page(&container.clone().upcast::<Widget>(), Some(&label))
    }
}
