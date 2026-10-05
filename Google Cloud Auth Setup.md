# Google Cloud Authentication Setup

You will need a Google OAuth client ID and client secret to authorize Cloud Mounter
and not use the default rclone client. The default client is shared by many users and
will be retires 2026. The author of Cloud Mounter is not planning to provide
Google OAuth client ID for the applet but describes here how you create your own.

Cloud Mounter uses `rclone` for Google Drive access. Authentication occurs
between `rclone`, Google, and the user through a browser. Cloud Mounter does not
authenticate the Google account itself and does not store the resulting Google
OAuth token.

A Google OAuth client ID and client secret identify the application; they do not
grant access to a Google account. Each Google account must still sign in and
approve access in the browser. One Desktop OAuth client can be used with both
personal and Google Workspace accounts when the Workspace administrator's
policy permits it.

## 1. Create or select a Google Cloud project

1. Open the [Google Cloud Console](https://console.cloud.google.com/).
2. Select an existing project or create a project for Cloud Mounter.
3. Keep that project selected throughout the remaining steps.

## 2. Enable the Google Drive API

1. Open **APIs & Services > Library**.
2. Search for **Google Drive API**.
3. Open **Google Drive API** and select **Enable**.

Do not select a similarly named third-party product. The service name shown by
Google for the correct API is `drive.googleapis.com`.

## 3. Configure Google Auth Platform branding

Open the [Google Auth Platform](https://console.cloud.google.com/auth/overview)
and select **Branding**.

Complete these fields:

- **App name:** a recognizable name such as `Cloud Mounter`.
- **User support email:** an address you monitor.
- **Application home page:** a public page that describes the application.
- **Application privacy policy link:** a public privacy policy linked from the
  homepage.
- **Authorized domains:** the domain hosting the homepage and privacy policy,
  without `https://` or a path.
- **Developer contact information:** an address you monitor.

The terms-of-service link is optional. Leave the app logo empty unless you want
to complete Google's branding-verification process; uploading a logo to an
External production app triggers verification before Google displays it.

The Cloud Mounter project publishes an example of the required pages:

- [Cloud Mounter homepage](https://uutzinger.github.io/cosmic-ext-applet-mounter/)
- [Cloud Mounter privacy policy](https://uutzinger.github.io/cosmic-ext-applet-mounter/privacy/)

If you create your own OAuth project, Google may require you to use pages on a
domain that you control and can verify. Do not claim the project's GitHub Pages
domain as your own authorized domain unless Google accepts you as an owner of
that domain.

## 4. Configure the audience

Select **Audience** and choose **External** if the client will authorize personal
Google accounts, accounts from more than one organization, or both.

For ongoing use, select **Publish app** so the publishing status becomes **In
production**. An External production app can be authorized by any Google
account, so keep a private client ID and secret private. Publishing does not
give anyone access to your files: every account must still sign in and consent.

An unverified personal-use app may display Google's unverified-app warning and
has a lifetime cap of 100 users. Google permits personal-use apps below that cap
to operate without completing OAuth verification.

Use **Testing** only for temporary setup. Testing allows only accounts in the
test-user list, and authorization for Drive access expires seven days after
consent. Changing to **In production** removes that seven-day limitation for new
authorizations.

See Google's current [app audience documentation](https://support.google.com/cloud/answer/15549945)
and [verification exceptions](https://support.google.com/cloud/answer/13464323).

## 5. Review data access

Open **Data Access** and declare the Google Drive scope requested by the client.
The default full-access rclone Drive configuration uses:

```text
https://www.googleapis.com/auth/drive
```

Request only the scopes needed for the intended rclone configuration.

## 6. Create the Desktop OAuth client

1. Select **Clients**.
2. Select **Create client**.
3. For **Application type**, select **Desktop app**.
4. Enter a name such as `Cloud Mounter`.
5. Select **Create**.
6. Copy the displayed **Client ID** and **Client secret** into a secure local
   record.

Cloud Mounter needs the two text values. A downloaded JSON file is not required.
Do not commit the client ID, client secret, OAuth token, or downloaded credential
file to the repository.

## 7. Configure a new Cloud Mounter connection

1. Open Cloud Mounter settings and select **Add Connection**.
2. Select **Google Drive** and the desired access mode.
3. Enter a connection name and an unused rclone remote name.
4. Enter the Google OAuth client ID and matching client secret.
5. Complete the browser sign-in with the Google account that owns the Drive.
6. Complete the remaining connection fields.
7. Select **Test Connection**.
8. After the test passes, select **Save Connection**.

The Google account used during browser authorization determines which Drive is
connected. The Google Cloud account that created the client does not have to be
the same account.

## 8. Update an existing Google Drive connection

1. Open the connection in Modify mode.
2. Enter the client ID and matching client secret.
3. Select **Update Google OAuth Client**.
4. Complete browser authorization for the intended Google account.
5. Select **Test Connection**.
6. After the test passes, select **Save Connection**.

Cloud Mounter clears the credential fields after the update. The corresponding
rclone remote retains the client configuration and OAuth token.

If the account was authorized while the OAuth app was in Testing, authorize it
again after changing the app to In production so the connection receives a new
production authorization.

## Google Workspace accounts

A private Desktop OAuth client does not bypass Google Workspace policy. A
managed organization can block unverified clients, restrict Drive scopes, or
require an administrator to trust the client. If a personal account works but a
corporate account is blocked, provide the client ID and requested Drive scope to
the Workspace administrator.

See Google's documentation for
[controlling third-party and internal app access](https://support.google.com/a/answer/7281227).

## Troubleshooting

- **The wrong Drive is shown:** update the OAuth client again and sign in with
  the intended Google account. The browser account, not the client owner,
  selects the Drive.
- **Authorization expires after seven days:** the OAuth app is still in Testing,
  or the token was issued while it was in Testing. Change the app to In
  production and authorize the connection again.
- **The corporate account is blocked:** ask the Google Workspace administrator
  whether the OAuth client and Drive scope are permitted.
- **Test Connection reports malformed rclone output:** confirm the client ID and
  client secret belong to the same Desktop OAuth client, then repeat **Update
  Google OAuth Client** and browser authorization.
- **Client ID and secret were exposed:** create or rotate the client secret in
  Google Auth Platform, then update every connection that uses that client.
