import { DataTable, Column } from './DataTable';
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
    return <div className="nondata_message">Loading users...</div>;
  }

  if (error) {
    return <div className="nondata_message">Error: {error.message || "Failed to fetch users."}</div>;
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
