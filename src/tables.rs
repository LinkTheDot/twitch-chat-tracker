use crate::errors::AppError;
use crate::irc_chat::message::MessageData;
use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Tabled)]
pub struct Entry<'a> {
  pub place: usize,
  pub name: &'a str,
  pub total_chats: usize,
  pub chat_percentage: f32,
}

impl<'a> Entry<'a> {
  pub fn from_message_list(
    mut message_list: Vec<Vec<&'a MessageData>>,
  ) -> Result<Vec<Self>, AppError> {
    let total_chats = message_list
      .iter()
      .map(|messages| messages.len())
      .sum::<usize>() as f32;
    message_list.sort_by_key(|messages| messages.len());

    let mut entries = vec![];

    for (place, messages) in message_list.iter().enumerate() {
      let Some(name) = messages.first().map(|message| &message.user) else {
        return Err(AppError::MissingUserMessages);
      };
      let user_chats = messages.len();

      let entry = Entry {
        place,
        name,
        total_chats: user_chats,
        chat_percentage: user_chats as f32 / total_chats,
      };

      entries.push(entry);
    }

    Ok(entries)
  }

  pub fn table(entries: Vec<Self>) -> Table {
    let mut table = Table::new(entries);

    table.with(Style::markdown());

    table
  }
}
