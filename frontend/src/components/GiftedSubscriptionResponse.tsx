import { GiftedSubscription } from "../types/Subscriptions";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";

export interface GiftedSubscriptionResultsProps {
  gifted_subscriptions: GiftedSubscription[];
}

export function GiftedSubscriptionResults(props: GiftedSubscriptionResultsProps) {
  const giftedSubscriptionColumns: Column<GiftedSubscription>[] = [
    { header_name: 'Id', header_value_key: 'id' },
    { header_name: 'Months', header_value_key: 'recipient_months_subscribed' },
    {
      header_name: 'Timestamp',
      render: (item) => item.donation_event.timestamp
    },
    {
      header_name: 'Channel Name',
      render: (item) => item.donation_event.donation_receiver.login_name
    },
    {
      header_name: 'Recipient Name',
      render: (item) => item.recipient_twitch_user.login_name
    },
    {
      header_name: 'Donator Name',
      render: (item) => item.donation_event.donator.login_name
    },
    {
      header_name: 'Subscription Tier',
      render: (item) => item.donation_event.subscription_tier
    },
  ];

  return (
    <>
      <h3 className="text-xl font-semibold text-gray-200 mb-4">Gifted Subscriptions</h3>
      {props.gifted_subscriptions && (
        <ResponsiveDataDisplay
          data={props.gifted_subscriptions}
          columns={giftedSubscriptionColumns}
          rowKey="id"
          emptyMessage="No gifted subscriptions found."
        />
      )}
    </>
  );
}
