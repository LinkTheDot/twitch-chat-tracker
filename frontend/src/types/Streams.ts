import { User } from "./users";

export interface Streams {
  user: User;
  streams: Stream[];
}

export interface Stream {
  id: number;
  twitch_stream_id: number;
  start_timestamp: string;
  end_timestamp: string;
}
