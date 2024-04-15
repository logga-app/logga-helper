<h1 align="center">logga-helper</h1>
<p align="center">
    Automatic backup of compressed log files to any S3 compatible backend
</p>

# What does it solve?
When configured, logga rotates log files into compressed `zip` files.
The helper looks for these `zip` files being created and uploads them to an S3 compatible storage.

### Caveats

The repository is a work in progress, not fully supported yet. Use it with caution.

### Configuration

#### yaml
Add the following lines to your `logga` configuration `yaml`:
```yaml
s3:
  bucket: [string | bucket to upload zips to]
  endpoint: [string | s3 storage endpoint]
  region: [string | bucket region]
  keychainAuthentication: [bool | read aws credentials from keychain]
```
You are free to save this config as a separate file, just don't forget to point the helper to the correct config file location.

#### Configuration Profile
[Read more](https://docs.getlogga.com/usage/configuration#configuration-with-custom-mdm-configuration-profile)

You can use the following keys to configure the helper via Profiles:
```plist
<key>S3Bucket</key>
<string>testbucket</string>
<key>S3Endpoint</key>
<string>http://test2</string>
<key>S3Region</key>
<string>us-test-3</string>
<key>S3KeychainAuthentication</key>
<true/>
```

Configuration Profile take precedence over the `yaml` configuration. If (for some reason) the helper fails to use the Profile, it falls back to `yaml` configuration.

### Usage

`config-path`: where to find the **config.yaml** (*default*: /Library/Application Support/Logga/config.yaml)  
`profile-path`: Configuring the helper via MDM Configuration Profiles is supported. If your Profile uses the `com.logga.client` Bundle ID, then you don't ever need to override this flag. (*default*: /Library/Managed Preferences/com.logga.client.plist)  
`bundle-id`: Only override, if your Configuration Profile uses a different Bundle ID. (*default*: com.logga.client)  
`watch-dir`: Points to the directory to watch for `zip` creation events. (*default*: /Library/Application Support/Logga)

#### Env vars
When `keychainAuthentication` is set to **false**, the binary will expect these good old AWS env vars to connect to the S3 storage:

`AWS_ACCESS_KEY_ID`  
`AWS_SECRET_ACCESS_KEY`  
`AWS_DEFAULT_REGION`

#### Keychain
When `keychainAuthentication` is set to **true**, the binary will try to read AWS credentials from the Keychain.

Prior using the binary, create two Keychain password entires for the current user (that runs the helper). The service names should be respectively:
```
com.logga.aws-access-key-id
com.logga.aws-secret-access-key
```

#### Example Invocation

`logga-helper --config-path config.yaml --profile-path /tmp --bundle-id com.test.service --watch-dir /Users`
