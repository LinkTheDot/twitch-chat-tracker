import { getFollowing } from "../services/Followers";
import { Follow, FollowingRequest } from "../types/Followers";
import { Column, DataTable } from "./DataTable";
import { QueryFormData } from "./QueryForm";
import "./FollowingResults.css";

export interface FollowingResultsProps {
  queryResults: QueryFormData;
}

export function FollowingResults(props: FollowingResultsProps) {
  const identifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;

  const followingRequest: FollowingRequest = {
    userLogin: identifier,
  };

  const { value: following, error, isLoading } = getFollowing(followingRequest);

  const followingColumns: Column<Follow>[] = [
    { header_name: 'Twitch ID', header_value_key: 'id' },
    { header_name: 'Avatar', 
      render: (item) => (
        item.avatar && (
          <img
            className="following_avatar"
            src={item.avatar}
          />
        )
      )
    },
    { header_name: 'Display Name', header_value_key: 'displayName' },
    { header_name: 'Login Name', header_value_key: 'login' },
    { header_name: 'Followed At', header_value_key: 'followedAt' },
  ];

  if (isLoading) {
    return <div className="nondata_message">Loading users...</div>;
  }

  if (error) {
    return <div className="nondata_message">Error: {error.message || "Failed to fetch users."}</div>;
  }

  return (
    <div>
      {following && (
        <DataTable
          data={following}
          columns={followingColumns}
          rowKey="id"
          emptyMessage="No name changes found."
        />
      )}
    </div>
  );
}
