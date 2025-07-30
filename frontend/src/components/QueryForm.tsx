import { useState } from "react";
import { CategoryState } from "../types/CategoryState";
import "./QueryForm.css";

export interface QueryFormData {
  category: CategoryState;
  channelSearchQuery: string;
  userSearchQuery: string;
}

interface QueryFormProps {
  onSubmitQuery: (data: QueryFormData) => void;
}

const QueryForm: React.FC<QueryFormProps> = ({ onSubmitQuery }) => {
  const [formData, setFormData] = useState<QueryFormData>({
    category: CategoryState.Users,
    channelSearchQuery: '',
    userSearchQuery: '',
  });

  const categoryOptions = Object.values(CategoryState);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    onSubmitQuery(formData);
  };

  const handleCategoryChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setFormData({ ...formData, category: event.target.value as CategoryState });
  };

  const handleChannelSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({ ...formData, channelSearchQuery: event.target.value });
  };

  const handleUserSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({ ...formData, userSearchQuery: event.target.value });
  };

  return (
    <form onSubmit={handleSubmit} className="query-form">
      <select
        value={formData.category}
        onChange={handleCategoryChange}
        className="query-form-input form-category-select"
      >
        {categoryOptions.map((category) => (
          <option key={category} value={category}>
            {category}
          </option>
        ))}
      </select>

      <input
        type="text"
        placeholder="Channel Filter (optional)"
        value={formData.channelSearchQuery}
        onChange={handleChannelSearchChange}
        className="query-form-input form-channel-query-input query-form-text-input"
      />

      <input
        type="text"
        placeholder="User Search"
        value={formData.userSearchQuery}
        onChange={handleUserSearchChange}
        className="query-form-input form-user-query-input query-form-text-input"
      />

      <button
        type="submit"
        className="query-form-input query-form-submit-button"
      >
        Confirm
      </button>
    </form>
  );
};

export default QueryForm;
