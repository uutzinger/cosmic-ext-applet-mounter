---
title: Cloud Mounter for COSMIC
---

# Cloud Mounter for COSMIC

Cloud Mounter is a desktop applet for managing cloud and network storage on the
COSMIC desktop. It supports OneDrive, SharePoint, Google Drive, Box, SMB, and
SFTP connections through online mounts and offline mirrors.

Cloud Mounter uses locally installed connection engines, including rclone,
onedriver, and onedrive. For Google Drive, rclone communicates directly between
the user's computer and Google. Cloud Mounter does not operate an intermediary
server and does not receive users' Google Drive files or OAuth tokens.

Users choose which storage account and folder to connect, whether to use an
online mount or offline mirror, and when a connection is active. Access can be
revoked through the user's Google Account or by removing the corresponding
rclone remote.

## Documentation

### General description

[README](https://github.com/uutzinger/cosmic-ext-applet-mounter/blob/master/README.md)

### Setup and installation

- [Dependency installation](dependency-installation.md): supported engine
  versions and automated/manual dependency setup.
- [Build from source](build-from-source.md): development, verification, native
  installation, and package-building commands.
- [OpenConnect setup](openconnect.md): NetworkManager VPN profiles and the
  on-demand authentication agent.
- [Google Cloud authentication setup](google-cloud-auth-setup.md): private
  Google OAuth clients for rclone.
- [Microsoft cloud authentication setup](microsoft-cloud-auth-setup.md): your own
  Microsoft Entra application ID for OneDrive and SharePoint.

Run commands that refer to repository files from the checkout's root directory.

### Release and engineering records

- [Release notes](releases/README.md): versioned changes, compatibility,
  limitations, and delivery information.
- [Engineering records](engineering/README.md): baseline inventories, accepted
  safety contracts, and implementation/verification audits.

The [GitHub documentation entry](README.md) links to the central project
description, specifications, task records, and Flatpak packaging guide retained
elsewhere in the repository.

### Project and privacy

- [Project source and documentation](https://github.com/uutzinger/cosmic-ext-applet-mounter)
- [Privacy Policy](privacy/)
