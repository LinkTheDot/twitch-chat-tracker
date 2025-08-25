// import { useGetData } from "../services/DataRequest";
// import { buildFetchUrl } from "../services/FetchUrl";
// import { Pagination } from "../types/Pagination";
// import { QueryFormData } from "../types/QueryFormData";
// import { UserMessage } from "../types/UserMessage";
// import { Column } from "./ResponsiveDataDisplay";
//
// export interface UserMessagesResultsProps {
//   queryResults: QueryFormData;
//   pagination: Pagination | null;
//   updatePagination: (paginationResponse: Pagination | null) => void;
// }
//
// export function UserMessagesResults(props: UserMessagesResultsProps) {
//   if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
//     return;
//   }
//
//   const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
//   const requestType = Number(userIdentifier) ? "user_id" : "user_login";
//
//   const requestUrl = buildFetchUrl({
//     route: "/users/messages",
//     dataName: requestType,
//     data: userIdentifier,
//     pagination: props.pagination,
//     channel: props.queryResults.channelSearchQuery,
//   });
//
//   const { response_data, error, isLoading } = useGetData<UserMessage[]>({ requestUrl, updatePagination: props.updatePagination });
//
//   const userMessageColumns: Column<UserMessage>[] = [
//     { header_name: 'Login', header_value_key: '' },
//   ];
//
//   if (isLoading) {
//     return (
//       <div className="flex justify-center items-center py-12">
//         <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-500"></div>
//         <span className="ml-3 text-gray-400">Loading name changes...</span>
//       </div>
//     );
//   }
//
//   if (error) {
//     return (
//       <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
//         <p className="text-red-400">Error: {error.message || "Failed to fetch name changes."}</p>
//       </div>
//     );
//   }
// }
