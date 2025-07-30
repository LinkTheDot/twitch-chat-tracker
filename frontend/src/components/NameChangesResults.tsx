import { getNameChanges } from "../services/NameChanges";
import { NameChange, NameChangeRequest } from "../types/NameChanges";
import { UserRequestType } from "../types/users";
import { Column, DataTable } from "./DataTable";
import { QueryFormData } from "./QueryForm";

export interface NameChangeResultsProps {
  queryResults: QueryFormData;
}

export function NameChangeResults(props: NameChangeResultsProps) {
  const identifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(identifier) ? UserRequestType.TwitchId : UserRequestType.Name;

  const nameChangeRequest: NameChangeRequest = {
    requestType,
    userIdentifier: identifier,
  };

  const { value: nameChanges, error, isLoading } = getNameChanges(nameChangeRequest);

  const nameChangeColumns: Column<NameChange>[] = [
    { header_name: 'Twitch ID', header_value_key: 'twitch_user_twitch_id' },
    { header_name: 'Previous Login', header_value_key: 'previous_login_name' },
    { header_name: 'New Login', header_value_key: 'new_login_name' },
    { header_name: 'Entry Creation Date', header_value_key: 'created_at' },
  ];

  if (isLoading) {
    return <div className="nondata_message">Loading users...</div>;
  }

  if (error) {
    return <div className="nondata_message">Error: {error.message || "Failed to fetch users."}</div>;
  }

  return (
    <div>
      {nameChanges && (
        <DataTable
          data={nameChanges}
          columns={nameChangeColumns}
          rowKey="twitch_user_twitch_id"
          emptyMessage="No name changes found."
        />
      )}
    </div>
  );
}
