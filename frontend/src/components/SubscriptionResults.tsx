import { getSubscriptions } from "../services/Subscriptions";
import { SubscriptionRequest, SubscriptionRequestType } from "../types/Subscriptions";
import { GiftedSubscriptionResults } from "./GiftedSubscriptionResponse";
import { QueryFormData } from "./QueryForm";
import { UserSubscriptionResults } from "./UserSubscriptionResults";

export interface SubscriptionResultsProps {
  queryResults: QueryFormData;
}

export function SubscriptionResults(props: SubscriptionResultsProps) {
  const userRequestType = Number(props.queryResults.userSearchQuery) ? SubscriptionRequestType.TwitchId : SubscriptionRequestType.Name;
  
  const subscriptionRequest: SubscriptionRequest = {
    userRequestType,
    userIdentifier: props.queryResults.userSearchQuery,
    channel: props.queryResults.channelSearchQuery,
  };

  const subscriptionResponse = getSubscriptions(subscriptionRequest);

  if (subscriptionResponse.isLoading) {
    return <div className="nondata_message">Loading users...</div>;
  }

  if (subscriptionResponse.error) {
    return <div className="nondata_message">Error: {subscriptionResponse.error.message || "Failed to fetch users."}</div>;
  }

  return (
    <div>
      {subscriptionResponse.value?.subscriptions &&
        <UserSubscriptionResults
          subscriptions={subscriptionResponse.value.subscriptions}
        />
      }

      {subscriptionResponse.value?.gifted_subscriptions &&
        <GiftedSubscriptionResults
          gifted_subscriptions={subscriptionResponse.value.gifted_subscriptions}
        />
      }
    </div>
  );
}
