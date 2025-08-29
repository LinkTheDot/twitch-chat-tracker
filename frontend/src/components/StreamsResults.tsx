import { useGetData } from "../services/DataRequest";
import { buildFetchUrl } from "../services/FetchUrl";
import { Pagination } from "../types/Pagination";
import { QueryFormData } from "../types/QueryFormData";
import { Stream, Streams } from "../types/Streams";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";
import { formatDate } from '../services/FormatDate';

export interface StreamsResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
  setIsLoading: (isLoading: boolean) => void;
}

export function StreamsResults(props: StreamsResultsProps) {
  if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
    return;
  }

  const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(userIdentifier) ? "user_id" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/users/streams",
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
  });

  const { response_data, error } = useGetData<Streams>({
    requestUrl,
    updatePagination: props.updatePagination,
    setIsLoading: props.setIsLoading,
  });

  const followingColumns: Column<Stream>[] = [
    { header_name: 'Stream ID', header_value_key: 'twitch_stream_id' },
    {
      header_name: 'Start Time',
      render: (item) => (
        <span className="text-sm text-gray-300">
          {formatDate(item.start_timestamp)}
        </span>
      )
    },
    {
      header_name: 'End Time',
      render: (item) => (
        <span className="text-sm text-gray-300">
          {formatDate(item.end_timestamp)}
        </span>
      )
    },
  ];

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch streams list."}</p>
      </div>
    );
  }

  return (
    <>
      <h3 className="text-center text-xl font-semibold text-gray-200 mb-4">Streams for `{response_data?.data.user.login_name}`</h3>

      {response_data?.data && (
        <ResponsiveDataDisplay
          data={response_data.data.streams}
          columns={followingColumns}
          rowKey="id"
          emptyMessage="No following data found."
        />
      )}
    </>
  );
}
