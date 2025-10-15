use crate::{conditions::query_conditions::AppQueryConditions, errors::AppError};
use entities::{emote, emote_usage};
use sea_orm::*;
use std::collections::HashMap;

pub async fn get_top_n_emotes_table(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
  amount: Option<usize>,
) -> Result<String, AppError> {
  let top_emotes = get_top_n_emotes(query_conditions, database_connection, amount).await?;
  let top_emotes_table = build_emote_ranking_table(top_emotes);

  Ok(top_emotes_table)
}

async fn get_top_n_emotes(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
  amount: Option<usize>,
) -> Result<Vec<(String, usize)>, AppError> {
  let emotes_used: Vec<(emote_usage::Model, Option<emote::Model>)> = emote_usage::Entity::find()
    .join(
      JoinType::LeftJoin,
      emote_usage::Relation::StreamMessage.def(),
    )
    .filter(query_conditions.messages().clone()) // Filter for the messages wanted because it's joined
    .find_also_related(emote::Entity)
    .all(database_connection)
    .await?;

  let emotes_used_totals: HashMap<String, usize> =
    emotes_used
      .into_iter()
      .fold(HashMap::new(), |mut usage_totals, (emote_usage, emote)| {
        let Some(emote) = emote else {
          tracing::error!(
            "Failed to get an emote from an emote_usage relation. Message id: {} | Emote id: {}",
            emote_usage.stream_message_id,
            emote_usage.emote_id,
          );
          return usage_totals;
        };

        let entry = usage_totals.entry(emote.name).or_default();
        *entry += emote_usage.usage_count as usize;

        usage_totals
      });

  let mut emote_uses: Vec<(String, usize)> = emotes_used_totals.into_iter().collect();
  emote_uses.sort_by(|(_, uses_lhs), (_, uses_rhs)| uses_rhs.cmp(uses_lhs));

  let amount = amount.unwrap_or(emote_uses.len());

  Ok(emote_uses.into_iter().take(amount).collect())
}

fn build_emote_ranking_table(top_emotes: Vec<(String, usize)>) -> String {
  let longest_emote_name = top_emotes
    .iter()
    .map(|(emote_name, _)| emote_name.chars().count())
    .max()
    .unwrap();
  let title = format!("= Top {} Emotes Used =", top_emotes.len());
  let emote_rankings_max_digits = number_of_digits(top_emotes.len());

  let top_emotes_string = top_emotes
    .into_iter()
    .enumerate()
    .map(|(rank, (emote_name, use_count))| {
      let rank = rank + 1;
      let emote_name_length = emote_name.chars().count();
      let usage_padding = " ".repeat(longest_emote_name - emote_name_length);
      let rank_padding = " ".repeat(emote_rankings_max_digits - number_of_digits(rank));

      format!("{rank}:{rank_padding} {emote_name}{usage_padding} - {use_count}")
    })
    .collect::<Vec<String>>()
    .join("\n");

  format!("{title}\n{top_emotes_string}")
}

/// Counts the amount of digits in the passed in number.
fn number_of_digits(n: usize) -> usize {
  if n == 0 {
    1
  } else {
    (n.ilog10() + 1) as usize
  }
}
