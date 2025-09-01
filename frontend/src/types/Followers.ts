import { User } from './users';

export interface Follows {
  forUser?: User;
  follows: Follow[];
}

export interface Follow {
  id: string;
  displayName: string;
  login: string;
  avatar: string;
  followedAt: string;
}
