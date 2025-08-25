import {Response} from './Response';

export interface User {
  id: number;
  twitch_id: number;
  display_name: string;
  login_name: string;
}

export interface UserResponse extends Response<User[]> {}
