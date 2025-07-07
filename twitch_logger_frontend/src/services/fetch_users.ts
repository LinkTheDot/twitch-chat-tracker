import { User, UserRequest, UserRequestType } from '../types/users'

// Takes a UserRequest to return a list of users.
//
// Use `getUserByName` or `getUserByTwitchID` if only one user is desired.
export const getUsers = async (userRequest: UserRequest): Promise<User[]> => {
  const fetchIdentifier = userRequest.request_type === UserRequestType.Name ? "logins" : "user_ids";

  const response = await fetch(`${import.meta.env.VITE_BACKEND_URL}/users?${fetchIdentifier}=${userRequest.user_identifier}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch users ${response.status}`);
  }

  return response.json();
};

export const getUserByName = async (userName: string): Promise<User> => {
  const users = await getUsers({
    request_type: UserRequestType.Name,
    user_identifier: userName,
  });

  if (users.length === 0) {
    throw new Error('User not found');
  }

  return users[0];
};

export const getUserByTwitchID = async (userID: string): Promise<User> => {
  const users = await getUsers({
    request_type: UserRequestType.TwitchId,
    user_identifier: userID,
  });

  if (users.length === 0) {
    throw new Error('User not found');
  }

  return users[0];
};
