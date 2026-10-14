# Microsoft Cloud Authentication Setup

Some organizations do not allow third-party applications to open OneDrive or
SharePoint with the engines' built-in application IDs. In that case, you can
register your own application in Microsoft Entra ID and give its IDs to the
engine. Cloud Mounter itself has no Microsoft client ID fields; the IDs go 
into the configuration of the engine that signs in.

> **Status:** these steps follow the upstream documentation for
> [rclone](https://rclone.org/onedrive/) and
> [abraunegg/onedrive](https://github.com/abraunegg/onedrive/blob/master/docs/national-cloud-deployments.md).
> They have not been tested against a corporate tenant. Menu names in the Entra
> portal change; if a step differs, follow the intent and the upstream pages.

Registering an application does not bypass your organization's rules. Conditional
access, device compliance, consent policy and a ban on user app registrations
still apply. Ask your IT department before using private credentials with
company data.

Which engine uses which IDs:

| Connection | Engine | Where the IDs go |
|---|---|---|
| OneDrive Offline mirror | `abraunegg/onedrive` | `config` file in the connection's `onedrive-sync/<connection-id>/` directory |
| SharePoint / Teams library | `rclone` | rclone remote (`rclone config`) |
| OneDrive Online mount | `jstaf/onedriver` | Not configurable from Cloud Mounter |

## 1. Register the application

1. Sign in to the [Microsoft Entra admin center](https://entra.microsoft.com)
   (or the Azure portal) with your work account.
2. Open **Entra ID > App registrations** (older portals: **Identity > Applications >
   App registrations**). Do not use **App Services**; that is a web hosting product.
3. Select **New registration** and enter:
   - **Name:** for example `Cloud Mounter`.
   - **Supported account types:** **Accounts in this organizational directory only
     (Single tenant)** is the right choice for company storage.
   - **Redirect URI:** depends on the engine, see below. You may leave it empty
     and add it later under **Authentication**.
4. Select **Register**.

If the button is greyed out or registration fails with an authorization error,
your tenant does not allow users to register applications. Ask IT to register one
for you, or to give you the Application (client) ID, Directory (tenant) ID and,
for rclone, a client secret.

## 2. Redirect URI and secret

Use the section for your engine.

### rclone (SharePoint, Teams libraries)

1. Open **Authentication > Add a platform > Web** and enter
   `http://localhost:53682/`. Type it rather than pasting it.
2. Open **Certificates & secrets > New client secret**, choose an expiry, and copy
   the secret **Value** immediately. It cannot be shown again. A new secret is needed
   when this one expires.

### abraunegg/onedrive (OneDrive Offline mirror)

1. Open **Authentication > Add a platform > Mobile and desktop applications** and
   enter `https://login.microsoftonline.com/common/oauth2/nativeclient`.
2. No client secret is needed. If sign-in later fails with an error saying a
   client secret or assertion is required (AADSTS7000218), open
   **Authentication > Advanced settings** and set **Allow public client flows**
   to **Yes**.
3. If sign-in reports a redirect URI mismatch (AADSTS50011), add the exact URI
   shown in the error message as an additional redirect URI.

## 3. Add API permissions

1. Open **API permissions > Add a permission > Microsoft Graph > Delegated
   permissions**. Delegated means the application acts only with the signed-in
   user's own access.
2. Add the permissions for your engine:

   | Engine | Delegated permissions |
   |---|---|
   | rclone | `Files.Read`, `Files.ReadWrite`, `Files.Read.All`, `Files.ReadWrite.All`, `offline_access`, `User.Read`, and `Sites.Read.All` (needed to search SharePoint sites) |
   | abraunegg/onedrive | `Files.ReadWrite`, `Files.ReadWrite.All`, `Sites.ReadWrite.All`, `offline_access` |

   `offline_access` is required. Without it the engine cannot refresh its sign-in
   and asks you to authorize again after about an hour.
3. Select **Add permissions**. If your tenant requires administrator approval, an
   administrator must select **Grant admin consent** for these permissions; the
   user cannot do it.

## 4. Copy the IDs

On the application's **Overview** page, copy:

- **Application (client) ID**
- **Directory (tenant) ID**

## 5. Give the IDs to the engine

### rclone

Run `rclone config` and create or edit the OneDrive remote. Enter the Application
ID as `client_id` and the secret value as `client_secret`. Because the application
is single-tenant, also set these in the remote's advanced options, replacing
`TENANT_ID` with the Directory (tenant) ID:

```text
auth_url  = https://login.microsoftonline.com/TENANT_ID/oauth2/v2.0/authorize
token_url = https://login.microsoftonline.com/TENANT_ID/oauth2/v2.0/token
```

Then use that remote name in Cloud Mounter. Without the two tenant URLs, sign-in
through the shared `common` endpoint is rejected for a single-tenant application.

### abraunegg/onedrive

In the connection's `config` file add:

```text
application_id = "APPLICATION_CLIENT_ID"
azure_tenant_id = "DIRECTORY_TENANT_ID"
```

`azure_tenant_id` makes the client use your tenant instead of the shared `common`
endpoint, which a single-tenant application needs. Then authorize the connection
again so the client obtains a token for the new application. Existing tokens were
issued to the old application ID and do not carry over.

## Troubleshooting

| Message | Likely cause |
|---|---|
| Redirect URI mismatch (AADSTS50011) | The URI in step 2 does not match what the engine sends. Add the exact URI from the error. |
| Application not found in the directory (AADSTS700016) | Wrong tenant or client ID, or the single-tenant URLs/`azure_tenant_id` are missing. |
| Needs admin approval / consent required | Your tenant restricts user consent. Ask an administrator to grant consent. |
| Client secret or assertion required (AADSTS7000218) | Enable **Allow public client flows** (abraunegg) or supply the client secret (rclone). |
| Sign-in repeats after about an hour | `offline_access` was not added or not consented. |
