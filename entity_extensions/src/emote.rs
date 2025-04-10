use entities::emote;
use sea_orm::*;

#[derive(Debug)]
struct EmoteData<'a> {
  twitch_id: &'a str,
  name: String,
  /// List of the start and end of an emote in a user's message.
  positions: Vec<(usize, usize)>,
}

pub trait EmoteExtensions {
  /// The list is taken as Twitch's IRC client provides.
  /// Formatted as: `emote_id:0-1,2-3/`
  ///
  /// Returned as a list of the emote models and the positions they were used in the message.
  async fn get_or_set_list(
    message_contents: &str,
    emote_list: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, Vec<(usize, usize)>)>, DbErr>;
}

impl EmoteExtensions for emote::Model {
  async fn get_or_set_list(
    message_contents: &str,
    emote_list: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, Vec<(usize, usize)>)>, DbErr> {
    if emote_list.is_empty() {
      return Ok(vec![]);
    }

    let emote_data = parse_emotes(message_contents, emote_list);
    let mut emote_models = vec![];

    for emote in emote_data {
      let emote_model = emote::Entity::find()
        .filter(emote::Column::TwitchId.eq(emote.twitch_id))
        .one(database_connection)
        .await?;

      if let Some(emote_model) = emote_model {
        emote_models.push((emote_model, emote.positions));

        continue;
      }

      let emote_active_model = emote::ActiveModel {
        twitch_id: ActiveValue::Set(emote.twitch_id.to_owned()),
        name: ActiveValue::Set(emote.name),
        ..Default::default()
      };

      let emote_model = emote_active_model.insert(database_connection).await?;

      emote_models.push((emote_model, emote.positions));
    }

    Ok(emote_models)
  }
}

/// The list is formatted as: `emote_id:0-1,2-3/`
fn parse_emotes<'a>(message_contents: &str, emote_list: &'a str) -> Vec<EmoteData<'a>> {
  let emote_list = emote_list.split('/');

  emote_list.filter_map(|emote_usage| {
    let mut emote_usage = emote_usage.split(':');
    let emote_id = emote_usage.next()?;
    let emote_positions: Vec<(usize, usize)> = emote_usage
      .next()?
      .split(',')
      .filter_map(|emote_positions| {
        let mut positions = emote_positions.split('-');
        let start = positions.next()?.parse::<usize>().ok()?;
        let end = positions.next()?.parse::<usize>().ok()?;

        Some((start, end))
      })
      .collect();
    let (emote_name_start, emote_name_end) = *emote_positions.first()?;

    if message_contents.chars().count() < emote_name_end {
      tracing::error!("Encountered a message where the emote position provided is larger than the message length. Message contents: {:?} | Emote range: {}-{}", message_contents, emote_name_start, emote_name_end);
      return None;
    }

    let emote_name: String = message_contents
      .chars()
      .skip(emote_name_start)
      .take(emote_name_end - emote_name_start + 1)
      .collect();

    Some(EmoteData {
      twitch_id: emote_id,
      name: emote_name,
      positions: emote_positions,
    })
  }).collect()
}
