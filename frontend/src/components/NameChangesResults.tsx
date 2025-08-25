import { NameChange } from "../types/NameChanges";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";
import { useGetData } from "../services/DataRequest";
import { Pagination } from "../types/Pagination";
import { buildFetchUrl } from "../services/FetchUrl";
import { QueryFormData } from "../types/QueryFormData";

export interface NameChangeResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
  setIsLoading: (isLoading: boolean) => void;
}

export function NameChangeResults(props: NameChangeResultsProps) {
  if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
    return;
  }

  const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(userIdentifier) ? "twitch_id" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/users/name_changes",
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
  });

  const { response_data, error } = useGetData<NameChange[]>({
    requestUrl,
    updatePagination: props.updatePagination,
    setIsLoading: props.setIsLoading,
  });

  console.log(`Response data=${response_data?.data}`);

  const nameChangeColumns: Column<NameChange>[] = [
    { header_name: 'Twitch ID', header_value_key: 'twitch_user_twitch_id' },
    { header_name: 'Previous Login', header_value_key: 'previous_login_name' },
    { header_name: 'New Login', header_value_key: 'new_login_name' },
    { header_name: 'Entry Creation Date', header_value_key: 'created_at' },
  ];

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch name changes."}</p>
      </div>
    );
  }

  return (
    <>
      {response_data?.data && (
        <ResponsiveDataDisplay
          data={response_data.data}
          columns={nameChangeColumns}
          rowKey="id"
          emptyMessage="No name changes found."
        />
      )}
    </>
  );
}
