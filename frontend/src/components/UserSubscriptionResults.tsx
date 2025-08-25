import { UserSubscription } from "../types/Subscriptions";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";

export interface UserSubscriptionResultsProps {
  subscriptions: UserSubscription[],
}

export function UserSubscriptionResults(props: UserSubscriptionResultsProps) {
  const userSubscriptionColumns: Column<UserSubscription>[] = [
    { header_name: 'Id', header_value_key: 'id' },
    { header_name: 'Months', header_value_key: 'months_subscribed' },
    { header_name: 'Timestamp', header_value_key: 'timestamp' },
    {
      header_name: 'Channel Name',
      render: (item) => item.channel.login_name
    },
    {
      header_name: 'Subscriber Name',
      render: (item) => item.subscriber.login_name
    },
    { header_name: 'Subscription Tier', header_value_key: 'subscription_tier' },
  ];

  return (
    <>
      <h3 className="text-xl font-semibold text-gray-200 mb-4">User Subscriptions</h3>
      {props.subscriptions && (
        <ResponsiveDataDisplay
          data={props.subscriptions}
          columns={userSubscriptionColumns}
          rowKey="id"
          emptyMessage="No user subscriptions found."
        />
      )}
    </>
  );
}
