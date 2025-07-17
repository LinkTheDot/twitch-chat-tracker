import { useState, useEffect } from 'react';
import { User, UserRequest, UserRequestType, UserResponse } from '../types/users'; // Assuming these types are correct

export const getUsers = (userRequest: UserRequest): UserResponse => {
  const [users, setUsers] = useState<User[] | null>(null);
  const [error, setError] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    setUsers(null);
    setError(null);
    setIsLoading(true);

    const fetchIdentifier = userRequest.userRequestType === UserRequestType.Name ? "maybe_login" : "user_ids";
    const fetchUrl = `${import.meta.env.VITE_BACKEND_URL}/users?${fetchIdentifier}=${userRequest.userIdentifier}`;

    const fetchData = async () => {
      try {
        const response = await fetch(fetchUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonResponse = await response.json();

        // Logs the response if needed.
        // console.log(JSON.stringify(jsonResponse, null, 2));

        setUsers(jsonResponse);
      } catch (err: any) {
        setError(err);
      } finally {
        setIsLoading(false);
      }
    };

    if (userRequest.userIdentifier.trim() !== '') {
      fetchData();
    } else {
      setUsers(null);
      setError(null);
      setIsLoading(false);
    }

  }, [userRequest.userRequestType, userRequest.userIdentifier]);

  return { users, error, isLoading };
};
