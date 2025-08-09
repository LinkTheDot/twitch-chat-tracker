import { ResponsiveDataDisplay, Column } from './ResponsiveDataDisplay';
import { User, UserRequest, UserRequestType } from '../types/users';
import { getUsers } from '../services/users'; // Import the new custom hook
import { QueryFormData } from '../components/QueryForm';


export interface UserResultsProps {
  queryResults: QueryFormData;
}

export function UserResults(props: UserResultsProps) {
  const identifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(identifier) ? UserRequestType.TwitchId : UserRequestType.Name;

  const userRequest: UserRequest = {
    userRequestType: requestType,
    userIdentifier: identifier
  };

  const { value: users, error, isLoading } = getUsers(userRequest);

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
      {users && (
        <ResponsiveDataDisplay
          data={users}
          columns={userColumns}
          rowKey="id"
          emptyMessage="No users found."
        />
      )}
    </>
  );
}
