import { User } from "./users";

export interface UserMessageResponse {
  user: User;
  channel: User;

  messages: UserMessage[]
}

export interface UserMessage {
  id: number,
  is_first_message: boolean,
  timestamp: string,
  contents: string,
  is_subscriber: boolean,
  emote_usage: Emote[],
}

export interface Emote {
  contents_indices: number[],
  emote_name_size: number,
  emote_image_url: string,
}
