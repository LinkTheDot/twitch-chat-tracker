import { useGetData } from "../services/DataRequest";
import { buildFetchUrl } from "../services/FetchUrl";
import { Follow, Follows } from "../types/Followers";
import { Pagination } from "../types/Pagination";
import { QueryFormData } from "../types/QueryFormData";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";

export interface FollowingResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
}

export function FollowingResults(props: FollowingResultsProps) {
  if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
    return;
  }

  const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(userIdentifier) ? "user_id" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/users/following",
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
  });

  const { response_data, error, isLoading } = useGetData<Follows>({ requestUrl, updatePagination: props.updatePagination });

  const followingColumns: Column<Follow>[] = [
    { header_name: 'Twitch ID', header_value_key: 'id' },
    {
      header_name: 'Avatar',
      render: (item) => (
        item.avatar && (
          <img
            className="w-10 h-10 rounded-full object-cover"
            src={item.avatar}
            alt={`${item.displayName} avatar`}
          />
        )
      )
    },
    { header_name: 'Display Name', header_value_key: 'displayName' },
    { header_name: 'Login Name', header_value_key: 'login' },
    { header_name: 'Followed At', header_value_key: 'followedAt' },
  ];

  if (isLoading) {
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
        <span className="ml-3 text-gray-400">Loading following list...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch following list."}</p>
      </div>
    );
  }

  return (
    <>
      <h3 className="text-center text-xl font-semibold text-gray-200 mb-4">Following list for `{response_data?.data.forUser.login_name}`</h3>

      {response_data?.data && (
        <ResponsiveDataDisplay
          data={response_data.data.follows}
          columns={followingColumns}
          rowKey="id"
          emptyMessage="No following data found."
        />
      )}
    </>
  );
}
