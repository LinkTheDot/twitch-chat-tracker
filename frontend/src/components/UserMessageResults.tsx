import { formatDate } from '../services/FormatDate';
import { buildFetchUrl } from '../services/FetchUrl';
import { useGetData } from '../services/DataRequest';
import { QueryFormData } from '../types/QueryFormData';
import { Pagination } from '../types/Pagination';
import { Emote, UserMessage, UserMessageResponse } from '../types/UserMessage';
import { User } from '../types/users';

interface MessageResultsProps {
  queryResults: QueryFormData;
  pagination: Pagination | null;
  updatePagination: (paginationResponse: Pagination | null) => void;
  setIsLoading: (isLoading: boolean) => void;
}

// Component to render a single message with emotes
const MessageRow: React.FC<{ message: UserMessage; user: User }> = ({ message, user }) => {
  const renderMessageWithEmotes = (contents: string, emoteUsage: Emote[]): React.ReactNode => {
    if (!emoteUsage || emoteUsage.length === 0) {
      return contents;
    }

    // Convert string to UTF-8 bytes to match backend indexing
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const contentBytes = encoder.encode(contents);

    // Flatten all emote instances with their positions
    const allEmoteInstances: Array<{
      contents_index: number;
      emote_name_size: number;
      emote_image_url: string;
    }> = [];

    // Process each emote type and add all its instances
    for (const emote of emoteUsage) {
      const { contents_indices, emote_name_size, emote_image_url } = emote;

      // Add each instance of this emote
      for (const index of contents_indices) {
        allEmoteInstances.push({
          contents_index: index,
          emote_name_size,
          emote_image_url
        });
      }
    }

    // Sort all emote instances by their position in the message (ascending)
    const sortedEmotes = allEmoteInstances.sort((a, b) => a.contents_index - b.contents_index);

    // Validate emote positions to prevent out-of-bounds errors
    const validEmotes = sortedEmotes.filter(emote => {
      const endIndex = emote.contents_index + emote.emote_name_size;
      return emote.contents_index >= 0 &&
        emote.contents_index < contentBytes.length &&
        endIndex <= contentBytes.length &&
        emote.emote_name_size > 0;
    });

    const parts: React.ReactNode[] = [];
    let lastByteIndex = 0;
    let keyCounter = 0;

    // Process each emote instance in sorted order
    for (const emote of validEmotes) {
      const { contents_index, emote_name_size, emote_image_url } = emote;
      const endByteIndex = contents_index + emote_name_size;

      // Add text before this emote (convert byte indices back to string)
      if (contents_index > lastByteIndex) {
        const textBeforeBytes = contentBytes.slice(lastByteIndex, contents_index);
        const textBefore = decoder.decode(textBeforeBytes);
        if (textBefore) {
          parts.push(
            <span key={`text-${keyCounter++}`}>{textBefore}</span>
          );
        }
      }

      // Get the emote name (convert byte indices back to string)
      const emoteNameBytes = contentBytes.slice(contents_index, endByteIndex);
      const emoteName = decoder.decode(emoteNameBytes);

      // Add the emote image
      parts.push(
        <img
          key={`emote-${keyCounter++}-${contents_index}`}
          src={emote_image_url}
          alt={emoteName}
          className="inline-block h-6 w-auto mx-0.5"
          style={{ verticalAlign: 'middle' }}
          onError={(e) => {
            console.log(`Emote failed to load: ${emoteName} at ${emote_image_url}`);
            // Replace with text if image fails to load
            e.currentTarget.style.display = 'none';
            const textNode = document.createTextNode(emoteName);
            e.currentTarget.parentNode?.insertBefore(textNode, e.currentTarget);
          }}
        // onLoad={() => console.log(`Emote loaded: ${emoteName}`)}
        />
      );

      lastByteIndex = endByteIndex;
    }

    // Add any remaining text at the end
    if (lastByteIndex < contentBytes.length) {
      const textAfterBytes = contentBytes.slice(lastByteIndex);
      const textAfter = decoder.decode(textAfterBytes);
      if (textAfter) {
        parts.push(
          <span key={`text-${keyCounter++}`}>{textAfter}</span>
        );
      }
    }

    return <span className="inline-flex items-center flex-wrap">{parts}</span>;
  };

  const rowClasses = `
    flex items-center gap-2 py-1 px-2 text-sm
    ${message.is_first_message
      ? 'bg-green-900/30'
      : 'hover:bg-gray-800/50'
    }
  `;

  return (
    <div className={rowClasses}>
      {/* Date */}
      <span className="text-gray-400 text-xs font-mono shrink-0 w-32">
        {formatDate(message.timestamp)}
      </span>

      {/* Subscriber badge */}
      {message.is_subscriber && (
        <img
          src="https://static-cdn.jtvnw.net/badges/v1/5d9f2208-5dd8-11e7-8513-2ff4adfae661/3"
          alt="Subscriber"
          className="h-4 w-4 shrink-0"
          onError={(e) => {
            console.log('Badge image failed to load:', e);
            e.currentTarget.style.display = 'none';
          }}
        // onLoad={() => console.log('Badge image loaded successfully')}
        />
      )}

      {/* Username */}
      <span className="text-purple-300 font-medium shrink-0">
        {user.login_name}:
      </span>

      {/* Message content with emotes */}
      <div className="text-gray-100 flex-1 min-w-0">
        {renderMessageWithEmotes(message.contents, message.emote_usage)}
      </div>
    </div>
  );
};

// Main component
export function MessageResults(props: MessageResultsProps) {
  if (!props.queryResults.userSearchQuery || !props.queryResults.channelSearchQuery) {
    let missingData;

    if (!props.queryResults.userSearchQuery && !props.queryResults.channelSearchQuery) {
      missingData = "user and channel";
    } else if (!props.queryResults.userSearchQuery) {
      missingData = "user"
    } else {
      missingData = "channel"
    }

    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {`Missing ${missingData}` || "Failed to fetch users."}</p>
      </div>
    );
  }

  const userIdentifier = props.queryResults.userSearchQuery || props.queryResults.channelSearchQuery;
  const requestType = Number(userIdentifier) ? "user_id" : "maybe_login";

  const requestUrl = buildFetchUrl({
    route: "/users/messages",
    dataName: requestType,
    data: userIdentifier,
    pagination: props.pagination,
    channel: props.queryResults.channelSearchQuery,
    additional: "page_size=1000"
  });

  const { response_data, error } = useGetData<UserMessageResponse>({
    requestUrl,
    updatePagination: props.updatePagination,
    setIsLoading: props.setIsLoading
  });

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-400">Error: {error.message || "Failed to fetch user messages."}</p>
      </div>
    );
  }

  return (
    <>
      {response_data?.data && (
        <div className="bg-gray-900/50 rounded-lg border border-gray-700">
          {/* Header */}
          <div className="px-4 py-3 border-b border-gray-700">
            <h2 className="text-lg font-semibold text-gray-100">
              Chat Messages from `{response_data.data.user.display_name}` to `{response_data.data.channel.display_name}`
            </h2>
          </div>

          {/* Messages list */}
          <div className="divide-y divide-gray-700/50">
            {response_data.data.messages.map((message) => (
              <MessageRow key={message.id} message={message} user={response_data.data.user} />
            ))}
          </div>

          {/* Empty state */}
          {response_data.data.messages.length === 0 && (
            <div className="px-4 py-8 text-center">
              <p className="text-gray-400">No messages found</p>
            </div>
          )}
        </div>
      )}
    </>
  );
}
