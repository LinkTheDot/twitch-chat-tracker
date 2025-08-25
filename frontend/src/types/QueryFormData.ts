import { CategoryState } from "./CategoryState";

export interface QueryFormData {
  category: CategoryState;
  channelSearchQuery: string;
  userSearchQuery: string;
  messageSearch: string;
}
