import {Response} from './Response';

export interface Follow {
  id: number;
  displayName: string;
  login: string;
  avatar: string;
  followedAt: string;
}

export interface FollowingRequest {
  userLogin: string;
}

export interface FollowingResponse extends Response<Follow[]> {}

