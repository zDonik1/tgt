use crate::action::Action;
use crate::app_context::AppContext;
use crate::component_name::ComponentName::Prompt;
use crate::components::component_traits::{Component, HandleFocus};
use crate::event::Event;
use crate::tg::message_entry::MessageEntry;
use ratatui::layout::Rect;
use ratatui::symbols::border::PLAIN;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::block::{Block, Title};
use ratatui::widgets::Borders;
use ratatui::widgets::{List, ListDirection, ListState};
use ratatui::Frame;
use std::sync::Arc;
use tdlib_rs::enums::{ChatList, UserStatus};
use tdlib_rs::types::User;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ChatListEntry {
    chat_id: i64,
    chat_name: String,
    last_message: Option<MessageEntry>,
    user: Option<User>,
    is_marked_as_unread: bool,
    unread_count: i32,
    /// Identifier of the last read incoming message
    last_read_inbox_message_id: Option<i64>,
    /// Identifier of the last read outgoing message
    last_read_outbox_message_id: Option<i64>,
}
impl Default for ChatListEntry {
    fn default() -> Self {
        Self::new()
    }
}
impl ChatListEntry {
    pub fn new() -> Self {
        Self {
            chat_id: 0,
            chat_name: String::new(),
            last_message: None,
            user: None,
            is_marked_as_unread: false,
            unread_count: 0,
            last_read_inbox_message_id: None,
            last_read_outbox_message_id: None,
        }
    }

    pub fn set_chat_id(&mut self, chat_id: i64) {
        self.chat_id = chat_id;
    }
    pub fn set_chat_name(&mut self, chat_name: String) {
        self.chat_name = chat_name;
    }
    pub fn set_last_message(&mut self, last_message: MessageEntry) {
        self.last_message = Some(last_message);
    }
    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }
    pub fn set_is_marked_as_unread(&mut self, is_marked_as_unread: bool) {
        self.is_marked_as_unread = is_marked_as_unread;
    }
    pub fn set_unread_count(&mut self, unread_count: i32) {
        self.unread_count = unread_count;
    }
    pub fn set_last_read_inbox_message_id(&mut self, last_read_inbox_message_id: i64) {
        self.last_read_inbox_message_id = Some(last_read_inbox_message_id);
    }
    pub fn set_last_read_outbox_message_id(&mut self, last_read_outbox_message_id: i64) {
        self.last_read_outbox_message_id = Some(last_read_outbox_message_id);
    }

    fn get_text_styled(&self, app_context: &AppContext) -> Text {
        let mut online_symbol = "";
        let mut verificated_symbol = "";
        if let Some(user) = &self.user {
            online_symbol = match user.status {
                UserStatus::Online(_) => "🟢 ",
                UserStatus::Offline(_) => "",
                UserStatus::Empty => "",
                UserStatus::Recently(_) => "",
                UserStatus::LastWeek(_) => "",
                UserStatus::LastMonth(_) => "",
            };
            verificated_symbol = if user.is_verified { "✅" } else { "" };
        }
        let unread_info = if self.is_marked_as_unread {
            format!("({})", self.unread_count)
        } else {
            "".to_string()
        };

        let preview_lines = -1;
        let mut entry = Text::default();
        entry.extend(vec![Line::from(vec![
            Span::raw(online_symbol),
            Span::styled(
                self.chat_name.clone(),
                app_context.style_chat_list_item_chat_name(),
            ),
            Span::raw(" "),
            Span::styled(
                unread_info,
                app_context.style_chat_list_item_unread_counter(),
            ),
            Span::raw(" "),
            Span::raw(verificated_symbol),
            Span::raw(" | "),
            self.last_message.as_ref().map_or_else(Span::default, |e| {
                e.timestamp().get_span_styled(app_context)
            }),
        ])]);
        entry.extend(self.last_message.as_ref().map_or_else(Line::default, |e| {
            e.get_lines_styled_with_style(
                app_context.style_chat_list_item_message_content(),
                preview_lines,
            )[0]
            .clone()
        }));

        entry
    }
}
/// `ChatListWindow` is a struct that represents a window for displaying a list
/// of chat items. It is responsible for managing the layout and rendering of
/// the chat list.
pub struct ChatListWindow {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `ChatListWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A list of chat items to be displayed in the `ChatListWindow`.
    chat_list: Vec<ChatListEntry>,
    /// The state of the list.
    chat_list_state: ListState,
    /// Indicates whether the `ChatListWindow` is focused or not.
    focused: bool,
}
/// Implementation of the `ChatListWindow` struct.
impl ChatListWindow {
    /// Create a new instance of the `ChatListWindow` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `ChatListWindow` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let name = "".to_string();
        let command_tx = None;
        let chat_list = vec![];
        let chat_list_state = ListState::default();
        let focused = false;

        ChatListWindow {
            app_context,
            name,
            command_tx,
            chat_list,
            chat_list_state,
            focused,
        }
    }
    /// Set the name of the `ChatListWindow`.
    ///
    /// # Arguments
    /// * `name` - The name of the `ChatListWindow`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ChatListWindow`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
    /// Select the next chat item in the list.
    fn next(&mut self) {
        let i = match self.chat_list_state.selected() {
            Some(i) => {
                if i == self.chat_list.len() / 2 {
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx
                            .send(Event::LoadChats(ChatList::Main.into(), 20))
                            .unwrap();
                    }
                }

                if i >= self.chat_list.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.chat_list_state.select(Some(i));
    }
    /// Select the previous chat item in the list.
    fn previous(&mut self) {
        let i = match self.chat_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.chat_list_state.select(Some(i));
    }
    /// Unselect the chat item in the list.
    fn unselect(&mut self) {
        self.chat_list_state.select(None);
    }
    /// Confirm the selection of the chat item in the list.
    fn confirm_selection(&mut self) {
        if let Some(i) = self.chat_list_state.selected() {
            if let Some(chat) = self.chat_list.get(i) {
                self.app_context
                    .tg_context()
                    .set_open_chat_user(chat.user.clone());
                self.app_context.tg_context().set_open_chat_id(chat.chat_id);
                self.app_context.tg_context().clear_open_chat_messages();
                self.app_context
                    .action_tx()
                    .send(Action::FocusComponent(Prompt))
                    .unwrap();

                if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                    // Load chat history
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // event_tx.send(Event::PrepareChatHistory).unwrap();
                    // sleep(Duration::from_millis(100));
                    event_tx.send(Event::GetChatHistory).unwrap();

                    // Mark all unread messages as read
                    event_tx.send(Event::ViewAllMessages).unwrap();
                }
            }
        }
    }
}

/// Implement the `HandleFocus` trait for the `ChatListWindow` struct.
/// This trait allows the `ChatListWindow` to be focused or unfocused.
impl HandleFocus for ChatListWindow {
    /// Set the `focused` flag for the `ChatListWindow`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `ChatListWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for ChatListWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.command_tx = Some(tx.clone());
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ChatListNext => self.next(),
            Action::ChatListPrevious => self.previous(),
            Action::ChatListUnselect => self.unselect(),
            Action::ChatListOpen => self.confirm_selection(),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> std::io::Result<()> {
        let style_border_focused = if self.focused {
            self.app_context.style_border_component_focused()
        } else {
            self.app_context.style_chat_list()
        };
        if let Ok(Some(items)) = self.app_context.tg_context().get_chats_index() {
            self.chat_list = items;
        }
        let items = self
            .chat_list
            .iter()
            .map(|item| item.get_text_styled(&self.app_context));
        let block = Block::default()
            .border_set(PLAIN)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title(Title::from(self.name.as_str()));

        let list = List::new(items)
            .block(block)
            .style(self.app_context.style_chat_list())
            .highlight_style(self.app_context.style_chat_list_item_selected())
            .direction(ListDirection::TopToBottom);
        // .highlight_symbol("➤ ")
        // .repeat_highlight_symbol(true)

        frame.render_stateful_widget(list, area, &mut self.chat_list_state);
        Ok(())
    }
}
