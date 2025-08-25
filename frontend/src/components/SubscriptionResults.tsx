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
  setIsLoading: (isLoading: boolean) => void;
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

  const { response_data, error } = useGetData<Subscriptions>({
    requestUrl,
    updatePagination: props.updatePagination,
    setIsLoading: props.setIsLoading,
  });

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
