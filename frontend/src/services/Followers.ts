import { useEffect, useState } from "react";
import { Follow, FollowingRequest, FollowingResponse } from "../types/Followers";

export const getFollowing = (followingRequest: FollowingRequest): FollowingResponse => {
  const [follows, setFollows] = useState<Follow[] | null>(null);
  const [error, setError] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    setFollows(null);
    setError(null);
    setIsLoading(true);

    const fetchUrl = `${import.meta.env.VITE_BACKEND_URL}/users/following?login=${followingRequest.userLogin}`;

    const fetchData = async () => {
      try {
        const response = await fetch(fetchUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonResponse = await response.json();

        // Logs the response if needed.
        console.log(JSON.stringify(jsonResponse, null, 2));

        setFollows(jsonResponse);
      } catch (err: any) {
        setError(err);
      } finally {
        setIsLoading(false);
      }
    };

    if (followingRequest.userLogin.trim() !== '') {
      fetchData();
    } else {
      setFollows(null);
      setError(null);
      setIsLoading(false);
    }

  }, [followingRequest.userLogin]);

  return { value: follows, error, isLoading };
}
