mod email_folder_pane;
pub use email_folder_pane::EmailFolderPane;

mod email_list_pane;
pub use email_list_pane::EmailListPane;

mod email_view;
pub use email_view::{EmailAvatar, EmailView};

mod reader_pane;
pub use reader_pane::ReaderPane;

mod split_panes;
pub use split_panes::{Axis, SplitPanes};

mod settings_dialog;
pub use settings_dialog::{SettingsDialog, show_settings_dialog};

mod add_email_account_screen;
pub use add_email_account_screen::AddEmailAccountScreen;
