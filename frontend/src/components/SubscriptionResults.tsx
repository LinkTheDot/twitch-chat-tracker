import { useGetData } from "../services/DataRequest";
import { buildFetchUrl } from "../services/FetchUrl";
import { Pagination } from "../types/Pagination";
import { QueryFormData } from "../types/QueryFormData";
import { Subscriptions } from "../types/Subscriptions";
import { GiftedSubscriptionResults } from "./GiftedSubscriptionResponse";
import { UserSubscriptionResults } from "./UserSubscriptionResults";

export interface SubscriptionResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
}

export function SubscriptionResults(props: SubscriptionResultsProps) {
  if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
    return;
  }

  const userIdentifier = props.queryResults.userSearchQuery;
  const requestType = Number(userIdentifier) ? "twitch_id" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/donations/subscriptions",
    channel: props.queryResults.channelSearchQuery,
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
  });

  const { response_data, error, isLoading } = useGetData<Subscriptions>({ requestUrl, updatePagination: props.updatePagination });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
        <span className="ml-3 text-gray-400">Loading subscriptions...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch subscriptions."}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {response_data?.data.subscriptions &&
        <UserSubscriptionResults
          subscriptions={response_data.data.subscriptions}
        />
      }

      {response_data?.data.gifted_subscriptions &&
        <GiftedSubscriptionResults
          gifted_subscriptions={response_data.data.gifted_subscriptions}
        />
      }
    </div>
  );
}
