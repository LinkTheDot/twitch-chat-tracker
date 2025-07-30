import { useEffect, useState } from "react"
import { NameChange, NameChangeRequest, NameChangeResponse, UserRequestType } from "../types/NameChanges"

export const getNameChanges = (nameChangeRequest: NameChangeRequest): NameChangeResponse => {
  const [nameChanges, setNameChanges] = useState<NameChange[] | null>(null);
  const [error, setError] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    setNameChanges(null);
    setError(null);
    setIsLoading(true);

    const fetchIdentifier = nameChangeRequest.requestType === UserRequestType.Name ? "maybe_name" : "twitch_id";
    const fetchUrl = `${import.meta.env.VITE_BACKEND_URL}/users/name_changes?${fetchIdentifier}=${nameChangeRequest.userIdentifier}`;

    const fetchData = async () => {
      try {
        const response = await fetch(fetchUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonResponse = await response.json();

        setNameChanges(jsonResponse);
      } catch (err: any) {
        setError(err);
      } finally {
        setIsLoading(false);
      }
    };

    if (nameChangeRequest.userIdentifier.trim() !== '') {
      fetchData();
    } else {
      setNameChanges(null);
      setError(null);
      setIsLoading(false);
    }

  }, [nameChangeRequest.userIdentifier]);

  return { value: nameChanges, error, isLoading };
}
