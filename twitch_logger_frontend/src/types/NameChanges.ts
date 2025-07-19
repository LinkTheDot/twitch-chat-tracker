import { Response } from "./Response";

export interface NameChange {
  twitch_user_twitch_id: number;
  previous_login_name: string;
  new_login_name: string;
  created_at: string;
}

export enum UserRequestType {
  TwitchId,
  Name,
}

// Takes a UserRequestType and the identifier for how to get the user(s).
//
// If you want to request multiple users, separate the values by a comma (,).
export interface NameChangeRequest {
  requestType: UserRequestType,
  userIdentifier: string;
}

export interface NameChangeResponse extends Response<NameChange[]> { }
