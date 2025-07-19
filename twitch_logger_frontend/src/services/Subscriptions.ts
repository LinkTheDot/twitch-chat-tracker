import { useEffect, useState } from "react";
import { SubscriptionRequest, SubscriptionRequestType, SubscriptionResponse, Subscriptions } from "../types/Subscriptions";

export const getSubscriptions = (subscriptionRequest: SubscriptionRequest): SubscriptionResponse => {
  const [subscriptions, setSubscriptions] = useState<Subscriptions | null>(null);
  const [error, setError] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    setSubscriptions(null);
    setError(null);
    setIsLoading(true);

    const maybeChannel = subscriptionRequest.channel ? `${subscriptionRequest.channel}/` : '';
    const fetchIdentifier = subscriptionRequest.userRequestType === SubscriptionRequestType.Name ? "login" : "user_id";
    const path = `${maybeChannel}donations/subscriptions`;
    const fetchUrl = `${import.meta.env.VITE_BACKEND_URL}/${path}?${fetchIdentifier}=${subscriptionRequest.userIdentifier}`;

    const fetchData = async () => {
      try {
        const response = await fetch(fetchUrl);
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const jsonResponse = await response.json();

        setSubscriptions(jsonResponse);
      } catch (err: any) {
        setError(err);
      } finally {
        setIsLoading(false);
      }
    };

    if (subscriptionRequest.userIdentifier.trim() !== '') {
      fetchData();
    } else {
      setSubscriptions(null);
      setError(null);
      setIsLoading(false);
    }

  }, [subscriptionRequest.userRequestType, subscriptionRequest.userIdentifier]);

  return { value: subscriptions, error, isLoading };
}
