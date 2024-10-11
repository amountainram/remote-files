# Remote Files

A command line and library wrapper of [OpenDAL](https://github.com/apache/opendal), which allows managing remote files on buckets, such as download and download them.

Currently supported buckets are:

- [Google Cloud Storage](https://cloud.google.com/storage)
- [Amazon S3](https://aws.amazon.com/s3/)

Remote-files may work also with cloud storage providers that offer an API compatible with GCS or S3, but no guarantees are provided.

## Installation

```bash
cargo install remote-files
```

## Configuration

In order to use `remote-files` it is necessary to generate a configuration file that
lists all the connection profiles, alongside their credentials details.

The location of this configuration file is governed through environment variable
`RF_CFG_FILEPATH`, whose default value depends on the operating system.


| OS      | OS Configuration Folder             | Example                                                          |
| ------- | ----------------------------------- | ---------------------------------------------------------------- |
| Linux   | `$HOME/.config`                     | `/home/alice/.config/rf/configuration.json`                      |
| MacOS   | `$HOME/Library/Application Support` | `/Users/Alice/Library/Application Support/rf/configuration.json` |
| Windows | `{FOLDERID_RoamingAppData}`         | `C:\Users\Alice\AppData\Roaming\rf\configuration.json`           |

### Examples

Here is provided an example of configuration file that defines two different connection profiles,
one for Google Cloud Storage and one for Amazon S3.

```json
{
  "my-gcs-bucket": {
    "type": "gcs",
    "configuration": {
      "name": "my-gcs-bucket",
      "credentialPath": "~/.config/gcloud/application_default_credentials.json"
    }
  },
  "my-aws-bucket": {
    "type": "s3",
    "configuration": {
      "name": "my-aws-bucket",
      "endpoint": null,
      "prefix": null,
      "region": "eu-west-1",
      "accessKeyId": "********************",
      "secretAccessKey": "******************************",
      "defaultStorageClass": null
    }
  }
}
```

## Commands

In the following sections are described which commands can be executed with `remote-files`.

### `profiles`

Manage connection profiles

### `list`

List files in selected folder

### `download`

Download selected file from source directory

### `upload`

Upload selected file to target directory