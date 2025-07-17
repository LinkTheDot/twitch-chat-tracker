import { DataTable, Column } from './DataTable';
import { User, UserRequest, UserRequestType } from '../types/users';
import { CategoryState } from '../types/CategoryState'; // Assuming CategoryState is used in QueryFormData
import { getUsers } from '../services/users'; // Import the new custom hook
import { QueryFormData } from '../components/QueryForm';


export interface UserResultsProps {
  queryResults: QueryFormData;
}

export function UserResults(props: UserResultsProps) {
  const requestType = Number(props.queryResults.userSearchQuery) ? UserRequestType.TwitchId : UserRequestType.Name;

  const userRequest: UserRequest = {
    userRequestType: requestType,
    userIdentifier: props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery
  };

  const { users, error, isLoading } = getUsers(userRequest);

  const userColumns: Column<User>[] = [
    { header_name: 'Id', header_value_key: 'id' },
    { header_name: 'Twitch ID', header_value_key: 'twitch_id' },
    { header_name: 'Display Name', header_value_key: 'display_name' },
    { header_name: 'Login Name', header_value_key: 'login_name' },
  ];

  if (isLoading) {
    return <div>Loading users...</div>;
  }

  if (error) {
    return <div>Error: {error.message || "Failed to fetch users."}</div>;
  }

  return (
    <div>
      {users && (
        <DataTable
          data={users}
          columns={userColumns}
          rowKey="id"
          emptyMessage="No users found."
        />
      )}
    </div>
  );
}
