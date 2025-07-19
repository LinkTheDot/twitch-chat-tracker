import './DataTable.css';

export interface Column<T> {
  header_name: string;
  header_value_key?: keyof T;
  render?: (item: T) => React.ReactNode;
}

export interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  rowKey: keyof T | ((item: T) => string | number);
  emptyMessage?: string;
}

export function DataTable<T>({ data, columns, rowKey, emptyMessage = "No results found." }: DataTableProps<T>) {
  if (!data || data.length === 0) {
    return (
      <p className="nondata_message">
        {emptyMessage}
      </p>
    );
  }

  return (
    <div className="data-table-container">
      <table>
        <thead>
          <tr>
            {columns.map((column, index) => (
              <th key={column.header_name || index}>{column.header_name}</th>
            ))}
          </tr>
        </thead>

        <tbody>
          {
            data.map((item, _rowIndex) => {
              const rowKeyValue = typeof rowKey === 'function' ? rowKey(item) : item[rowKey] as string | number;

              return (
                <tr key={rowKeyValue}>
                  {
                    columns.map((column, colIndex) => (
                      <td key={`${rowKeyValue}-${column.header_name || colIndex}`}>
                        {
                          column.render ? column.render(item) : (column.header_value_key ? item[column.header_value_key] as React.ReactNode : '')
                        }
                      </td>
                    ))
                  }
                </tr>
              );
            })
          }
        </tbody>
      </table>
    </div>
  );
}
