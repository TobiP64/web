import React, { ChangeEvent, PureComponent } from 'react';
import { LegacyForms } from '@grafana/ui';
import { DataSourcePluginOptionsEditorProps } from '@grafana/data';
import { MyDataSourceOptions, MySecureJsonData } from './types';

const { SecretFormField, FormField } = LegacyForms;

interface Props extends DataSourcePluginOptionsEditorProps<MyDataSourceOptions> {}

interface State {}

export class ConfigEditor extends PureComponent<Props, State> {
  onUrlChange = (event: ChangeEvent<HTMLInputElement>) => {
    const { onOptionsChange, options } = this.props;
    const jsonData = {
      ...options.jsonData,
      url: event.target.value,
    };
    onOptionsChange({ ...options, jsonData });
  };

  onTimeoutChange = (event: ChangeEvent<HTMLInputElement>) => {
    const { onOptionsChange, options } = this.props;
    const jsonData = {
      ...options.jsonData,
      timeout: event.target.value,
    };
    onOptionsChange({ ...options, jsonData });
  };

  onChangeTlsPrivateKey = (event: ChangeEvent<HTMLInputElement>) => {
    const { onOptionsChange, options } = this.props;
    onOptionsChange({
      ...options,
      secureJsonData: {
        tlsPrivateKey: event.target.value,
      },
    });
  };

  onChangeTlsCertificate = (event: ChangeEvent<HTMLInputElement>) => {
    const { onOptionsChange, options } = this.props;
    onOptionsChange({
      ...options,
      secureJsonData: {
        tlsCertificate: event.target.value,
      },
    });
  };

  onResetTlsPrivateKey = () => {
    const { onOptionsChange, options } = this.props;
    onOptionsChange({
      ...options,
      secureJsonFields: {
        ...options.secureJsonFields,
        tlsPrivateKey: false,
      },
      secureJsonData: {
        ...options.secureJsonData,
        tlsPrivateKey: '',
      },
    });
  };

  onResetTlsCertificate = () => {
    const { onOptionsChange, options } = this.props;
    onOptionsChange({
      ...options,
      secureJsonFields: {
        ...options.secureJsonFields,
        tlsCertificate: false,
      },
      secureJsonData: {
        ...options.secureJsonData,
        tlsCertificate: '',
      },
    });
  };

  render() {
    const { options } = this.props;
    const { jsonData, secureJsonFields } = options;
    const secureJsonData = (options.secureJsonData || {}) as MySecureJsonData;

    return (
      <div className="gf-form-group">
        <div className="gf-form">
          <FormField
            label="URL"
            labelWidth={10}
            inputWidth={20}
            onChange={this.onUrlChange}
            value={jsonData.url || ''}
            placeholder="URL"
          />
        </div>

        <div className="gf-form">
          <FormField
              label="Timeout"
              labelWidth={10}
              inputWidth={20}
              onChange={this.onTimeoutChange}
              value={jsonData.url || ''}
              placeholder="Timeout in seconds"
          />
        </div>

        <div className="gf-form-inline">
          <div className="gf-form">
            <SecretFormField
              isConfigured={(secureJsonFields && secureJsonFields.tlsPrivateKey) as boolean}
              value={secureJsonData.tlsPrivateKey || ''}
              label="TLS Private Key"
              placeholder=""
              labelWidth={10}
              inputWidth={20}
              onReset={this.onResetTlsPrivateKey}
              onChange={this.onChangeTlsPrivateKey}
            />
          </div>
        </div>

        <div className="gf-form-inline">
          <div className="gf-form">
            <SecretFormField
              isConfigured={(secureJsonFields && secureJsonFields.tlsCertificate) as boolean}
              value={secureJsonData.tlsCertificate || ''}
              label="TLS Certificate"
              placeholder=""
              labelWidth={10}
              inputWidth={20}
              onReset={this.onResetTlsCertificate}
              onChange={this.onChangeTlsCertificate}
            />
          </div>
        </div>
      </div>
    );
  }
}
