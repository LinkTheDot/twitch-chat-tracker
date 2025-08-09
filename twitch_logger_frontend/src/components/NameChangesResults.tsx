import { getNameChanges } from "../services/NameChanges";
import { NameChange, NameChangeRequest } from "../types/NameChanges";
import { UserRequestType } from "../types/users";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";
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
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
        <span className="ml-3 text-gray-400">Loading name changes...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch name changes."}</p>
      </div>
    );
  }

  return (
    <>
      {nameChanges && (
        <ResponsiveDataDisplay
          data={nameChanges}
          columns={nameChangeColumns}
          rowKey="twitch_user_twitch_id"
          emptyMessage="No name changes found."
        />
      )}
    </>
  );
}
