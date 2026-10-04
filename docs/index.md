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

- [Project source and documentation](https://github.com/uutzinger/cosmic-ext-applet-mounter)
- [Privacy Policy](privacy/)
