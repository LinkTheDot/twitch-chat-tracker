import { ResponsiveDataDisplay, Column } from './ResponsiveDataDisplay';
import { User } from '../types/users';
import { Pagination } from '../types/Pagination';
import { QueryFormData } from '../types/QueryFormData';
import { useGetData } from '../services/DataRequest';
import { buildFetchUrl } from '../services/FetchUrl';


export interface UserResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
}

export function UserResults(props: UserResultsProps) {
  if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
    return;
  }

  const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(userIdentifier) ? "user_ids" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/users",
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
  });

  const { response_data, error, isLoading } = useGetData<User[]>({ requestUrl, updatePagination: props.updatePagination });

  const userColumns: Column<User>[] = [
    { header_name: 'Id', header_value_key: 'id' },
    { header_name: 'Twitch ID', header_value_key: 'twitch_id' },
    { header_name: 'Display Name', header_value_key: 'display_name' },
    { header_name: 'Login Name', header_value_key: 'login_name' },
  ];

  if (isLoading) {
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
        <span className="ml-3 text-gray-400">Loading users...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch users."}</p>
      </div>
    );
  }

  return (
    <>
      {response_data?.data && (
        <ResponsiveDataDisplay
          data={response_data.data}
          columns={userColumns}
          rowKey="id"
          emptyMessage="No users found."
        />
      )}
    </>
  );
}
