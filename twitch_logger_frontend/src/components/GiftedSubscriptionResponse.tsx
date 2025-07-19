import { GiftedSubscription } from "../types/Subscriptions";
import { Column, DataTable } from "./DataTable";

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
    <div className="gifted_subscriptions_data_table">
      {props.gifted_subscriptions && (
        <DataTable
          data={props.gifted_subscriptions}
          columns={giftedSubscriptionColumns}
          rowKey="id"
          emptyMessage="No gifted subscriptions found."
        />
      )}
    </div>
  );
}
