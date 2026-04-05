---
title: Minimal automated updates for Alpine Linux
slug: minimal-automated-updates-for-alpine-linux
date: 2022-07-16
publish_date: 2022-07-16
tags: security, sysadmin
description: Many Linux distros have a way to configure automated updates but somewhat surprisingly Alpine Linux does not.
cover_image: alpine-linux.webp
---

Alpine Linux does run a hardened kernel, I always add a firewall to my servers, lock down SSH access to my IP address, and follow various other server security best practices so I shouldn't have any security problems but I do like to keep things updated.

I've found a very straight forward way of keeping my Alpine Linux servers up-to-date. For every new Alpine Linux server I make I always create a simple shell script in my `/etc/periodic/daily/` folder named `apk-autoupgrade` with the permissions `700`:

```shell
#!/bin/sh
apk upgrade --update | sed "s/^/[`date`] /" >> /var/log/apk-autoupgrade.log
```

You can create this yourself or run the following to create it in a single command:

```shell
echo -e "#!/bin/sh\napk upgrade --update | sed \"s/^/[\`date\`] /\" >> /var/log/apk-autoupgrade.log" > /etc/periodic/daily/apk-autoupgrade && \
	chmod 700 /etc/periodic/daily/apk-autoupgrade
```

Your Alpine Linux server now auto-updates itself, assuming you have cron jobs running. You can also enable those easily with:

```shell
apk add crond && \
	rc-service crond start && \
    rc-update add crond
```

What my script does is run the command `apk upgrade --update` once a day. Luckily `apk` by default never asks for user input so it should always just work. The rest of the script is to help you out by providing some logging. You can check the `/var/log/apk-autoupgrade.log` file every now and then to make sure everything is running smoothly. As an example of the output here's one of my servers:

```shell
[Wed Jul  6 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/main/x86_64/APKINDEX.tar.gz
[Wed Jul  6 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/community/x86_64/APKINDEX.tar.gz
[Wed Jul  6 02:00:00 UTC 2022] (1/4) Upgrading libcrypto1.1 (1.1.1p-r0 -> 1.1.1q-r0)
[Wed Jul  6 02:00:00 UTC 2022] (2/4) Upgrading libssl1.1 (1.1.1p-r0 -> 1.1.1q-r0)
[Wed Jul  6 02:00:00 UTC 2022] (3/4) Upgrading openssl (1.1.1p-r0 -> 1.1.1q-r0)
[Wed Jul  6 02:00:00 UTC 2022] (4/4) Upgrading openssl-doc (1.1.1p-r0 -> 1.1.1q-r0)
[Wed Jul  6 02:00:00 UTC 2022] Executing busybox-1.35.0-r14.trigger
[Wed Jul  6 02:00:00 UTC 2022] Executing ca-certificates-20211220-r0.trigger
[Wed Jul  6 02:00:00 UTC 2022] OK: 512 MiB in 253 packages
[Thu Jul  7 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/main/x86_64/APKINDEX.tar.gz
[Thu Jul  7 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/community/x86_64/APKINDEX.tar.gz
[Thu Jul  7 02:00:00 UTC 2022] OK: 512 MiB in 253 packages
[Fri Jul  8 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/main/x86_64/APKINDEX.tar.gz
[Fri Jul  8 02:00:00 UTC 2022] fetch http://dl-cdn.alpinelinux.org/alpine/latest-stable/community/x86_64/APKINDEX.tar.gz
[Fri Jul  8 02:00:00 UTC 2022] (1/1) Upgrading sgdisk (1.0.9-r1 -> 1.0.9-r2)
[Fri Jul  8 02:00:00 UTC 2022] Executing busybox-1.35.0-r14.trigger
[Fri Jul  8 02:00:00 UTC 2022] OK: 512 MiB in 253 packages
```

The one thing this script doesn't do is restart services once they are updated so that's something you'll need to determine yourself. In the future I may expand upon the script by checking for kernel updates or updates to running services and reboot the system or the service based off of the log output. As of right now I automatically reboot my servers every Sunday night since reboots happen almost instantly. If I hear of something critical then I'll just run a quick manual reboot.

That's it, a minimal solution to keep your Alpine Linux systems updated.
