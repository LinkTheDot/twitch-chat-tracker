import {Response} from './Response';

export interface User {
  id: number;
  twitch_id: number;
  display_name: string;
  login_name: string;
}

export enum UserRequestType {
  TwitchId,
  Name,
}

// Takes a UserRequestType and the identifier for how to get the user(s).
//
// If you want to request multiple users, separate the values by a comma (,).
export interface UserRequest {
  userRequestType: UserRequestType,
  userIdentifier: string;
}

export interface UserResponse extends Response<User[]> {}
