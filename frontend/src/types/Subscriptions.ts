import { Response } from "./Response";
import { User } from "./users";

export interface Subscriptions {
  subscriptions: UserSubscription[];
  gifted_subscriptions: GiftedSubscription[];
}

export interface UserSubscription {
  id: number;
  months_subscribed: number;
  timestamp: string;
  channel: User;
  subscriber: User;
  subscription_tier: number;
}

export interface GiftedSubscription {
  id: number;
  recipient_months_subscribed: number;
  recipient_twitch_user: User;
  donation_event: DonationEvent;
}

export interface DonationEvent {
  id: number;
  timestamp: string;
  donator: User;
  subscription_tier: number
  donation_receiver: User;
}

export enum SubscriptionRequestType {
  TwitchId,
  Name,
}

export interface SubscriptionRequest {
  userRequestType: SubscriptionRequestType,
  userIdentifier: string;
  channel: string
}

export interface SubscriptionResponse extends Response<Subscriptions> { }
