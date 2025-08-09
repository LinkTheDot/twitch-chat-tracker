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
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
        <span className="ml-3 text-gray-400">Loading subscriptions...</span>
      </div>
    );
  }

  if (subscriptionResponse.error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {subscriptionResponse.error.message || "Failed to fetch subscriptions."}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
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
