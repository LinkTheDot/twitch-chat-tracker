import { getFollowing } from "../services/Followers";
import { Follow, FollowingRequest } from "../types/Followers";
import { Column, ResponsiveDataDisplay } from "./ResponsiveDataDisplay";
import { QueryFormData } from "./QueryForm";

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
      {following && (
        <ResponsiveDataDisplay
          data={following}
          columns={followingColumns}
          rowKey="id"
          emptyMessage="No following data found."
        />
      )}
    </>
  );
}
