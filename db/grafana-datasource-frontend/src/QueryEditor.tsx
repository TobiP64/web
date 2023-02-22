import { defaults } from 'lodash';

import React, { ChangeEvent, PureComponent, SyntheticEvent } from 'react';
import { LegacyForms } from '@grafana/ui';
import { QueryEditorProps } from '@grafana/data';
import { DataSource } from './datasource';
import { defaultQuery, MyDataSourceOptions, MyQuery } from './types';

const { FormField, Switch } = LegacyForms;

type Props = QueryEditorProps<DataSource, MyQuery, MyDataSourceOptions>;

export class QueryEditor extends PureComponent<Props> {
  onQueryTextChange = (event: ChangeEvent<HTMLInputElement>) => {
    const { onChange, query } = this.props;
    onChange({ ...query, queryText: event.target.value });
  };

  onWithStreamingChange = (event: SyntheticEvent<HTMLInputElement>) => {
    const { onChange, query, onRunQuery } = this.props;
    onChange({ ...query, withStreaming: event.currentTarget.checked });
    // executes the query
    onRunQuery();
  };

  render() {
    const query = defaults(this.props.query, defaultQuery);
    const { queryText, withStreaming } = query;

    return (
      <div className="gf-form">
        <FormField
          labelWidth={10}
          value={queryText || ''}
          onChange={this.onQueryTextChange}
          label="Query"
        />

        <InlineLabel width="auto" tooltip="Tooltip content">
          Cluster Durability
        </InlineLabel>
        <Select
            options={[
              { label: 'Single', value: 0 },
              { label: 'Majority', value: 0 },
            ]}
            value={value}
        />
        <InlineLabel width="auto" tooltip="Tooltip content">
          Durability
        </InlineLabel>
        <Select
            options={[
              { label: 'Soft', value: 0 },
              { label: 'Hard', value: 0 },
            ]}
            value={value}
        />
        <InlineLabel width="auto" tooltip="Tooltip content">
          Isolation
        </InlineLabel>
        <Select
            options={[
              { label: 'Committed', value: 0 },
              { label: 'Snapshot', value: 0 },
            ]}
            value={value}
        />
      </div>
    );
  }
}
